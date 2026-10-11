---
id: ADR-0002
title: Typed IPC with tauri-specta (pinned RC)
status: accepted
date: 2026-10-10
decisions: [D-014, D-121]
spec: specs/S0-foundations.md R-4, §5.5
---

# ADR-0002: Typed IPC with tauri-specta (pinned RC)

## Context
Constitution Article 1 forbids hand-written duplicates of Rust types in TypeScript. The GUI calls Rust commands and listens to Rust events, so both need generated, typed wrappers.

## Decision
Use **tauri-specta `=2.0.0-rc.25`** (exact pin) to generate TS types, typed command wrappers and typed events into `packages/bindings/`. The generated code is committed, and CI fails on drift (`pnpm gen` + `git diff --exit-code`).

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| **tauri-specta (RC)** | Commands + events + types end to end | Still a release candidate (since 2024) |
| ts-rs 12 (stable) | Stable, very popular | Types only; we'd generate command/event wrappers ourselves in `atabey-gen` |
| TauRPC | Elegant trait-based API | Smaller community |

## Consequences
- An RC can break between versions: upgrade deliberately, one PR, regenerate bindings.
- **Fallback:** switch to ts-rs + an `atabey-gen` wrapper generator. Generated code is isolated in `packages/bindings`, so the GUI's imports don't change shape.
