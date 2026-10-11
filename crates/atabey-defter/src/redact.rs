//! The redactor (R-23, spec §5.8, ADR-0011): replaces secrets with `«redacted:<kind>»` before
//! anything is written to the ledger, the app log or an IPC error.
//!
//! Two detectors run on the **original** text and their spans are merged, so the output is built
//! in one pass and never re-scanned:
//!
//! * **Known secrets**: every value currently in the keychain, found with Aho-Corasick in the
//!   plain, trimmed, JSON-escaped and `Debug`-escaped forms (each also with `+` for spaces), and
//!   again in a percent-decoded copy of the text (URL encoding).
//! * **Pattern rules**: key prefixes, bearer tokens, private-key blocks and `name=value` pairs
//!   whose value looks random.
//!
//! Nothing in this module logs: the redactor sits inside the app-log writer, and a `tracing`
//! call here would recurse into itself.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::sync::{Arc, OnceLock, PoisonError, RwLock};

use aho_corasick::{AhoCorasick, MatchKind};
use atabey_types::{AtabeyError, ErrorKind, Permission};
use regex::Regex;
use serde_json::{Map, Value};

use crate::entry::{Actor, EntryKind, LedgerEntry, Outcome};

/// What a redacted value was. Appears in the marker `«redacted:<kind>»`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactionKind {
    /// A value currently stored in the keychain.
    Keychain,
    /// A PEM private-key block.
    PrivateKey,
    /// A key with a well-known vendor prefix (`sk-…`, `ghp_…`).
    ApiKey,
    /// The token after `Bearer` or `Basic`.
    BearerToken,
    /// The random-looking value of a `*token*`, `*secret*`, `*key*` or `*password*` field.
    Credential,
}

impl RedactionKind {
    /// The `<kind>` in the marker, in snake_case.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keychain => "keychain",
            Self::PrivateKey => "private_key",
            Self::ApiKey => "api_key",
            Self::BearerToken => "bearer_token",
            Self::Credential => "credential",
        }
    }

    /// The text that replaces a redacted value.
    #[must_use]
    pub fn marker(self) -> &'static str {
        match self {
            Self::Keychain => "«redacted:keychain»",
            Self::PrivateKey => "«redacted:private_key»",
            Self::ApiKey => "«redacted:api_key»",
            Self::BearerToken => "«redacted:bearer_token»",
            Self::Credential => "«redacted:credential»",
        }
    }

    /// Higher wins when two spans merge: keychain > private key > API key > bearer > credential.
    fn priority(self) -> u8 {
        match self {
            Self::Keychain => 4,
            Self::PrivateKey => 3,
            Self::ApiKey => 2,
            Self::BearerToken => 1,
            Self::Credential => 0,
        }
    }
}

/// Why the known-secret set could not be replaced. The old set stays in force.
#[derive(Debug, thiserror::Error)]
pub enum RedactError {
    /// The matcher could not be built (for example, the secrets are too large in total).
    #[error("could not build the secret matcher: {0}")]
    Matcher(String),
}

/// One secret the keychain holds. Its `Debug` output never shows the value.
#[derive(Clone)]
pub struct KnownSecret {
    value: String,
}

impl KnownSecret {
    /// Wrap a secret value. Empty (or whitespace-only) values are ignored by the redactor.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl fmt::Debug for KnownSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("KnownSecret(<redacted>)")
    }
}

/// The matcher for the current set of known secrets. Deliberately has no `Debug`.
struct Known {
    matcher: Option<AhoCorasick>,
    secrets: usize,
}

struct Inner {
    known: RwLock<Arc<Known>>,
}

/// Replaces secrets with `«redacted:<kind>»`. Cheap to clone; all clones share one set of known
/// secrets, so the keychain can refresh it while writers hold their own handle.
#[derive(Clone)]
pub struct Redactor {
    inner: Arc<Inner>,
}

impl fmt::Debug for Redactor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Redactor")
            .field("known_secrets", &self.known().secrets)
            .finish()
    }
}

impl Default for Redactor {
    fn default() -> Self {
        Self::new()
    }
}

