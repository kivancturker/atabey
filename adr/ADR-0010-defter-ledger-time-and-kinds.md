---
id: ADR-0010
title: "Defter ledger: the time crate, UTC day folders, entry kinds and the clock seam"
status: accepted
date: 2026-10-12
decisions: [D-029]
spec: specs/S0-foundations.md R-21, R-22, §5.4, §5.8 (T-05)
---

# ADR-0010: Defter ledger: the time crate, UTC day folders, entry kinds and the clock seam

## Context
T-05 builds the ledger writer (R-21) and the app-log setup (R-22). The spec leaves four things open:

1. **No date-time crate** is in the stack table (§5.1), yet `Timestamp` and the `YYYY-MM-DD` day folders need one.
2. **Which day?** §5.8 names the folders `YYYY-MM-DD` but not the time zone. (§5.9 says the *budget* rolls over at local midnight; that is a money rule, not a log rule.)
3. **`EntryKind` and `Outcome`** are named in §5.4 but not defined.
4. **Testing midnight** needs a controllable clock, and T-07 (retention, 90 days) needs the same one.

## Decision

| Question | Decision |
|---|---|
| Date-time crate | **`time` 0.3.55** (MIT OR Apache-2.0, published 2026-08-01; crates.io, checked 2026-10-12). Features: `formatting`, `parsing`, `serde-well-known`. Already in `Cargo.lock` and used by `tracing-appender`, so it adds no new crate to the build |
| `Timestamp` | `pub type Timestamp = time::OffsetDateTime`, always UTC, serialized as RFC 3339 via `time::serde::rfc3339` (`2026-10-12T09:30:15.25Z`). In TypeScript and JSON Schema it is a `string` |
| Day folders | **UTC**. A ledger entry lands in the folder of its UTC date, and the app log (`tracing-appender` rolls on UTC dates) agrees with it. Retention math in T-07 ("older than 90 days") is then unambiguous and survives time-zone changes and daylight saving. The GUI can show local time; the files do not |
| `EntryKind` | Internally tagged enum (`"type"`), one variant per source R-21 lists, plus `system`: `user_action { action }`, `ipc_command { command }`, `extension_message { topic }`, `llm_call { model }`, `system { event }`. Each variant carries the one identifying name; everything else goes in `data`. New variants are additive |
| `Outcome` | Internally tagged enum (`"status"`): `ok`, `failed { code }` (an `ErrorKind` code), `denied { reason }` (permission or budget refusal), `cancelled` |
| `Actor` | As in §5.4, serialized snake_case and externally tagged: `"user"`, `"core"`, `{"extension":"atabey.defter-viewer"}`, `{"agent":"claude-code"}`, `{"provider":"openrouter"}` |
| Clock seam | `trait Clock { fn now(&self) -> OffsetDateTime }` with `SystemClock` and a test-friendly `ManualClock`. `Ledger::new` takes an `Arc<dyn Clock>`. T-07 reuses it |
| Correlation | `Ledger::begin_flow()` returns a `Flow` holding a fresh `CorrelationId` and a `tracing` span `flow{correlation_id=…}`. `flow.record(draft)` stamps the id on every entry; code running inside the span puts the same id on its app-log lines |
| Session id | New `SessionId` (UUIDv7) in `atabey-types`, used as the ledger file name |
| Write path | One lock, one `write_all` of a complete line per entry on a file opened in append mode. All writes pass through the private `Ledger::commit`, where T-06 inserts the redactor |
| Free-form `data` | `serde_json::Value`. specta cannot export `serde_json::Number` (BigInt rule), so the field is declared TypeScript `unknown` with `#[specta(type = specta_typescript::Unknown)]`; the JSON Schema accepts any value |

## How others do it

| Tool | Format and layout | Time | Lesson for Atabey |
|---|---|---|---|
| Claude Code session transcripts | JSONL, one file per session under `~/.claude/projects/<project>/` | ISO 8601 strings per record | Same shape as ours (one object per line, per-session file). Ours adds a day folder so retention can drop whole folders |
| Codex CLI rollout files | JSONL, `rollout-<timestamp>-<conversation_id>.jsonl` under `~/.codex/sessions/` (Codex source, checked 2026-10-12) | Timestamp in the file name and in records | Confirms per-session JSONL. The date is only in the name, so listing by age means parsing names; our date folders avoid that |
| VS Code log folders | One folder per launch, named `YYYYMMDDTHHMMSS`, holding one `.log` per component | Local time in the folder name | Easy to browse, but local-time names make "older than N days" ambiguous across zones |
| OpenTelemetry log records | Data model, not a file format: `Timestamp` as `uint64` nanoseconds since the Unix epoch, plus `TraceId` (data model, checked 2026-10-12) | Always an absolute instant | Our `correlation` plays the role of a trace id. We keep readable RFC 3339 strings instead of integers because the files are meant to be read and diffed |

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| `time` (chosen) | Already compiled in; small; the crate `tracing-appender` uses, so file names and entries agree; serde support for RFC 3339 | Less ergonomic than `jiff`; no time-zone database (not needed for UTC) |
| `chrono` | Most widely known | A second date-time stack alongside the one `tracing-appender` already pulls in |
| `jiff` | Best API, built-in zone support | New dependency, and the zone support is unused with UTC-only files |
| Local-time day folders | Match the user's wall clock and the budget's "local midnight" | Retention and log-matching become ambiguous across time-zone and DST changes |
| Free-form `kind: String` | Maximally extensible | No exhaustive matching in Rust or TypeScript; typos create new kinds silently |
| Open and close the file on every append | No state between appends | A file open per entry; the cached handle keeps appends in one place and in one lock |
| Ambient correlation id (thread-local or task-local) | Callers pass nothing | Hidden state that breaks across `spawn` and threads; the explicit `Flow` is easy to follow and to test |

## Consequences
- **Easier:** retention (T-07) compares UTC dates; tests cross midnight by moving a `ManualClock`; the wire form is pinned by an `insta` snapshot, so format changes show up in review.
- **Harder:** the GUI converts to local time for display. A person near midnight local time may see "yesterday's" folder hold entries from this morning.
- **Limits:** the ledger keeps the current day's file open, which on Windows blocks deleting that file. Retention only touches folders older than 90 days, so it never meets an open file.
- **Reverse:** switching crate or zone changes only `Timestamp`, `day_folder` and the `Clock` implementations; old files stay readable because RFC 3339 strings carry their offset.
