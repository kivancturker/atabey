---
session: v0.3
date: 2026-10-07
status: closed
title: Round-2 answers, new name, extensions, quality pipeline, decision models, Talimhane, build process
previous: ../v0.2/SUMMARY.md
---

# Session v0.3: Name, Platform, Quality, Learning & Process

## Files in this session

| File | What it is |
|---|---|
| [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) | Name options (Ottoman/Seljuk/Göktürk), v1/v2 session modes, adjustable caps, session folders vs topic threads, git sync effort, skill-assessment design ("Not yet" 🌱) |
| [02-PROPOSAL-PLATFORM.md](02-PROPOSAL-PLATFORM.md) | Core + extensions ("like Neovim"), Kervan stack packs, **decision models (Jev)** wrapper + use cases, code intelligence (tree-sitter + LSP + SCIP cache), Defter logging, themes |
| [03-PROPOSAL-QUALITY.md](03-PROPOSAL-QUALITY.md) | Feature-completion flow (swarm after green CI), change narratives (diff + command ❓), Pipeline Recipes CI, static analysis → SARIF, Kurultay judge tiers |
| [04-PROPOSAL-LEARNING.md](04-PROPOSAL-LEARNING.md) | Production-grade teaching standard, **Talimhane** (system-thinking practice by directing a weak agent), sharing scenarios |
| [05-HOW-WE-BUILD-IT.md](05-HOW-WE-BUILD-IT.md) | **Spec-Lite** process for building the app solo, revised S0–S9 build order, v1/v2 split, when brainstorming ends |
| [06-OPEN-QUESTIONS.md](06-OPEN-QUESTIONS.md) | Round 3: 25 questions + 2 free-text (answered 2026-10-09) |
| [07-ACTION-ROADMAPSH-EMAIL.md](07-ACTION-ROADMAPSH-EMAIL.md) | 📌 **User action**: guide + draft email asking roadmap.sh for permission (Claude doesn't send it) |

## What happened

1. The user answered v0.2 (19 questions) and added ~14 new ideas in v0.2-Q21.
2. Facts checked (2026-10-07):
   - **Jev** = TypeSafe's decision model (Sept 2026), available via OpenRouter, $0.042 per million input tokens. The OpenAI Decisions API is in limited preview. Open alternatives: Laya, Kev. The **JevOut** paper shows natural context can flip decisions.
   - **OpenCode** supports `serve` + `attach` + an SSE event stream, which enables v1 Terminal + GUI on one session.
   - **SonarQube's analyzers** are source-available since Nov 2024; **Opengrep** is the LGPL fork of Semgrep CE.
   - **Dagger** = CI as an SDK.
   - **lala.ai** exists, so "Lala" collides; no collision found for **Atabey**.
   - roadmap.sh contact found on its About page.

## Key changes vs v0.2

| Topic | v0.2 | v0.3 |
|---|---|---|
| Name | Atelier (codename) | **Dropped.** Working name **Atabey** (Seljuk mentor-guardian); themed module names proposed |
| Session modes | Native default + terminal fallback | **v1 = Terminal + GUI** (OpenCode serve/attach + events); **v2 = Native** + switch-back button |
| Swarm timing | Per completed task | **Per completed feature, after CI is green** |
| Architecture | Monolithic core + later plugins | **Core infrastructure + first-party extensions on a public API** (VS Code/Obsidian style), Bedesten library on GitHub |
| New pillars/features | — | Decision-model wrapper (Jev) · change narratives · Pipeline Recipes CI · SARIF static analysis · Kervan stack packs · Defter logging · Talimhane · production-grade teaching · dark Monokai theme |
| Build order | S0–S6 | **S0–S9 = v1.0**, Native mode = v2.0 |
| License | — | **Apache-2.0** |

## Open items (answered in v0.4)

Name · themed modules · extension model, runtime and registry · code-intel layers · decision-model defaults · log retention · CD scope · CI approach · judge model · analyzer set · narrative timing · Talimhane timing, model and sharing · first stack pack · tool installer · theme · Spec-Lite · brainstorm tags · build order · git sync · end of brainstorming · git init this repo.
