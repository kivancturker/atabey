---
id: ADR-0008
title: Ignore RUSTSEC-2024-0370 (proc-macro-error) from Tauri's Linux GTK stack
status: accepted
date: 2026-10-11
spec: specs/S0-foundations.md T-02, CONSTITUTION.md Article 10
---

# ADR-0008: Ignore RUSTSEC-2024-0370 (proc-macro-error)

*(ADR-0007 is reserved for the T-04 iframe-isolation spike.)*

## Context
Adding `tauri` 2.12 made `cargo deny check advisories` fail on RUSTSEC-2024-0370: `proc-macro-error` is unmaintained. It is pulled in only by `gtk3-macros` (via `gtk`/`webkit2gtk`/`wry`), i.e. Tauri's **Linux** backend. It is a build-time proc-macro, not a vulnerability, and no safe upgrade exists until Tauri moves off GTK3 (not available in Tauri 2.x; see ADR-0001).

## Decision
Add `RUSTSEC-2024-0370` to `[advisories] ignore` in `deny.toml`, with a comment pointing here. Revisit at every Tauri minor upgrade and remove the ignore when the dependency disappears.

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| Restrict `cargo deny` to macOS and Windows targets | No ignore needed | Hides the Linux build's dependency set from the policy check |
| Fork or patch `gtk3-macros` | Clears the warning | Maintenance burden for a build-time-only macro |
| Ignore with this ADR | Narrow and documented; any other advisory still fails | Must be re-checked on Tauri upgrades |

## Consequences
Only this one advisory is ignored. The macOS and Windows builds do not compile the crate.
