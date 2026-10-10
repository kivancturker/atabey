---
session: v0.5
date: 2026-10-10
status: open            # becomes "closed" when the user answers 03-OPEN-QUESTIONS.md and the S0 spec is frozen
title: Brainstorming ends; Product Spec + Constitution frozen; detailed S0 Foundations spec drafted
previous: ../v0.4/SUMMARY.md
---

# Session v0.5: From Brainstorm to Build Plan

## Files in this session

| File | What it is |
|---|---|
| [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) | What was applied from v0.4 · missing Claude plan · **three findings**: iframe security (D-100), OpenCode Go only via OpenCode, missing licenses in Art. 10 · how freezing works |
| [02-S0-TECH-CHOICES.md](02-S0-TECH-CHOICES.md) | Open technical choices compared with other tools: typed IPC, Markdown parsing, frontend stack, extension isolation, first extension, **repo visibility & CI cost**, task workflow |
| [03-OPEN-QUESTIONS.md](03-OPEN-QUESTIONS.md) | Round 5: 12 questions + 2 free-text → **S0 freeze** |
| [04-ACTION-GITHUB-REPO.md](04-ACTION-GITHUB-REPO.md) | 📌 **User action**: create the GitHub repo, push, set a $0 Actions budget |

**New at the repo root** (outside this folder, because they're living project documents):

| File | Status |
|---|---|
| [PRODUCT-SPEC.md](../../../PRODUCT-SPEC.md) | **v1.0.0 frozen** · amendment A-1 proposed |
| [CONSTITUTION.md](../../../CONSTITUTION.md) | **v1.0.0 frozen** · amendment A-1 proposed |
| [specs/S0-foundations.md](../../../specs/S0-foundations.md) | **Draft**: 40 requirements · 22 acceptance criteria · 26 tasks in 6 milestones |
| [ROADMAP.md](../../../ROADMAP.md) · [PARKING-LOT.md](../../../PARKING-LOT.md) | Living |

## What happened

1. You accepted every ⭐⭐ in v0.4 (Claude plan left blank). **Brainstorming is complete** (D-083).
2. `brainstorm/` moved to `docs/brainstorm/` (v0.4-Q10) with `git mv`.
3. Facts checked (2026-10-10): **Tauri 2.12.2** (3.0 is alpha) · **tauri-specta 2.0.0-rc.25** (still RC) vs **ts-rs 12.0.1** · Tauri iframe advisory **CVE-2024-35222** (fixed; Windows same-origin exception) · WebdriverIO's Tauri service supports **macOS** E2E · **serde_yaml deprecated** · `notify` is **CC0-1.0** · `git2`/libgit2 license isn't on the allowed list → **gix** · GitHub Actions: **macOS $0.062/min**, public repos free · React 19.3, TS 7.0, Vite 8.3, Tailwind 4.3, Mermaid 12.1, Shiki 4.5.

## Key changes vs v0.4

| Topic | v0.4 | v0.5 |
|---|---|---|
| Phase | Brainstorming | **Implementation planning** |
| Product Spec / Constitution | Drafts | **Frozen v1.0.0** (+ one proposed amendment each) |
| S0 | One table row | **Full technical spec**: crates, types, IPC surface, rendering, themes, Defter, gateway, extension host, CI, security, 26 tasks |
| Repo | `brainstorm/` at root, local only | `docs/brainstorm/` · root docs · GitHub repo pending |
| D-100 extension UI | Sandboxed iframes | Same, **proven by spike T-04 first** (Tauri advisory) |

## Open items (answered before S0 is frozen)

Typed IPC · Markdown parsing · frontend stack · iframe isolation · first extension · repo visibility & CI · Claude plan · `outputs/` · PR workflow · Product Spec A-1 · Constitution A-1 · S0 freeze.

## Next

Freeze S0 → **T-01** (workspace skeleton, `AGENTS.md`/`CLAUDE.md` for implementation, ADRs) as the first implementation task.
