---
session: v0.1
date: 2026-10-07
status: closed
title: Initial idea, first proposal, first question round
next: ../v0.2/SUMMARY.md
---

# Session v0.1: Initial Idea & First Proposal

> **Read this instead of the full files** unless you need detail. This folder is a frozen historical record.

## Files in this session

| File | What it is |
|---|---|
| [00-ORIGINAL-IDEA.md](00-ORIGINAL-IDEA.md) | The user's original idea, verbatim |
| [01-PROJECT-UNDERSTANDING.md](01-PROJECT-UNDERSTANDING.md) | Claude's understanding, assessment, architecture proposal, answers to the 4 technical questions |
| [02-OPEN-QUESTIONS.md](02-OPEN-QUESTIONS.md) | 30 multiple-choice questions, **with the user's answers** (filled in at the start of v0.2) |

## What happened

1. The user pasted the idea: an IDE + multi-agent harness + mentorship platform.
2. Claude restated it as **5 pillars**: P1 Shell/UI, P2 Spec & Roadmap, P3 Living Docs, P4 Observer Swarm, P5 Mentorship.
3. Claude proposed:
   - a **headless engine** with GUIs as clients
   - a **Session Ledger** (event log) as the shared backbone
   - a **`.harness/` folder** in the repo for all knowledge
   - **Git checkpoint commits** per agent turn
4. Claude answered the 4 technical questions:
   - **GUI→TUI input:** use structured protocols (SDK/ACP) plus MCP tools like `ask_user`. Typing into the terminal (PTY injection) is only a fallback.
   - **Foundation:** don't fork VS Code. Recommended engine + CLI first, then a VS Code extension, then a Tauri app.
   - **Consensus loop:** a fixed pipeline (independent review → one cross-examination pass → tests or a judge settle disagreements), with fingerprints and budget caps so it can't loop.
   - **Screenshot:** hide the overlay → capture → show it again → crop a frozen frame. No native code.
5. Claude pushed back on four points: scope (five products in one), "more models is better", "perfectly finalized specs", and building a custom agent first.

## Key outcomes (answers given at the start of v0.2)

See [../DECISIONS.md](../DECISIONS.md) for the authoritative list. In short:

- Name **Atelier** (if it's legally safe). Solo dev, personal tool, open to open-sourcing.
- **#1 purpose changed:** *understanding code without reading every line.* Spec-driven development is #2. The GUI is how everything is delivered.
- Wrap existing agents. Tauri app directly. Cross-platform (macOS first, Windows too).
- Specs are frozen and changed only through amendments. One spec per feature. A Parking Lot holds new ideas.
- Swarm: about 3 models from different vendors, run per completed task. High/critical findings block acceptance. **Observers are read-only.**
- Budget: **under $3/day**, ideally less. OpenCode Go is the preferred provider.
- v1 docs: ADRs, code map, high-level feature docs.
- Debrief mode, notebook with spaced repetition (no Anki export), AI feedback stored.
- Feedback overlay **on demand only**, for testing the app being built.
- **New requests:** a Brainstorm tool, a cross-project learning profile with skill roadmaps, and rich GUI rendering of everything (no plain/ASCII text).

## Superseded ideas from this session

| v0.1 idea | Replaced by | Where |
|---|---|---|
| CLI first → VS Code extension → Tauri later | Vertical slices: engine and Tauri GUI built together | v0.2 |
| P2 Spec is the MVP pillar | Code understanding (Codebase Atlas) is the first slice | v0.2 |
| TypeScript engine | Rust core + TypeScript UI (tentative, awaiting confirmation) | v0.2 |
