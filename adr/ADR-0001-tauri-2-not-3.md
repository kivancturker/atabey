---
id: ADR-0001
title: Tauri 2.12, not the 3.0 alpha
status: accepted
date: 2026-10-10
decisions: [D-011, D-127]
spec: specs/S0-foundations.md §5.1, §5.3
---

# ADR-0001: Tauri 2.12, not the 3.0 alpha

## Context
Atabey is a Tauri desktop app (D-011). On 2026-10-10 the latest stable release was **tauri 2.12.2** (2026-10-09) and the newest pre-release was **3.0.0-alpha.4** (2026-10-01).

## Decision
Use **Tauri 2.12.x** (`tauri = "2.12"`). Only `apps/desktop/src-tauri` depends on Tauri; every other crate is plain Rust.

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| **Tauri 2.12** | Stable, documented, plugin ecosystem matches | A 2 → 3 migration later |
| Tauri 3.0 alpha | No migration later | Breaking changes between alphas; plugins lag; few examples |

## Consequences
- The migration to 3.x touches one crate (`src-tauri`) thanks to the isolation rule.
- Revisit when 3.0 is stable *and* the plugins we use (shell, store/dialog, WebDriver test plugin) support it.
