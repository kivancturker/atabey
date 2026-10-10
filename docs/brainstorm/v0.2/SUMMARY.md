---
session: v0.2
date: 2026-10-07
status: closed
title: Answers processed, clarifications, priorities re-ordered, new pillars
previous: ../v0.1/SUMMARY.md
---

# Session v0.2: Answers, Clarifications & Re-prioritization

## Files in this session

| File | What it is |
|---|---|
| [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) | Answers to every ❓ from v0.1, each with "what other tools do" + pros/cons + sources |
| [02-UPDATED-PROPOSAL.md](02-UPDATED-PROPOSAL.md) | Only what changed/new: revised pillars, Codebase Atlas, document format, Brainstorm Studio, learner profile & skill roadmaps, architecture, vertical-slice build order |
| [03-OPEN-QUESTIONS.md](03-OPEN-QUESTIONS.md) | Round 2: 19 questions + 2 free-text (**awaiting the user's answers**) |

## What happened

1. The user answered all 30 v0.1 questions inside [../v0.1/02-OPEN-QUESTIONS.md](../v0.1/02-OPEN-QUESTIONS.md) and added new requests.
2. The brainstorm was restructured into **versioned session folders** (this structure) at the user's request. `CLAUDE.md` holds the standing rules.
3. Facts checked online (2026-10-07):
   - **Anthropic bans Claude subscription OAuth in third-party tools, including the Agent SDK.**
   - **OpenAI "Sign in with ChatGPT" for third-party apps is for selected partners only.**
   - **OpenCode Go ($10/mo) officially allows use "with any agent".**
   - **roadmap.sh content is copyrighted** (personal use OK, no redistribution).
   - **InterSystems Atelier** was an IDE (name overlap).
   - **ACP** is supported by ~50 agents (native: OpenCode, Gemini CLI, Cursor; via adapter: Claude Code, Codex).

## Key changes vs v0.1

| Topic | v0.1 | v0.2 |
|---|---|---|
| #1 purpose | Spec & Roadmap MVP | **Codebase understanding (Codebase Atlas)**; spec-driven development is #2 |
| Build order | CLI → VS Code ext → Tauri | **Vertical slices**, Tauri from day one; CLI is a debug tool only |
| Engine language | TypeScript | **Rust core + TS/React GUI** *(tentative, v0.2-Q2)* |
| Agent connection | SDK/ACP + MCP + PTY fallback | Same, plus **Native mode / Terminal mode** and the **ban-safe rule** (official binaries for subscriptions) |
| Storage | `.harness/` files | **`.atelier/`** files + SQLite derived index + **`~/.atelier/`** user level *(tentative, v0.2-Q8)* |
| Observers | Advisory, blocking high/critical | Same + **structurally read-only** (no write tools; the engine writes findings) |
| Feedback overlay | Always-on-top | **On demand** (hotkey); defect → fix queue, idea → Parking Lot |
| New pillars | — | **Brainstorm Studio**, **cross-project learner profile + skill roadmaps**, **rich document format with versioning** |

## Open items (answered in v0.3)

Name codename vs final · engine language · GUI framework · build order · explaining existing repos · eager vs lazy explanations · document format · storage · first agent · default session mode · defect/idea routing · overlay trigger · budget caps · brainstorm structure · learner profile location · skill tree source · progress counting · Windows timing · license.
