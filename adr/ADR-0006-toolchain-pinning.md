---
id: ADR-0006
title: "Toolchain pinning: rust-toolchain.toml + mise + pnpm"
status: accepted
date: 2026-10-10
decisions: [D-103, D-016]
spec: specs/S0-foundations.md R-39
---

# ADR-0006: Toolchain pinning: rust-toolchain.toml + mise + pnpm

## Context
The same commands must give the same results on the user's Mac, on Windows and in CI (R-39). Stack packs will use `mise` for user projects (D-103), so Atabey should use it on itself.

## Decision

| Tool | Pinned in | Version (2026-10-10) |
|---|---|---|
| Rust (+ rustfmt, clippy) | `rust-toolchain.toml` (read by rustup everywhere, including CI) | 1.99.0 |
| Node | `mise.toml` | 24.21.0 (Active LTS) |
| pnpm | `mise.toml` + `packageManager` in `package.json` | 12.10.1 |
| cargo-deny, gitleaks, cargo-nextest, cargo-insta | `mise.toml` | 0.20.2, 8.30.1, 0.9.148, 1.49.0 |
| JS packages | `pnpm-lock.yaml` | — |
| Rust crates | `Cargo.lock` + `[workspace.dependencies]` | — |

Everyday gate: **`mise run check`** (the same steps CI runs).

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| **mise + rust-toolchain.toml** | One file per ecosystem convention; cross-platform; prebuilt binaries | One more tool to install (`brew install mise`) |
| Nix / devbox | Fully reproducible | Steep learning curve; weak native Windows support |
| Dev Containers | Isolated | Needs Docker; can't run the desktop GUI natively |
| Documented versions only | Nothing to install | Drift between machines |

## Consequences
- New machine setup: `brew install mise` (or winget on Windows) → `mise install` → `pnpm install`.
- Corepack is not used (removed from Node 25+); pnpm comes from mise.
