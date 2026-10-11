---
id: ADR-0011
title: "Redaction: known-secret matcher, pattern rules and the two write choke points"
status: proposed
date: 2026-10-12
decisions: [D-029]
spec: specs/S0-foundations.md R-23, AC-11, §5.8 (T-06)
---

# ADR-0011: Redaction: known-secret matcher, pattern rules and the two write choke points

## Context
R-23 says that before any write, values that match secret patterns, plus every secret currently in the keychain, are replaced with `«redacted:<kind>»`. Constitution Art. 5 ("leak no secret") depends on it. T-05 left one hook (`Ledger::commit`); T-06 builds the redactor and wires it into the ledger, the app log and (as a function, wired by T-10) IPC errors. The spec fixes the shape (Aho-Corasick for known secrets, pattern rules for the rest) but leaves open:

1. How the keychain, which arrives in T-08, tells the redactor about its secrets.
2. Which forms of a secret count as "the secret" (a secret rarely reaches a log byte for byte).
3. What the pattern rules are, and how they avoid redacting ordinary text.
4. Where the redactor sits, so that it cannot be forgotten.

## Decision

| Question | Decision |
|---|---|
| Keychain link | A **push API**, not a trait. `Redactor::set_known_secrets(impl IntoIterator<Item = KnownSecret>)` replaces the whole set. The keychain (or the T-10 composition root) calls it at startup and after `secret_set` / `secret_clear`. `atabey-defter` never depends on the keychain crate. On error the old set stays, so `secret_set` can refuse to store. Tests use a `KeychainMock` that does the same |
| Shared state | `Redactor` is `Clone` (an `Arc`); the set sits in an `RwLock<Arc<…>>` and is swapped after the new matcher is built, so a write never waits for a rebuild. A poisoned lock is reused, as in the ledger |
| Forms of a known secret | Plain and whitespace-trimmed, each escaped for **JSON**, for Rust **`Debug`** and for one more such layer (a `Debug`-formatted field written into a JSON log line), each also with spaces as `+` (form encoding). `Debug` matters because `tracing`'s text format writes `?field` values with `{:?}` |
| URL encoding | **Handled by decoding, not by listing variants.** The redactor builds a percent-decoded byte copy of the text with a map from each decoded byte back to its original bytes, searches both copies, and maps hits back. Encoders disagree on what to escape (`sk-or-…` becomes `sk%2Dor%2D…` in some) and on hex case; decoding covers all of them. The decoder is hand-written (about 40 lines); `percent-encoding` is a dev-dependency used by the tests to encode |
| Overlapping secrets | The matcher uses `MatchKind::Standard` and `find_overlapping_iter`; all hits are unioned. A non-overlapping search would leave the tail of a second secret visible when two secrets overlap in the text |
| One pass | Spans from the known matcher and from every pattern rule are collected on the **original** text, widened to `char` boundaries, merged (overlapping or touching), and the output is built once. Output is never re-scanned, so a marker cannot be redacted twice. The merged span's kind is the highest of `keychain > private_key > api_key > bearer_token > credential` |
| Pattern rules | See the table below. Rust `regex` (linear time, so hostile log text cannot cause ReDoS) |
| "Looks random" | At least 16 characters, no whitespace, Shannon entropy at least **3.5 bits per character**, and not made only of letters and `_ . -`. Both numbers come from gitleaks' `generic-api-key` rule. So `token_type: "bearer"` and `error_key: "authentication_required"` survive |
| JSON | `redact_json` redacts every string leaf and object key. For an object key containing `token`, `secret`, `key` or `password`, a string value (or each string in an array) that looks random is replaced whole. Numbers and booleans are never touched, so `"prompt_tokens": 18432` stays. Two keys that redact to the same text get a `#n` suffix instead of overwriting each other |
| Choke points | `Ledger::new(…, redactor)` and `app_log(…, redactor)` / `init_app_log(…, redactor)` take the redactor as a **required** argument, so a ledger or log without one cannot be built. `Ledger::commit` calls `redact_entry` before serializing and returns the redacted entry. The app log wraps its writer: `tracing-subscriber` formats one event into one buffer and writes it with one `write_all`, so the writer sees a whole line, redacts it (`String::from_utf8_lossy`, never raw bytes), and writes it on |
| Complete coverage | `redact_entry` destructures `LedgerEntry` and `match`es `Actor`, `EntryKind`, `Outcome` with no `..` and no wildcard arm; `redact_error` does the same for `AtabeyError` and every `ErrorKind` variant and `Permission`. A new field or variant fails to compile until it is handled |
| No logging inside | The redactor and the writer make no `tracing` calls: they sit inside the log writer and would recurse into themselves |
| Failing closed | If the built-in regexes ever failed to compile (a unit test guards against it), every text is replaced whole instead of passed through |
| Dependencies | `aho-corasick` 1.1.5 (Unlicense OR MIT), `regex` 1.13.1 (MIT OR Apache-2.0); both were already in `Cargo.lock`. Dev-only: `percent-encoding` 2.3.2 and `proptest` **locked at 1.11.0** (1.12.0 was published on 2026-10-10, under the two-week rule we apply to pnpm; versions checked on crates.io 2026-10-12) |

### Pattern rules