/// A half-open byte range of the original text and why it is redacted.
#[derive(Debug, Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
    kind: RedactionKind,
}

impl Redactor {
    /// A redactor with the pattern rules and no known secrets.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                known: RwLock::new(Arc::new(Known {
                    matcher: None,
                    secrets: 0,
                })),
            }),
        }
    }

    /// Replace the whole set of known secrets (the keychain calls this at startup and after
    /// `secret_set` / `secret_clear`). On error the old set stays in force.
    ///
    /// # Errors
    /// Returns [`RedactError::Matcher`] if the matcher can't be built.
    pub fn set_known_secrets(
        &self,
        secrets: impl IntoIterator<Item = KnownSecret>,
    ) -> Result<(), RedactError> {
        let mut patterns = BTreeSet::new();
        let mut count = 0;
        for secret in secrets {
            if add_variants(&secret.value, &mut patterns) {
                count += 1;
            }
        }
        let matcher = if patterns.is_empty() {
            None
        } else {
            Some(
                AhoCorasick::builder()
                    // Standard + overlapping search: two secrets that overlap in the text must
                    // both be covered, so the union of all hits is redacted.
                    .match_kind(MatchKind::Standard)
                    .build(&patterns)
                    .map_err(|error| RedactError::Matcher(error.to_string()))?,
            )
        };
        let known = Arc::new(Known {
            matcher,
            secrets: count,
        });
        *self
            .inner
            .known
            .write()
            .unwrap_or_else(PoisonError::into_inner) = known;
        Ok(())
    }

    fn known(&self) -> Arc<Known> {
        Arc::clone(
            &self
                .inner
                .known
                .read()
                .unwrap_or_else(PoisonError::into_inner),
        )
    }

    /// Redact `text`. Borrowed when nothing matched.
    #[must_use]
    pub fn redact<'a>(&self, text: &'a str) -> Cow<'a, str> {
        let mut spans = Vec::new();
        self.known().collect(text, &mut spans);
        match patterns() {
            Some(patterns) => patterns.collect(text, &mut spans),
            // The built-in rules failed to compile (a bug a unit test would catch). Fail closed.
            None => spans.push(whole(text, RedactionKind::Credential)),
        }
        apply(text, spans)
    }

    fn redact_in_place(&self, text: &mut String) {
        let replaced = match self.redact(text) {
            Cow::Owned(new) => Some(new),
            Cow::Borrowed(_) => None,
        };
        if let Some(new) = replaced {
            *text = new;
        }
    }

    /// Redact every string leaf and object key of `value`. An object field whose key contains
    /// `token`, `secret`, `key` or `password` also has its string value (or each string in an
    /// array) replaced when it looks random. Numbers and booleans are never touched.
    pub fn redact_json(&self, value: &mut Value) {
        match value {
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
            Value::String(text) => self.redact_in_place(text),
            Value::Array(items) => {
                for item in items {
                    self.redact_json(item);
                }
            }
            Value::Object(map) => {
                let old = std::mem::take(map);
                let mut new = Map::new();
                for (key, mut child) in old {
                    if name_is_sensitive(&key) {
                        self.redact_credential_value(&mut child);
                    } else {
                        self.redact_json(&mut child);
                    }
                    let mut redacted_key = self.redact(&key).into_owned();
                    let mut n = 1;
                    while new.contains_key(&redacted_key) {
                        redacted_key = format!("{}#{n}", self.redact(&key));
                        n += 1;
                    }
                    new.insert(redacted_key, child);
                }
                *map = new;
            }
        }
    }

    /// The value under a sensitive key: a random-looking string (or each string of an array) is
    /// replaced whole; anything else is redacted like any other JSON.
    fn redact_credential_value(&self, value: &mut Value) {
        match value {
            Value::String(text) if looks_random(text) => {
                *text = RedactionKind::Credential.marker().to_owned();
            }
            Value::Array(items) => {
                for item in items {
                    self.redact_credential_value(item);
                }
            }
            other => self.redact_json(other),
        }
    }

    /// Redact every free-text field of a ledger entry. `ts`, `id` and `correlation` are
    /// generated by the ledger and left alone. Destructuring without `..` makes a new field a
    /// compile error until it is handled here.
    pub fn redact_entry(&self, entry: &mut LedgerEntry) {
        let LedgerEntry {
            ts: _,
            id: _,
            correlation: _,
            actor,
            kind,
            summary,
            outcome,
            data,
        } = entry;
        match actor {
            Actor::User | Actor::Core => {}
            Actor::Extension(id) => self.redact_in_place(&mut id.0),
            Actor::Agent(name) => self.redact_in_place(name),
            Actor::Provider(id) => self.redact_in_place(&mut id.0),
        }
        match kind {
            EntryKind::UserAction { action } => self.redact_in_place(action),
            EntryKind::IpcCommand { command } => self.redact_in_place(command),
            EntryKind::ExtensionMessage { topic } => self.redact_in_place(topic),
            EntryKind::LlmCall { model } => self.redact_in_place(model),
            EntryKind::System { event } => self.redact_in_place(event),
        }
        self.redact_in_place(summary);
        match outcome {
            Outcome::Ok | Outcome::Cancelled => {}
            Outcome::Failed { code } => self.redact_in_place(code),
            Outcome::Denied { reason } => self.redact_in_place(reason),
        }
        self.redact_json(data);
    }

    /// Redact an error before it crosses a boundary: its `message` and every string in its kind.
    /// Calling it at the IPC boundary is wired in T-10.
    pub fn redact_error(&self, error: &mut AtabeyError) {
        let AtabeyError { kind, message } = error;
        match kind {
            ErrorKind::NotFound { what } => self.redact_in_place(what),
            ErrorKind::InvalidInput { field, reason } => {
                self.redact_in_place(field);
                self.redact_in_place(reason);
            }
            ErrorKind::PermissionDenied { extension, needed } => {
                self.redact_in_place(&mut extension.0);
                match needed {
                    Permission::UlakCommand(name)
                    | Permission::UlakSubscribe(name)
                    | Permission::UlakEmit(name) => self.redact_in_place(name),
                    Permission::Network(hosts) => self.redact_in_place(&mut hosts.0),
                    Permission::UiPanel(_) | Permission::ReadProjectFiles => {}
                }
            }
            ErrorKind::BudgetPaused {
                request: _,
                needed_usd: _,
                cap: _,
            } => {}
            ErrorKind::Provider {
                provider,
                status: _,
                detail,
            } => {
                self.redact_in_place(&mut provider.0);
                self.redact_in_place(detail);
            }
            ErrorKind::Timeout { after_ms: _ } => {}
            ErrorKind::Internal { summary } => self.redact_in_place(summary),
        }
        self.redact_in_place(message);
    }
}

