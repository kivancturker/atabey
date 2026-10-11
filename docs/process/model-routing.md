---
doc: process
title: Model routing for implementation tasks
date: 2026-10-11
status: active
---

# Model Routing: Which Model Does Which Task

> **Why:** the user is on **Claude Pro**, the plan with the tightest usage limits. Session and weekly limits are **shared across all models**, and the bigger models use them up faster. Matching the model to the task stretches the plan.
> **Who decides:** each task's tier is set here when the slice spec is frozen. A task can be **escalated** (M → H) if it turns out harder than expected; it's never silently downgraded mid-task.

## 1. The short version

| Tier | When | Who does it | How to run it |
|---|---|---|---|
| **H · Hard** | Sets patterns other tasks copy · security · concurrency · poorly documented library integration | **Opus plans, Sonnet implements, Opus reviews** | Session in **plan mode** with model `opusplan` → approve plan → Sonnet executes → `reviewer` subagent (Opus) before the PR |
| **M · Standard** | Well-specified feature work following existing patterns | **Sonnet** | Normal session (project default `opusplan` runs Sonnet outside plan mode), or the `implementer` subagent |
| **L · Mechanical** | Docs tables, fixtures from a list, renames, boilerplate | **Haiku** | `mechanical` subagent (or OpenCode on the Go plan) |

**Overflow rule:** when the Claude Pro window is used up, Tier M/L work can continue in **OpenCode on the OpenCode Go plan** (it reads the same `AGENTS.md`). Tier H waits for the window to reset.

## 2. Price check (API list prices, per million tokens, 2026-10-06)

| Model | Input | Output | Relative to Opus | Position |
|---|---|---|---|---|
| Opus 5.5 | $4.00 | $20.00 | 1× | Complex reasoning, architecture |
| Sonnet 5.5 | $2.00 | $10.00 | ½× | Everyday coding and agents |
| Haiku 5.5 | $0.10 | $0.50 | 1/40× | Simple, fast tasks |
| Fable 5.1 | $10.00 | $50.00 | 2.5× | **Not used:** on Pro it can bill to usage credits instead of plan limits |

Anthropic doesn't publish how each model draws on Pro's limits; API prices are the best available proxy. The Claude Code docs say: *"Sonnet handles most coding tasks well and costs less than Opus. Reserve Opus for complex architectural decisions or multi-step reasoning."*

## 3. The bigger lever: small, fresh sessions

Model choice is only half of it. **Every turn re-sends the whole conversation.** A session that's been open all day costs more per message than a fresh one, whatever the model.

| Habit | Why |
|---|---|
| **One task = one fresh session** (`/clear` or a new session per T-nn) | Context starts small; the spec + constitution + task are all an agent needs |
| Plan in plan mode, then execute | A wrong direction caught in the plan costs one turn, not a rewrite |
| Hand verbose work (test runs, log reading) to subagents | Only a summary comes back into the main context |
| Keep `CLAUDE.md` + `AGENTS.md` short | They're loaded into every session |
| Check `/usage` | Shows plan usage bars and which subagents/skills used the most |

## 4. What others do

| Tool | Approach | Pros | Cons |
|---|---|---|---|
| **Aider** | *Architect mode*: a strong model designs the change, a cheaper "editor" model writes the edits | The original precedent for this split | Per-request, not per-task |
| **Claude Code** | `opusplan` (Opus in plan mode, Sonnet for execution) + per-subagent `model:` | Built in; no tooling needed | Choosing the tier is up to us |
| **Cline** | Separate models for *Plan* and *Act* modes | Same idea as `opusplan` | — |
| **Cursor** | "Auto" picks a model per request | Zero effort | Opaque; no per-task control |
| **OpenCode** | A model per agent in config | Flexible | Same: choosing is up to us |
| **Atabey (this policy)** | Tier per **task**, decided at spec freeze, escalate-only | Predictable, reviewable, matches Spec-Lite | Needs a judgment call per task |

## 5. S0 task tiers

| Task | Tier | Reason |
|---|---|---|
| T-01 Workspace skeleton | — | Done (Opus, before this policy) |
| T-02 Tauri shell + CI | M | Well documented; follows official templates |
| T-03 Types + codegen (tauri-specta RC) | **H** | Sets the IPC pattern every later task copies; RC library |
| T-04 Iframe isolation spike | **H** | Security boundary (CVE-2024-35222) |
| T-05 Defter ledger | M | Straightforward file writing |
| T-06 Redactor + proptest | **H** | Security: a gap leaks secrets |
| T-07 Retention job | M | Small; clock injection |
| T-08 Settings + keychain | M | Documented crates |
| T-09 Project model (gix, watcher, index) | M | Large but standard; escalate if gix fights back |
| T-10 Composition root + CLI | M | Wiring |
| T-11 Markdown → AST (comrak) | **H** | Core data model; many edge cases |
| T-12 Typed blocks | M | Follows T-11 patterns |
| T-13 Question cards + golden corpus | M | Pattern detection + snapshots |
| T-14 Theme system | M | Data + CSS variables (the Monokai JSON can go to `mechanical`) |
| T-15 Layout + palette | M | Standard React |
| T-16 Renderer components | M | Standard React; many components |
| T-17 File tree + code view | M | Standard |
| T-18 LLM gateway | M | HTTP client + price table |
| T-19 Budget meter (pause/resume) | **H** | A hard Constitution limit; async pause flow |
| T-20 JSON-RPC framing | M | Small, well-known protocol |
| T-21 Ulak broker | **H** | Concurrency, permissions, schema validation |
| T-22 Extension host + supervisor | **H** | Process supervision + security boundary |
| T-23 Defter Viewer extension | M | Follows T-22 |
| T-24 E2E suite | M | Test writing |
| T-25 Release / CSP / capability audit | **H** | Security review |
| T-26 Slice review + Parking Lot triage | L | Docs, with the user |

**Totals:** 8 H · 16 M · 1 L. Opus is used only for **planning and review** on the 8 H tasks; Sonnet writes all the code.

## 6. Running a task (checklist)

1. Start a **fresh session** in this repo (the project default model is `opusplan`).
2. Create the branch: `s0/t-nn-short-name` (from `main`, or from the previous task's branch until it's merged).
3. **Tier H:** switch to plan mode (Shift+Tab) → "Plan T-nn from specs/S0-foundations.md" → review and approve → execution runs on Sonnet → ask for the `reviewer` subagent → fix the findings.
   **Tier M:** "Implement T-nn from specs/S0-foundations.md" (runs on Sonnet).
   **Tier L:** "Use the mechanical subagent to …".
4. `mise run check` green → commit `T-nn: …` → PR (once the GitHub repo exists).
