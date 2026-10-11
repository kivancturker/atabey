---
id: ADR-0009
title: "IPC bindings: export path, error shape and error-type audit"
status: accepted
date: 2026-10-11
decisions: [D-014]
spec: specs/S0-foundations.md R-4, R-5, AC-3, §5.4, amendment A-2
---

# ADR-0009: IPC bindings: export path, error shape and error-type audit

*(Builds on ADR-0002, which chose tauri-specta.)*

## Context
T-03 sets the pattern every later IPC task copies. Four questions came up that the spec doesn't answer:

1. tauri-specta's builder needs a Tauri runtime type, but only `apps/desktop/src-tauri` may depend on Tauri (§5.2). Who exports the bindings?
2. R-5 says an error carries "code + message + details", but the §5.4 sketch had no `message`.
3. AC-3 needs a test that every command's error type is `AtabeyError`. tauri-specta keeps its collected command signatures private.
4. specta `=2.0.0-rc.25` pulls in `paste`, which RustSec marks unmaintained (RUSTSEC-2024-0436).

## Decision

| Question | Decision |
|---|---|
| Export path | `src-tauri` owns `ipc::specta_builder()`, the single list of commands, events and extra types. An **example binary** (`cargo run -p atabey-desktop --example export_bindings`) writes `packages/bindings/src/index.ts`. `atabey-gen` stays Tauri-free and writes the JSON Schemas in `schemas/`. `pnpm gen` runs both |
| Error shape | `AtabeyError { #[serde(flatten)] kind: ErrorKind, message: String }`. `ErrorKind` is tagged by `code`; `message` is its `Display`, written once in Rust. Wire form: `{ "code": "not_found", "what": "README.md", "message": "Not found: README.md" }`. Same idea as `std::io::Error` (kind + message) and JSON-RPC errors (code + message + data). Amendment A-2 |
| Error audit | `ErrorTypeAudit` implements tauri-specta's public `LanguageExt` trait: an "exporter" that writes nothing and inspects each command's result type. A command must return `Result<_, AtabeyError>` (`IpcResult<T>`); a negative test proves the audit flags a `String` error and a non-`Result` return |
| Generated-code drift | Checked twice: Rust tests (`bindings_are_up_to_date`, `schemas_are_up_to_date`) fail on every OS, and `mise run drift` (`pnpm gen` + `git diff` + no untracked output) runs in CI's lint job |
| Error handling mode | `ErrorHandlingMode::Result`: generated wrappers return `{ status: "ok", data } \| { status: "error", error: AtabeyError }`, so the GUI can't forget the error case |
| Money | `rust_decimal::Decimal`, serialized as a string (TS `string`), never `f64` |
| `paste` advisory | Ignored in `deny.toml` with a link here. It's a build-time proc-macro required by specta's `function` feature, which tauri-specta needs; unmaintained, not vulnerable |

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| Export from `atabey-gen` | One generator | `atabey-gen` would depend on Tauri, breaking the §5.2 rule |
| Export at app startup in debug builds (tauri-specta's README) | Zero setup | CI would have to launch the app to check drift |
| A test that writes the bindings file | Simple | Tests with side effects on committed files |
| Error shape as in the §5.4 sketch (no `message`) | No amendment | The GUI would rebuild every message in TS, a second copy of the text that can drift |
| Wrapper type `IpcError { message, error: AtabeyError }` | Keeps the enum named `AtabeyError` | Two error types to learn; AC-3's wording no longer fits |
| Audit by parsing the generated TS | No trait trick | Fragile string matching |

## Consequences
- Every new command: `#[tauri::command] #[specta::specta]`, return `IpcResult<T>`, add it to `collect_commands!` in `ipc.rs`, run `pnpm gen`.
- `ErrorKind` fields must never be named `code` or `message` (flatten collisions). `Provider` uses `detail` and `Internal` uses `summary`.
- Generated TS refuses 64-bit integers (`u64`, `i64`); use `u32` or a string. `Timeout.after_ms` is `u32` (49 days is enough).
- The `paste` ignore and the specta pins are re-checked whenever tauri-specta is upgraded. The ts-rs fallback from ADR-0002 still applies.
