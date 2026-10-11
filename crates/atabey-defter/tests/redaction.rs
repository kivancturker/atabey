//! R-23 and AC-11: a secret in the keychain never reaches the ledger or the app log, in any
//! position or encoding tested.
//!
//! Every key-shaped fixture is built at run time: gitleaks scans this repository's history with
//! its default rules, so a literal key in a test would fail the gate.

mod common;

use std::fs;
use std::path::Path;

use atabey_defter::{
    Actor, EntryDraft, EntryKind, KnownSecret, LedgerEntry, LogFormat, Outcome, Redactor, app_log,
};
use atabey_types::{AtabeyError, ErrorKind};
use common::*;
use percent_encoding::{AsciiSet, CONTROLS, NON_ALPHANUMERIC, utf8_percent_encode};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use serde_json::json;
use tracing_subscriber::EnvFilter;

const MARKER: &str = "«redacted:keychain»";

/// What a typical form or query encoder escapes besides controls: RFC 3986 forbids a raw space,
/// quote, `%` and backslash, so conforming encoders never leave those. It still leaves other
/// punctuation (`/`, `:`, `!`) alone, unlike `NON_ALPHANUMERIC`.
const FORM_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'\\')
    .add(b'#')
    .add(b'%')
    .add(b'&')
    .add(b'+')
    .add(b'=')
    .add(b'?');
/// RFC 3986 "unreserved" characters stay as they are.
const STRICT_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn fail(error: impl std::fmt::Display) -> TestCaseError {
    TestCaseError::fail(error.to_string())
}

// ---------------------------------------------------------------------------------------------
// Generators and encodings

fn secret_strategy() -> impl Strategy<Value = String> {
    let special = proptest::sample::select(vec![
        '"', '\\', ' ', '&', '%', '\n', '\t', '+', '/', '=', '#', '?', '\'', '\r',
    ]);
    let character = prop_oneof![
        4 => proptest::char::range('a', 'z'),
        3 => proptest::char::range('A', 'Z'),
        2 => proptest::char::range('0', '9'),
        3 => special,
        2 => proptest::char::range('!', '~'),
        2 => proptest::char::range('\u{a1}', '\u{2fff}'),
    ];
    proptest::collection::vec(character, 8..=64).prop_map(|chars| {
        chars
            .into_iter()
            .map(|c| if matches!(c, '«' | '»') { 'x' } else { c })
            .collect()
    })
}

#[derive(Debug, Clone, Copy)]
enum Encoding {
    Plain,
    Json,
    UrlEverything,
    UrlFormSet,
    UrlStrict,
    UrlLowerHex,
    FormPlus,
}

const ENCODINGS: [Encoding; 7] = [
    Encoding::Plain,
    Encoding::Json,
    Encoding::UrlEverything,
    Encoding::UrlFormSet,
    Encoding::UrlStrict,
    Encoding::UrlLowerHex,
    Encoding::FormPlus,
];

fn json_escape(value: &str) -> String {
    let quoted = serde_json::to_string(value).unwrap_or_default();
    quoted
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(&quoted)
        .to_owned()
}

fn debug_escape(value: &str) -> String {
    let quoted = format!("{value:?}");
    quoted
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(&quoted)
        .to_owned()
}