// ---------------------------------------------------------------------------------------------
// Known secrets

/// Add every searchable form of `value` to `out`. Returns whether anything was added.
///
/// The forms are the plain and the whitespace-trimmed value, each escaped for JSON or for Rust's
/// `Debug` (and for one more such escape, since a `Debug`-formatted field lands in a JSON log
/// line), each also with spaces as `+` (form encoding). URL encoding is not listed here: it is
/// handled by decoding the text.
fn add_variants(value: &str, out: &mut BTreeSet<String>) -> bool {
    if value.is_empty() {
        return false;
    }
    let mut added = false;
    let escapers: [fn(&str) -> String; 3] = [|s| s.to_owned(), json_escape, debug_escape];
    for base in [value, value.trim()] {
        for first in escapers {
            let once = first(base);
            for second in escapers {
                let form = second(&once);
                if form.trim().is_empty() {
                    continue;
                }
                added = true;
                if form.contains(' ') {
                    out.insert(form.replace(' ', "+"));
                }
                out.insert(form);
            }
        }
    }
    added
}

/// A string as JSON would write it, without the surrounding quotes.
fn json_escape(value: &str) -> String {
    serde_json::to_string(value)
        .map(|quoted| strip_quotes(&quoted).to_owned())
        .unwrap_or_else(|_| value.to_owned())
}

