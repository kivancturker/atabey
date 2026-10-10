---
id: ADR-0004
title: gix instead of git2
status: accepted
date: 2026-10-10
decisions: [D-127]
spec: specs/S0-foundations.md R-7, §5.1
---

# ADR-0004: gix instead of git2

## Context
S0 reads repository facts (branch, HEAD, dirty count); later slices do far more (checkpoint commits in S2, history for Orhun). Constitution Article 10 restricts linked dependencies to permissive licenses.

## Decision
Use **gix 0.89** (pure Rust, MIT OR Apache-2.0), with only the features each task needs.

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| **gix** | Pure Rust (no C toolchain issues on Windows); allowed license; fast | API still pre-1.0 and changes between minor versions |
| git2 (libgit2 bindings) | Mature API | libgit2 is GPL-2.0 with a linking exception, **not on the allowed list**; C build on Windows |
| Shelling out to `git` | Always matches the user's git | Requires git installed; parsing output; slower |

## Consequences
- Pin minor versions and upgrade gix deliberately.
- Write operations (S2 checkpoints) may use the `git` CLI where gix lacks support. That will be its own ADR.
