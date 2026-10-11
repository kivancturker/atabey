# Architecture Decision Records

> **Why something is built the way it is.** One file per decision, never deleted. A changed decision gets a new ADR that supersedes the old one (D-050). Product-level decisions live in the [decision ledger](../docs/brainstorm/DECISIONS.md); ADRs record the *technical* ones made while specifying or building slices.

| ADR | Title | Status | Date |
|---|---|---|---|
| [0001](ADR-0001-tauri-2-not-3.md) | Tauri 2.12, not the 3.0 alpha | Accepted | 2026-10-10 |
| [0002](ADR-0002-tauri-specta-typed-ipc.md) | Typed IPC with tauri-specta (pinned RC) | Accepted | 2026-10-10 |
| [0003](ADR-0003-markdown-parsed-in-rust.md) | Markdown is parsed in Rust (comrak) | Accepted | 2026-10-10 |
| [0004](ADR-0004-gix-not-git2.md) | gix instead of git2 | Accepted | 2026-10-10 |
| [0005](ADR-0005-frontend-stack.md) | Frontend stack | Accepted | 2026-10-10 |
| [0006](ADR-0006-toolchain-pinning.md) | Toolchain pinning: rust-toolchain.toml + mise + pnpm | Accepted | 2026-10-10 |
| [0007](ADR-0007-extension-iframe-isolation.md) | Extension UI isolation: sandboxed iframes on atabey-ext:// | Accepted | 2026-10-12 |
| [0008](ADR-0008-ignore-proc-macro-error-advisory.md) | Ignore RUSTSEC-2024-0370 (proc-macro-error) from Tauri's Linux GTK stack | Accepted | 2026-10-11 |
| [0009](ADR-0009-ipc-bindings-export-and-error-audit.md) | IPC bindings: export path, error shape and error-type audit | Accepted | 2026-10-11 |
| [0010](ADR-0010-defter-ledger-time-and-kinds.md) | Defter ledger: the time crate, UTC day folders, entry kinds and the clock seam | Accepted | 2026-10-12 |
| [0011](ADR-0011-redaction.md) | Redaction: known-secret matcher, pattern rules and the two write choke points | Proposed | 2026-10-12 |

## Template

```markdown
---
id: ADR-NNNN
title: …
status: proposed | accepted | superseded by ADR-NNNN
date: YYYY-MM-DD
decisions: [D-…]          # ledger ids, if any
spec: specs/S…            # section that relies on it
---

# ADR-NNNN: Title

## Context
## Decision
## Alternatives considered   (table: option · pros · cons)
## Consequences              (what gets easier, what gets harder, how we'd reverse it)
```