/// A string as `{:?}` would write it, without the surrounding quotes.
fn debug_escape(value: &str) -> String {
    strip_quotes(&format!("{value:?}")).to_owned()
}

fn strip_quotes(quoted: &str) -> &str {
    quoted
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(quoted)
}

impl Known {
    fn collect(&self, text: &str, out: &mut Vec<Span>) {
        let Some(matcher) = &self.matcher else {
            return;
        };
        match matcher.try_find_overlapping_iter(text) {
            Ok(hits) => {
                for hit in hits {
                    out.push(Span {
                        start: hit.start(),
                        end: hit.end(),
                        kind: RedactionKind::Keychain,
                    });
                }
            }
            Err(_) => out.push(whole(text, RedactionKind::Keychain)),
        }
        // URL encoding is handled by decoding, not by listing encoded variants: encoders
        // disagree on what to escape and on hex case.
        let Some(decoded) = percent_decode(text) else {
            return;
        };
        match matcher.try_find_overlapping_iter(decoded.bytes.as_slice()) {
            Ok(hits) => {
                for hit in hits {
                    let start = decoded.starts.get(hit.start()).copied();
                    let end = hit
                        .end()
                        .checked_sub(1)
                        .and_then(|last| decoded.ends.get(last).copied());
                    match (start, end) {
                        (Some(start), Some(end)) => out.push(Span {
                            start,
                            end,
                            kind: RedactionKind::Keychain,
                        }),
                        _ => out.push(whole(text, RedactionKind::Keychain)),
                    }
                }
            }
            Err(_) => out.push(whole(text, RedactionKind::Keychain)),
        }
    }
}

/// A percent-decoded copy of a text, with the original byte range of every decoded byte.
struct Decoded {
    bytes: Vec<u8>,
    starts: Vec<usize>,
    ends: Vec<usize>,
}

/// Decode `%XX` escapes. `None` when the text has none (nothing to search twice).
fn percent_decode(text: &str) -> Option<Decoded> {
    let raw = text.as_bytes();
    let mut decoded = Decoded {
        bytes: Vec::with_capacity(raw.len()),
        starts: Vec::with_capacity(raw.len()),
        ends: Vec::with_capacity(raw.len()),
    };
    let mut changed = false;
    let mut i = 0;
    while i < raw.len() {
        let escaped = match (raw.get(i), raw.get(i + 1), raw.get(i + 2)) {
            (Some(b'%'), Some(&high), Some(&low)) => hex_value(high)
                .zip(hex_value(low))
                .map(|(high, low)| (high << 4) | low),
            _ => None,
        };
        if let Some(byte) = escaped {
            decoded.bytes.push(byte);
            decoded.starts.push(i);
            decoded.ends.push(i + 3);
            changed = true;
            i += 3;
        } else {
            decoded.bytes.push(raw[i]);
            decoded.starts.push(i);
            decoded.ends.push(i + 1);
            i += 1;
        }
    }
    changed.then_some(decoded)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------
// Pattern rules

/// Substrings that make a field name sensitive (case-insensitive).
const SENSITIVE_NAMES: [&str; 4] = ["token", "secret", "key", "password"];

fn name_is_sensitive(name: &str) -> bool {
    let lower = name.to_lowercase();
    SENSITIVE_NAMES.iter().any(|word| lower.contains(word))
}

/// The value of a sensitive field is redacted only if it looks random: at least 16 characters,
/// no whitespace, Shannon entropy of at least 3.5 bits per character, and not made only of
/// letters and `_ . -` (the thresholds of gitleaks' `generic-api-key` rule).
fn looks_random(value: &str) -> bool {
    let length = value.chars().count();
    if length < 16 || value.chars().any(char::is_whitespace) {
        return false;
    }
    if value
        .chars()
        .all(|c| c.is_alphabetic() || matches!(c, '_' | '.' | '-'))
    {
        return false;
    }
    entropy(value) >= 3.5
}

#[allow(clippy::cast_precision_loss)] // text lengths are far below 2^52
fn entropy(value: &str) -> f64 {
    let mut counts = std::collections::BTreeMap::new();
    let mut total = 0_usize;
    for c in value.chars() {
        *counts.entry(c).or_insert(0_usize) += 1;
        total += 1;
    }
    let total = total as f64;
    counts
        .values()
        .map(|&n| {
            let p = n as f64 / total;
            -p * p.log2()
        })
        .sum()
}

/// A bearer or basic token is redacted unless it is a plain word: all letters, and either
/// lowercase after the first character (`authentication`, `Authentication`) or all uppercase
/// (`REQUIRED`). So `Basic authentication required` stays, while any token with a digit or
/// punctuation, or mixed-case letters, is redacted.
fn looks_like_token(token: &str) -> bool {
    let mut chars = token.chars();
    let first = chars.next();
    let rest: Vec<char> = chars.collect();
    let all_letters =
        first.is_some_and(char::is_alphabetic) && rest.iter().all(|c| c.is_alphabetic());
    let plain_word = all_letters
        && (rest.iter().all(|c| c.is_lowercase())
            || first.is_some_and(char::is_uppercase) && rest.iter().all(|c| c.is_uppercase()));
    !plain_word
}

struct Patterns {
    private_key: Regex,
    api_key: Regex,
    bearer: Regex,
    credential: Regex,
}

fn patterns() -> Option<&'static Patterns> {
    static PATTERNS: OnceLock<Option<Patterns>> = OnceLock::new();
    PATTERNS.get_or_init(|| Patterns::build().ok()).as_ref()
}