fn lower_hex(encoded: &str) -> String {
    let mut out = String::new();
    let mut digits_left = 0;
    for c in encoded.chars() {
        if c == '%' {
            digits_left = 2;
            out.push(c);
        } else if digits_left > 0 {
            digits_left -= 1;
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn encode(secret: &str, encoding: Encoding) -> String {
    match encoding {
        Encoding::Plain => secret.to_owned(),
        Encoding::Json => json_escape(secret),
        Encoding::UrlEverything => utf8_percent_encode(secret, NON_ALPHANUMERIC).to_string(),
        Encoding::UrlFormSet => utf8_percent_encode(secret, FORM_SET).to_string(),
        Encoding::UrlStrict => utf8_percent_encode(secret, STRICT_SET).to_string(),
        Encoding::UrlLowerHex => {
            lower_hex(&utf8_percent_encode(secret, NON_ALPHANUMERIC).to_string())
        }
        Encoding::FormPlus => secret.replace(' ', "+"),
    }
}

/// Every form in which the secret must not appear in an output file: each encoding, and the
/// extra layer of escaping the output format itself adds (a JSON log line or ledger line).
fn forbidden_forms(secret: &str) -> Vec<String> {
    let mut forms: Vec<String> = ENCODINGS.iter().map(|e| encode(secret, *e)).collect();
    for escape in [json_escape, debug_escape] {
        let once = escape(secret);
        forms.push(json_escape(&once));
        forms.push(debug_escape(&once));
        forms.push(once);
    }
    forms.retain(|form| !form.trim().is_empty());
    forms
}

fn leaks(output: &str, secret: &str) -> bool {
    forbidden_forms(secret)
        .iter()
        .any(|form| output.contains(form.as_str()))
}

#[derive(Debug, Clone, Copy)]
enum Place {
    Start,
    Middle,
    End,
}

impl Place {
    fn wrap(self, text: &str) -> String {
        match self {
            Self::Start => format!("{text} was rejected"),
            Self::Middle => format!("rejected {text} twice"),
            Self::End => format!("rejected by {text}"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Where {
    Summary,
    DataString,
    DataNested,
    DataKey,
    ActorAgent,
    DeniedReason,
    KindName,
}

fn draft_with(place: Where, text: &str) -> EntryDraft {
    let base = EntryDraft::new(
        Actor::User,
        EntryKind::IpcCommand {
            command: "secret_set".into(),
        },
        "Store a provider key",
    );
    match place {
        Where::Summary => EntryDraft::new(
            Actor::User,
            EntryKind::IpcCommand {
                command: "secret_set".into(),
            },
            text,
        ),
        Where::DataString => base.data(json!({ "note": text, "count": 3 })),
        Where::DataNested => base.data(json!({ "items": [{ "detail": [text] }] })),
        Where::DataKey => base.data(json!({ text: 1 })),
        Where::ActorAgent => EntryDraft::new(
            Actor::Agent(text.to_owned()),
            EntryKind::System {
                event: "start".into(),
            },
            "Agent started",
        ),
        Where::DeniedReason => base.outcome(Outcome::Denied {
            reason: text.to_owned(),
        }),
        Where::KindName => EntryDraft::new(
            Actor::Core,
            EntryKind::UserAction {
                action: text.to_owned(),
            },
            "User action",
        ),
    }
}

const WHERES: [Where; 7] = [
    Where::Summary,
    Where::DataString,
    Where::DataNested,
    Where::DataKey,
    Where::ActorAgent,
    Where::DeniedReason,
    Where::KindName,
];

fn encoding_strategy() -> impl Strategy<Value = Encoding> {
    proptest::sample::select(ENCODINGS.to_vec())
}

fn place_strategy() -> impl Strategy<Value = Place> {
    proptest::sample::select(vec![Place::Start, Place::Middle, Place::End])
}

fn where_strategy() -> impl Strategy<Value = Where> {
    proptest::sample::select(WHERES.to_vec())
}

/// Whether a random secret is a fair test subject: it must not coincide with fixed text.
fn usable(secret: &str) -> bool {
    !secret.trim().is_empty() && !MARKER.contains(secret)
}

fn read_only_file(dir: &Path) -> Result<String, TestCaseError> {
    let mut files = Vec::new();
    collect_files(dir, &mut files).map_err(fail)?;
    prop_assert_eq_len(&files)?;
    fs::read_to_string(&files[0]).map_err(fail)
}

fn prop_assert_eq_len(files: &[std::path::PathBuf]) -> Result<(), TestCaseError> {
    if files.len() == 1 {
        Ok(())
    } else {
        Err(fail(format!("expected one file, found {files:?}")))
    }
}

fn collect_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// AC-11

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// AC-11 (ledger): a random keychain secret never reaches the ledger file.
    #[test]
    fn ac11_ledger_never_contains_a_keychain_secret(
        secret in secret_strategy(),
        encoding in encoding_strategy(),
        place in place_strategy(),
        wherever in where_strategy(),
    ) {
        prop_assume!(usable(&secret));
        let text = place.wrap(&encode(&secret, encoding));

        // A control write without the secret: if the file already contains it, the secret
        // coincides with fixed text and the case says nothing.
        let control_dir = tempfile::tempdir().map_err(fail)?;
        let control = ledger_in(control_dir.path(), clock_at(MORNING));
        control.begin_flow().record(draft_with(wherever, &place.wrap(""))).map_err(fail)?;
        let control_text = read_only_file(control_dir.path())?;
        prop_assume!(!leaks(&control_text, &secret));

        let keychain = KeychainMock::new();
        keychain.set("openrouter", &secret).map_err(fail)?;
        let dir = tempfile::tempdir().map_err(fail)?;
        let ledger = atabey_defter::Ledger::new(
            dir.path(), project_id(), session_id(), clock_at(MORNING), keychain.redactor(),
        );
        ledger.begin_flow().record(draft_with(wherever, &text)).map_err(fail)?;

        let written = read_only_file(dir.path())?;
        prop_assert!(!leaks(&written, &secret), "leaked in {written}");
        prop_assert!(written.contains(MARKER), "no marker in {written}");
        let line = written.lines().next().ok_or_else(|| fail("empty file"))?;
        serde_json::from_str::<LedgerEntry>(line).map_err(fail)?;
    }

    /// AC-11 (app log): the same, for text and JSON log lines, with the secret in the message,
    /// a `%` field, a `?` field and a span field.
    #[test]
    fn ac11_app_log_never_contains_a_keychain_secret(
        secret in secret_strategy(),
        encoding in encoding_strategy(),
        place in place_strategy(),
        site in proptest::sample::select(vec![0_u8, 1, 2, 3]),
        json_format in any::<bool>(),
    ) {
        prop_assume!(usable(&secret));
        let format = if json_format { LogFormat::Json } else { LogFormat::Text };

        let emit = |redactor: Redactor, text: &str| -> Result<String, TestCaseError> {
            let dir = tempfile::tempdir().map_err(fail)?;
            let log = app_log(dir.path(), format, EnvFilter::new("info"), redactor).map_err(fail)?;
            tracing::dispatcher::with_default(&log.dispatch, || match site {
                0 => tracing::info!("{text}"),
                1 => tracing::info!(note = %text, "provider call failed"),
                2 => tracing::info!(note = ?text, "provider call failed"),
                _ => {
                    let span = tracing::info_span!("job", note = %text);
                    span.in_scope(|| tracing::info!("provider call failed"));
                }
            });
            drop(log);
            read_only_file(dir.path())
        };

        let control = emit(Redactor::new(), &place.wrap(""))?;
        prop_assume!(!leaks(&control, &secret));

        let keychain = KeychainMock::new();
        keychain.set("openrouter", &secret).map_err(fail)?;
        let written = emit(keychain.redactor(), &place.wrap(&encode(&secret, encoding)))?;

        prop_assert!(!leaks(&written, &secret), "leaked in {written}");
        // A JSON-escaped secret inside a Debug-formatted field of a JSON line is escaped three
        // times. That depth is a documented limit (ADR-0011), so no marker is demanded there.
        let three_layers = json_format && site == 2 && matches!(encoding, Encoding::Json);
        prop_assert!(three_layers || written.contains(MARKER), "no marker in {written}");
        if json_format {
            for line in written.lines() {
                serde_json::from_str::<serde_json::Value>(line).map_err(fail)?;
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Pure redaction properties

fn alnum(min: usize, max: usize) -> impl Strategy<Value = String> {
    let chars: Vec<char> = ('a'..='z').chain('A'..='Z').chain('0'..='9').collect();
    proptest::collection::vec(proptest::sample::select(chars), min..=max)
        .prop_map(|chars| chars.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    /// Two secrets that overlap in the text: neither survives, not even the tail of the second.
    #[test]
    fn overlapping_secrets_leave_nothing_behind(
        head in alnum(4, 12),
        shared in alnum(1, 6),
        tail in alnum(4, 12),
    ) {
        let first = format!("{head}{shared}");
        let second = format!("{shared}{tail}");
        let joined = format!("{head}{shared}{tail}");
        let keychain = KeychainMock::new();
        keychain.set("a", &first).map_err(fail)?;
        keychain.set("b", &second).map_err(fail)?;
        let out = keychain.redactor().redact(&format!("start: {joined} :end")).into_owned();
        prop_assert_eq!(out, format!("start: {MARKER} :end"));
    }

    /// `redact(redact(x)) == redact(x)`, for arbitrary text and for text made of secret-shaped
    /// pieces.
    #[test]
    fn redaction_is_idempotent(
        pieces in proptest::collection::vec(piece_strategy(), 1..8),
        noise in any::<String>(),
    ) {
        let keychain = KeychainMock::new();
        keychain.set("a", KNOWN_A).map_err(fail)?;
        keychain.set("b", KNOWN_B).map_err(fail)?;
        let redactor = keychain.redactor();
        for text in [pieces.join(" "), pieces.join("&"), noise] {
            let once = redactor.redact(&text).into_owned();
            let twice = redactor.redact(&once).into_owned();
            prop_assert_eq!(&once, &twice, "from {:?}", text);
        }
    }

    /// Realistic log sentences come back untouched (borrowed).
    #[test]
    fn ordinary_log_lines_are_not_redacted(
        words in proptest::collection::vec(proptest::sample::select(WORDS.to_vec()), 1..14),
    ) {
        let keychain = KeychainMock::new();
        keychain.set("a", KNOWN_A).map_err(fail)?;
        let redactor = keychain.redactor();
        let line = words.join(" ");
        prop_assert!(
            matches!(redactor.redact(&line), std::borrow::Cow::Borrowed(_)),
            "over-redacted: {line}"
        );
    }
}

const KNOWN_A: &str = "tr0ub4dor&3-horse-battery-staple";
const KNOWN_B: &str = "ghi-Jkl-0987654321-mno";

const WORDS: [&str; 24] = [
    "project",
    "opened",
    "in",
    "the",
    "editor",
    "/Users/kivancturker/Dev/harness",
    "budget",
    "cap",
    "reached",
    "retry",
    "after",
    "20",
    "s",
    "request",
    "429",
    "model",
    "anthropic/claude-sonnet-5",
    "prompt_tokens",
    "18432",
    "openrouter",
    "task-management-notes",
    "risk-assessment",
    "2026-10-12T09:30:15.250Z",
    "01920f4e-8c10-7a55-8f31-2b6d9e4a7c03",
];

fn piece_strategy() -> impl Strategy<Value = String> {
    let pem_begin = format!("-----BEGIN {} KEY-----", "RSA PRIVATE");
    let pem_end = format!("-----END {} KEY-----", "RSA PRIVATE");
    let fixed = vec![
        "api_key=".to_owned(),
        "token: ".to_owned(),
        "\"secret\":\"".to_owned(),
        "password=\"".to_owned(),
        "Bearer ".to_owned(),
        "basic ".to_owned(),
        "sk-".to_owned(),
        "sk-or-v1-".to_owned(),
        "ghp_".to_owned(),
        "%2B".to_owned(),
        "%41".to_owned(),
        pem_begin,
        pem_end,
        KNOWN_A.to_owned(),
        KNOWN_B.to_owned(),
        "«redacted:keychain»".to_owned(),
        "\n".to_owned(),
    ];
    prop_oneof![
        3 => proptest::sample::select(fixed),
        3 => alnum(8, 48),
        2 => "[a-z ]{0,12}",
    ]
}

// ---------------------------------------------------------------------------------------------
// Deterministic cases

#[test]
fn a_secret_set_after_the_ledger_was_created_is_redacted_from_then_on() -> TestResult {
    let root = tempfile::tempdir()?;
    let keychain = KeychainMock::new();
    let ledger = atabey_defter::Ledger::new(
        root.path(),
        project_id(),
        session_id(),
        clock_at(MORNING),
        keychain.redactor(),
    );
    let flow = ledger.begin_flow();
    let secret = &["hunter2", "correct", "horse", "battery"].join("-");
    let draft = || {
        EntryDraft::new(
            Actor::User,
            EntryKind::IpcCommand {
                command: "echo".into(),
            },
            format!("pasted {secret} into a prompt"),
        )
    };

    let before = flow.record(draft())?;
    keychain.set("openrouter", secret)?;
    let after = flow.record(draft())?;
    keychain.clear("openrouter")?;
    let cleared = flow.record(draft())?;

    assert!(before.summary.contains(secret));
    assert_eq!(after.summary, format!("pasted {MARKER} into a prompt"));
    assert!(
        cleared.summary.contains(secret),
        "clearing removes the secret from the set"
    );
    let on_disk = read_entries(&ledger.path_for(MORNING.date()))?;
    assert_eq!(on_disk, vec![before, after, cleared]);
    Ok(())
}

#[test]
fn set_known_secrets_replaces_instead_of_appending() -> TestResult {
    let keychain = KeychainMock::new();
    keychain.set("openrouter", "first-provider-secret")?;
    keychain.set("github", "second-provider-secret")?;
    let redactor = keychain.redactor();
    assert_eq!(redactor.redact("first-provider-secret"), MARKER);
    redactor.set_known_secrets([KnownSecret::new("third-provider-secret")])?;
    assert_eq!(
        redactor.redact("first-provider-secret"),
        "first-provider-secret"
    );
    assert_eq!(
        redactor.redact("second-provider-secret"),
        "second-provider-secret"
    );
    assert_eq!(redactor.redact("third-provider-secret"), MARKER);
    Ok(())
}

#[test]
fn a_provider_error_carrying_a_key_is_cleaned_in_detail_and_message() -> TestResult {
    let keychain = KeychainMock::new();
    let key = "unit-test-provider-key-0a1b2c3d4e5f";
    keychain.set("openrouter", key)?;
    let mut error = AtabeyError::from(ErrorKind::Provider {
        provider: "openrouter".into(),
        status: Some(401),
        detail: format!("invalid api key {key}"),
    });
    keychain.redactor().redact_error(&mut error);
    assert!(!error.message.contains(key), "{}", error.message);
    assert_eq!(
        error.message,
        format!("openrouter failed (HTTP 401): invalid api key {MARKER}")
    );
    assert!(matches!(
        &error.kind,
        ErrorKind::Provider { detail, .. } if detail == &format!("invalid api key {MARKER}")
    ));
    Ok(())
}

#[test]
fn pattern_rules_catch_keys_the_keychain_never_saw() -> TestResult {
    let dir = tempfile::tempdir()?;
    let ledger = ledger_in(dir.path(), clock_at(MORNING));
    let pasted = format!(
        "sk-or-v1-{}",
        "aB3dE5gH7jK9mN1pQ3sT5vW7yZ9bC1dE3fG5hJ7kL9mN1pQ3sT5vW7yZ9bC1dE3"
    );
    let entry = ledger.begin_flow().record(
        EntryDraft::new(
            Actor::User,
            EntryKind::UserAction {
                action: "paste".into(),
            },
            format!("pasted {pasted} into the prompt"),
        )
        .data(json!({ "Authorization": format!("Bearer {}", "x".repeat(8) + "9Qz7"), "prompt_tokens": 18432 })),
    )?;
    assert_eq!(entry.summary, "pasted «redacted:api_key» into the prompt");
    assert_eq!(entry.data["prompt_tokens"], 18432);
    assert_eq!(
        entry.data["Authorization"],
        "Bearer «redacted:bearer_token»"
    );
    Ok(())
}
