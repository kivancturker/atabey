---
session: v0.4
date: 2026-10-09
status: open            # becomes "closed" when the user answers 06-OPEN-QUESTIONS.md and v0.5 starts
title: Consolidation, Claude Code first, native-first v1 over ACP, Ulak messaging, Product Spec + Constitution drafts
previous: ../v0.3/SUMMARY.md
---

# Session v0.4: Consolidation & Native-First

## Files in this session

| File | What it is |
|---|---|
| [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) | Table of all v0.3 decisions · multiple themes · Claude Code first · observer runtime · extension messaging · **Claude Code via ACP: licensing & terms** · roadmap.sh cancelled · **skip v1? (my opinion: yes, with a terminal fallback)** |
| [02-PROPOSAL-AGENTS-AND-SESSIONS.md](02-PROPOSAL-AGENTS-AND-SESSIONS.md) | 4 ways to connect Claude Code · native-first rationale + risks · ACP session architecture with the Jev permission gate · OpenCode headless observers · **revised build order** |
| [03-PROPOSAL-ULAK-MESSAGING.md](03-PROPOSAL-ULAK-MESSAGING.md) | **Ulak · Extension Messaging**: commands, events, typed services through the core broker; contracts; real flow |
| [04-PRODUCT-SPEC-DRAFT.md](04-PRODUCT-SPEC-DRAFT.md) | Consolidated **Product Spec** draft (vision → glossary) |
| [05-CONSTITUTION-DRAFT.md](05-CONSTITUTION-DRAFT.md) | **Constitution** draft: 12 articles |
| [06-OPEN-QUESTIONS.md](06-OPEN-QUESTIONS.md) | Round 4 (last): 10 questions + 2 free-text (**awaiting your answers**) |
| [07-ACTION-GIT-INIT.md](07-ACTION-GIT-INIT.md) | 📌 **User action**: `git init` (blocked by session permissions) |

## What happened

1. You answered v0.3: 24 of 25 questions followed the ⭐⭐/⭐ recommendation (Q4 blank → ⭐⭐; Q11 = ⭐ option D). Custom answer on Q19 (multiple themes), and five new ideas in Q27.
2. Facts checked (2026-10-09):
   - **Anthropic's legal page:** Pro/Max limits "assume ordinary, individual usage of Claude Code and the Agent SDK". Third-party developers may not offer Claude.ai login or touch credentials. Running the unmodified binary with the user's own sign-in is allowed.
   - **Agent SDK** "runs the Claude Code binary". The ACP adapter `claude-agent-acp` is Apache-2.0 and built on it.
   - **Billing:** the June 15, 2026 plan to move ACP / `claude -p` / SDK usage into a separate credit was **paused** (Zed, June 16 update).
   - **ACP** has a registry (Claude Code, Codex, OpenCode, Gemini CLI, Copilot CLI…) and an official Rust crate.
   - **OpenCode:** `opencode acp` supports almost everything except `/undo`/`/redo`. Agents can deny edit/bash/webfetch. Go is "designed for OpenCode and other coding agents", with client rules and abuse monitoring.
3. `git init` was approved by you but **blocked by session permissions**, so it became an action item.

## Key changes vs v0.3

| Topic | v0.3 | v0.4 |
|---|---|---|
| Name | Working name Atabey | **Atabey decided** (legal check before going public) |
| Main coding agent | OpenCode | **Claude Code** (your subscription) via **ACP**; OpenCode second |
| v1 session mode | Terminal + GUI (OpenCode serve/attach) | 🟡 **Native GUI over ACP + Terminal fallback, both in S2** |
| v2 | Native mode | More doc types, CD, public Talimhane, more agents |
| Observers | OpenCode Go models (runtime unspecified) | 🟡 **OpenCode headless, read-only agents, separate worktree** |
| Extension communication | — | 🟡 **Ulak**: commands + events + typed services via the core broker |
| Judge | Choice open | Go's strongest model + **escalate-to-Opus 5.5 button** |
| Skill trees | AI + roadmap.sh links (permission email) | **AI-only; roadmap.sh cancelled** |
| Themes | One dark theme | **Theme system from S0**, Monokai ships, more later |
| Consolidation | Planned | **Product Spec + Constitution drafts written** |

## Open items (answered in v0.5)

Native-first v1 · Claude connection · second agent · observer runtime · Claude plan & meter · public sign-in policy · Ulak · Product Spec review · Constitution review · `docs/brainstorm/` move · last ideas.

## Next

**v0.5 = the very detailed S0 Foundations technical spec & roadmap** (crates, modules, types, IPC, renderer, theme system, Defter, extension host, Ulak v0, CI, tasks) + `PRODUCT-SPEC.md` v1.0 + `CONSTITUTION.md` v1.0 frozen.