impl Patterns {
    fn build() -> Result<Self, regex::Error> {
        Ok(Self {
            // BEGIN..END, or if the block is cut off, the base64 that follows the header.
            private_key: Regex::new(
                r"(?s)-----BEGIN[A-Z0-9 ]*PRIVATE KEY-----(?:.*?-----END[A-Z0-9 ]*PRIVATE KEY-----|[A-Za-z0-9+/=\s\\-]*)",
            )?,
            // `sk-` family (OpenAI, Anthropic `sk-ant-`, OpenRouter `sk-or-v1-`) and GitHub.
            api_key: Regex::new(
                r"\bsk-[A-Za-z0-9_-]{16,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_\w{22,}",
            )?,
            bearer: Regex::new(r"(?i)\b(?:bearer|basic)\s+([A-Za-z0-9._~+/=-]{8,})")?,
            // name, separator, value. The value stops before the marker characters so a value
            // that was already redacted is never matched again.
            credential: Regex::new(
                r#"(?i)[A-Za-z0-9_.\-]*(?:token|secret|key|password)[A-Za-z0-9_.\-]*(?:\\?["'])?\s*[=:]\s*(?:\\?["'])?([^\s"',;&)}\]«»\\]{16,})"#,
            )?,
        })
    }

    fn collect(&self, text: &str, out: &mut Vec<Span>) {
        for found in self.private_key.find_iter(text) {
            out.push(Span {
                start: found.start(),
                end: found.end(),
                kind: RedactionKind::PrivateKey,
            });
        }
        for found in self.api_key.find_iter(text) {
            out.push(Span {
                start: found.start(),
                end: found.end(),
                kind: RedactionKind::ApiKey,
            });
        }
        for captures in self.bearer.captures_iter(text) {
            if let Some(token) = captures.get(1).filter(|t| looks_like_token(t.as_str())) {
                out.push(Span {
                    start: token.start(),
                    end: token.end(),
                    kind: RedactionKind::BearerToken,
                });
            }
        }
        for captures in self.credential.captures_iter(text) {
            if let Some(value) = captures.get(1).filter(|v| looks_random(v.as_str())) {
                out.push(Span {
                    start: value.start(),
                    end: value.end(),
                    kind: RedactionKind::Credential,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Building the output

fn whole(text: &str, kind: RedactionKind) -> Span {
    Span {
        start: 0,
        end: text.len(),
        kind,
    }
}

/// Widen the spans to `char` boundaries, merge overlapping and touching ones, and build the
/// output once.
fn apply(text: &str, mut spans: Vec<Span>) -> Cow<'_, str> {
    spans.retain(|span| span.start < span.end);
    if spans.is_empty() {
        return Cow::Borrowed(text);
    }
    for span in &mut spans {
        span.end = span.end.min(text.len());
        span.start = span.start.min(span.end);
        while !text.is_char_boundary(span.start) {
            span.start -= 1;
        }
        while !text.is_char_boundary(span.end) {
            span.end += 1;
        }
    }
    spans.sort_by_key(|span| (span.start, span.end));

    let mut merged: Vec<Span> = Vec::with_capacity(spans.len());
    for span in spans {
        match merged.last_mut() {
            Some(last) if span.start <= last.end => {
                last.end = last.end.max(span.end);
                if span.kind.priority() > last.kind.priority() {
                    last.kind = span.kind;
                }
            }
            _ => merged.push(span),
        }
    }

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for span in merged {
        out.push_str(&text[cursor..span.start]);
        out.push_str(span.kind.marker());
        cursor = span.end;
    }
    out.push_str(&text[cursor..]);
    Cow::Owned(out)
}

// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A deterministic random-looking body of `len` characters. Fixtures are built at run time
    /// so no key-shaped literal ever sits in the repository (gitleaks scans history).
    fn body(len: usize, seed: u64) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (0..len)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let index = usize::try_from(state >> 33).unwrap_or(0) % ALPHABET.len();
                char::from(ALPHABET[index])
            })
            .collect()
    }

    fn red(redactor: &Redactor, text: &str) -> String {
        redactor.redact(text).into_owned()
    }

    fn redactor_with(secrets: &[&str]) -> Redactor {
        let redactor = Redactor::new();
        let set = redactor.set_known_secrets(secrets.iter().map(|s| KnownSecret::new(*s)));
        assert!(set.is_ok());
        redactor
    }

    #[test]
    fn pattern_rules_compile() {
        assert!(patterns().is_some());
    }

    #[test]
    fn api_key_prefixes_are_redacted_and_their_look_alikes_are_not() {
        let redactor = Redactor::new();
        let cases = [
            format!("sk-or-v1-{}", body(64, 1)),
            format!("sk-ant-api03-{}", body(90, 2)),
            format!("sk-proj-{}", body(48, 3)),
            format!("ghp_{}", body(36, 4)),
            format!("gho_{}", body(36, 5)),
            format!("ghu_{}", body(36, 6)),
            format!("ghs_{}", body(36, 7)),
            format!("ghr_{}", body(36, 8)),
            format!("github_pat_{}", body(82, 9)),
        ];
        for key in cases {
            let text = format!("request failed for {key} (HTTP 401)");
            let out = redactor.redact(&text);
            assert_eq!(
                out, "request failed for «redacted:api_key» (HTTP 401)",
                "{key}"
            );
        }
        for harmless in [
            "task-management-system-design-notes",
            "risk-assessment-for-the-quarterly-review",
            "sk-short",
            "the ghp_ prefix is documented",
        ] {
            assert!(
                matches!(redactor.redact(harmless), Cow::Borrowed(_)),
                "{harmless}"
            );
        }
    }

    #[test]
    fn bearer_and_basic_tokens_keep_the_scheme_word() {
        let redactor = Redactor::new();
        let token = body(40, 11);
        let out = red(&redactor, &format!("Authorization: Bearer {token}"));
        assert_eq!(out, "Authorization: Bearer «redacted:bearer_token»");
        let out = redactor.redact("authorization: basic dXNlcjpwYXNzd29yZA==");
        assert_eq!(out, "authorization: basic «redacted:bearer_token»");
        let mixed: String = body(24, 12)
            .chars()
            .filter(char::is_ascii_alphabetic)
            .collect();
        assert!(
            mixed.chars().any(|c| c.is_ascii_uppercase())
                && mixed.chars().any(|c| c.is_ascii_lowercase())
        );
        let out = red(&redactor, &format!("Bearer {mixed}"));
        assert_eq!(out, "Bearer «redacted:bearer_token»");
        assert_eq!(
            red(&redactor, "Bearer snake_case_token"),
            "Bearer «redacted:bearer_token»"
        );
        for harmless in [
            "Basic authentication required",
            "Bearer Authentication",
            "Basic REQUIRED",
            "token_type: bearer",
            "the bearer of this message",
        ] {
            assert!(
                matches!(redactor.redact(harmless), Cow::Borrowed(_)),
                "{harmless}"
            );
        }
    }

    #[test]
    fn private_key_blocks_are_redacted_whole_or_to_the_cut() {
        let redactor = Redactor::new();
        let begin = format!("-----BEGIN {} KEY-----", "OPENSSH PRIVATE");
        let end = format!("-----END {} KEY-----", "OPENSSH PRIVATE");
        let block = format!("{begin}\n{}\n{}\n{end}", body(64, 21), body(64, 22));
        let out = red(&redactor, &format!("loaded:\n{block}\nthen continued"));
        assert_eq!(out, "loaded:\n«redacted:private_key»\nthen continued");

        let cut = format!("\"key\":\"{begin}\\n{}\\n", body(64, 23));
        let out = red(&redactor, &format!("{cut}\" rest"));
        assert!(!out.contains(&body(64, 23)), "{out}");
        assert!(out.ends_with("\" rest"), "{out}");
    }

    #[test]
    fn named_values_are_redacted_only_when_they_look_random() {
        let redactor = Redactor::new();
        let value = body(32, 31);
        let out = red(&redactor, &format!("api_key={value}&mode=fast"));
        assert_eq!(out, "api_key=«redacted:credential»&mode=fast");
        let out = red(&redactor, &format!("{{\"client_secret\": \"{value}\"}}"));
        assert_eq!(out, "{\"client_secret\": \"«redacted:credential»\"}");
        for harmless in [
            "token_type: \"bearer\"",
            "error_key: \"authentication_required\"",
            "prompt_tokens: 18432",
            "key: Enter",
            "password=hunter2",
            "monkey: bananas-and-oranges-from-the-market",
        ] {
            assert!(
                matches!(redactor.redact(harmless), Cow::Borrowed(_)),
                "{harmless}"
            );
        }
    }

    #[test]
    fn credentials_in_once_escaped_json_are_redacted() {
        let redactor = Redactor::new();
        let value = body(32, 91);
        let out = red(
            &redactor,
            &format!(
                "body=\"{{\\\"client_secret\\\":\\\"{value}\\\",\\\"token_type\\\":\\\"bearer\\\"}}\""
            ),
        );
        assert!(!out.contains(&value), "{out}");
        assert!(out.contains("\\\"«redacted:credential»\\\""), "{out}");
        assert!(out.contains("\\\"token_type\\\":\\\"bearer\\\""), "{out}");
        assert_eq!(red(&redactor, &out), out);
        let negative = "\\\"token_type\\\":\\\"bearer\\\"";
        assert!(matches!(redactor.redact(negative), Cow::Borrowed(_)));
    }

    #[test]
    fn json_keys_decide_which_values_are_credentials() {
        let redactor = Redactor::new();
        let secret = body(40, 41);
        let mut value = json!({
            "prompt_tokens": 18432,
            "cache": true,
            "key": "Enter",
            "token_type": "bearer",
            "auth_token": secret,
            "session_keys": [body(24, 42), "short"],
            "note": "plain text",
        });
        redactor.redact_json(&mut value);
        assert_eq!(
            value,
            json!({
                "prompt_tokens": 18432,
                "cache": true,
                "key": "Enter",
                "token_type": "bearer",
                "auth_token": "«redacted:credential»",
                "session_keys": ["«redacted:credential»", "short"],
                "note": "plain text",
            })
        );
    }

    #[test]
    fn json_object_keys_are_redacted_and_never_collide() {
        let first = body(20, 51);
        let second = body(20, 52);
        let redactor = redactor_with(&[&first, &second]);
        let mut value = json!({ format!("a {first}"): 1, format!("b {second}"): 2 });
        redactor.redact_json(&mut value);
        let text = value.to_string();
        assert!(!text.contains(&first) && !text.contains(&second), "{text}");
        assert_eq!(value.as_object().map(Map::len), Some(2));
    }

    #[test]
    fn known_secrets_are_found_in_every_form() {
        let secret = "pa ss\"wo\\rd&é%41=+/ 7Hq";
        let redactor = redactor_with(&[secret]);
        let json_form = serde_json::to_string(secret).unwrap_or_default();
        let json_form = strip_quotes(&json_form);
        let debug_form = format!("{secret:?}");
        let debug_form = strip_quotes(&debug_form);
        let url_form: String =
            percent_encoding::utf8_percent_encode(secret, percent_encoding::NON_ALPHANUMERIC)
                .to_string();
        let plus_form = secret.replace(' ', "+");
        for form in [secret, json_form, debug_form, &url_form, &plus_form] {
            let out = red(&redactor, &format!("before {form} after"));
            assert_eq!(out, "before «redacted:keychain» after", "{form}");
        }
        let mut lower = String::new();
        let mut hex_digits_left = 0;
        for c in url_form.chars() {
            if c == '%' {
                hex_digits_left = 2;
                lower.push(c);
            } else if hex_digits_left > 0 {
                hex_digits_left -= 1;
                lower.push(c.to_ascii_lowercase());
            } else {
                lower.push(c);
            }
        }
        let out = red(&redactor, &format!("before {lower} after"));
        assert_eq!(out, "before «redacted:keychain» after", "{lower}");
    }

    #[test]
    fn overlapping_secrets_leave_no_tail() {
        let redactor = redactor_with(&["abcdef12", "ef12ghij"]);
        let out = redactor.redact("x abcdef12ghij y");
        assert_eq!(out, "x «redacted:keychain» y");
    }

    #[test]
    fn keychain_wins_over_a_pattern_on_the_same_span() {
        let key = format!("sk-or-v1-{}", body(64, 61));
        let redactor = redactor_with(&[&key]);
        assert_eq!(red(&redactor, &format!("[{key}]")), "[«redacted:keychain»]");
    }

    #[test]
    fn set_known_secrets_replaces_the_set() {
        let redactor = redactor_with(&["first-secret-value"]);
        assert!(
            redactor
                .set_known_secrets([KnownSecret::new("second-secret-value")])
                .is_ok()
        );
        assert!(matches!(
            redactor.redact("first-secret-value"),
            Cow::Borrowed(_)
        ));
        assert_eq!(
            redactor.redact("second-secret-value"),
            "«redacted:keychain»"
        );
        assert!(redactor.set_known_secrets([KnownSecret::new("")]).is_ok());
        assert!(matches!(
            redactor.redact("second-secret-value"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn spans_are_widened_to_char_boundaries() {
        // A span that cuts "é" in half must not panic or split the character.
        let text = "aé";
        let out = apply(
            text,
            vec![Span {
                start: 2,
                end: 3,
                kind: RedactionKind::Credential,
            }],
        );
        assert_eq!(out, "a«redacted:credential»");
    }

    #[test]
    fn debug_output_never_shows_a_secret() {
        let secret = body(24, 71);
        let redactor = redactor_with(&[&secret]);
        let known = KnownSecret::new(secret.clone());
        assert!(!format!("{known:?}").contains(&secret));
        assert!(!format!("{redactor:?}").contains(&secret));
        assert!(format!("{redactor:?}").contains('1'));
    }

    #[test]
    fn redact_error_cleans_the_detail_and_the_message() {
        let key = format!("sk-or-v1-{}", body(64, 81));
        let redactor = Redactor::new();
        let mut error = AtabeyError::from(ErrorKind::Provider {
            provider: "openrouter".into(),
            status: Some(401),
            detail: format!("invalid key {key}"),
        });
        assert!(error.message.contains(&key));
        redactor.redact_error(&mut error);
        assert!(!error.message.contains(&key), "{}", error.message);
        match &error.kind {
            ErrorKind::Provider { detail, .. } => {
                assert_eq!(detail, "invalid key «redacted:api_key»");
            }
            other => panic!("kind changed: {other:?}"),
        }
    }
}