| Kind | Rule | Redacts | Source |
|---|---|---|---|
| `private_key` | `-----BEGIN … PRIVATE KEY-----` up to the matching `-----END … KEY-----`; if there is no end, the base64 that follows the header (it stops at a quote, so a cut-off block inside a JSON log line does not break the line) | The whole block | gitleaks 8.30.1 `private-key` rule |
| `api_key` | `sk-` followed by at least 16 of `[A-Za-z0-9_-]`, preceded by a word boundary (so `task-management-…` is not matched) | The key | Spec §5.8 names `sk-…` and `sk-or-…`; gitleaks 8.30.1 `anthropic-api-key` (`sk-ant-api03-…`) and `openai-api-key` (`sk-proj-…`) |
| `api_key` | `ghp_`, `gho_`, `ghu_`, `ghs_`, `ghr_` followed by at least 30 alphanumerics; `github_pat_` followed by at least 22 word characters | The key | gitleaks 8.30.1 `github-pat`, `github-oauth`, `github-app-token` (36 alphanumerics), `github-fine-grained-pat` (82). Lengths are lowered on purpose: over-redacting a short look-alike is cheaper than leaking. The `ghr_` prefix is listed in GitHub's authentication docs |
| `bearer_token` | `bearer` or `basic` (any case), whitespace, then at least 8 token characters, **only if** the token contains a digit or one of `+ / = ~`, or is at least 32 characters | The token; the scheme word stays | HTTP `Authorization` header. The extra condition keeps `Basic authentication required` readable |
| `credential` (text) | A name containing `token`, `secret`, `key` or `password`, then `=` or `:` (optionally quoted), then a value that looks random | The value only | Spec §5.8; gitleaks `generic-api-key` thresholds |
| `credential` (JSON) | As above, by object key, in `redact_json` | The value | Same |

## How others do it

Facts below were checked on 2026-10-12 against the sources named.

| Tool | Approach | Pros | Cons | What Atabey takes |
|---|---|---|---|---|
| GitHub Actions runner ([`HostContext.cs`](https://github.com/actions/runner/blob/main/src/Runner.Common/HostContext.cs)) | Masks each registered secret **plus encoded forms**: Base64 (three shifts), command-line, expression, JSON, URI, XML, PowerShell, double-quote-trimmed | Catches a secret after an encoder touched it | A fixed list of encoders; exact values only | Match the secret's escaped forms, not only the plain one. Atabey decodes URL encoding instead of listing it; Base64 is deferred (P-006) |
| gitleaks 8.30.1 ([config](https://github.com/gitleaks/gitleaks/blob/v8.30.1/config/gitleaks.toml)) | One regex per vendor, plus a generic `key/secret/token/password = value` rule with an entropy threshold of 3.5 | Finds secrets nobody registered | Detects, does not redact; false positives | The prefix formats, the 3.5 threshold and the "letters only is not a secret" rule |
| Sentry Relay ([`regexes.rs`](https://github.com/getsentry/relay/blob/master/relay-pii/src/regexes.rs)) | Replaces the value of any field whose **name** matches `password`, `secret`, `api_key`, `token` and similar, with no entropy test | Simple; never misses a named field | Over-redacts (`token_type: "bearer"`) | Field-name matching for structured data, but with the entropy gate |

Codex CLI, Claude Code and Cursor were not compared: their redaction behavior is not documented in a source this task could verify.

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| Push API (chosen) | No dependency from `atabey-defter` on the keychain; trivial to test with a mock; the keychain decides when the set changes | The keychain must remember to push after every change (T-08's `secret_set` and `secret_clear` do, and T-10 wires startup) |
| A `SecretSource` trait the redactor pulls from | Redactor is always current | The redactor would call into the keychain on a hot path, or cache and need invalidation anyway |
| Enumerate percent-encoded variants of each secret | No decoder | Hex case, which characters get escaped and `+` for spaces multiply the variants, and every encoder disagrees |
| Decode and search (chosen) | One decoder covers every encoder and case | About 40 lines of byte mapping that must be tested (done) |
| Non-overlapping Aho-Corasick (`find_iter`) | Faster, simpler | Leaves the tail of an overlapping second secret visible |
| Re-scan the output until stable | Catches anything a first pass exposes | Can redact a marker, and hides span bugs |
| Redactor as an optional argument or a global | Fewer signature changes | Easy to forget; a global hides which writes are protected |
| Entropy rule on any high-entropy token | Finds more | Redacts hashes, UUIDs and IDs everywhere. The name condition keeps it targeted |

## Consequences
- **Easier:** one place to audit (`redact.rs`), two choke points, and tests that run the real ledger and the real app log against a mock keychain with random secrets in many encodings and positions (AC-11).
- **Harder:** every caller of `Ledger::new` and `app_log` passes a redactor; T-08 and T-10 must push the key set.
- **Accepted false positives:** a random-looking value (for example a 40-character hex digest) under a field named `*key*`, `*token*`, `*secret*` or `*password*` is redacted.
- **Known limits (not covered, not claimed):** a secret split across two fields or two log lines; double URL-encoding; JSON escapers that differ from `serde_json` (`\/`, `\uXXXX` for non-ASCII); a JSON-escaped secret inside a `Debug`-formatted field of a JSON log line (three layers); JSON escaping applied on top of a URL encoder that leaves a raw backslash or quote (non-conforming); Base64 forms of a secret (P-006); passwords in URL userinfo and bare JWTs (P-007); secrets in memory after use (P-008); a whitespace-only secret (never matched, to avoid redacting every run of spaces).
- **Reverse:** the pattern rules are one table in one file; switching the known-secret engine touches only `Known`.
