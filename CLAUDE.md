# Atabey: Standing Instructions for Claude

@AGENTS.md

## Phase

**Implementation of S0** (since 2026-10-10). Brainstorming is complete. Its history is in `docs/brainstorm/` and the decision ledger is `docs/brainstorm/DECISIONS.md`. The name **Atabey** is decided (D-006); a trademark check is due before going public.

## Start of every session

1. `docs/brainstorm/README.md`: status and **user action items**
2. `docs/brainstorm/DECISIONS.md`: the authoritative decision ledger
3. `ROADMAP.md` and the active slice spec (`specs/`): which task is next
4. `git status` / current branch: is a task already in progress?

## Model routing (Claude Pro limits)

Follow [`docs/process/model-routing.md`](docs/process/model-routing.md): **Tier H** = plan in plan mode (`opusplan`, the project default) → Sonnet executes → a short **Opus peek** at the diff (plan mode again; read-only, no test runs, no exploring) before the PR; **Tier M** = Sonnet (`implementer` subagent or the main session outside plan mode); **Tier L** = `mechanical` subagent (Haiku). One fresh session per task. **The agent creates the task branch itself** (the user doesn't run git commands). Never use Fable (it can bill to usage credits on Pro).

## Design rounds (question files)

The brainstorm Q&A format continues for **slice spec reviews** (e.g., the S1 spec). Each round is a new versioned folder `docs/brainstorm/vX.Y/` with `SUMMARY.md`, `01-CLARIFICATIONS.md` (answer every `❓ Clarify` and react to every custom answer), numbered proposal files, an open-questions file (IDs prefixed with the session, e.g., `v0.6-Q3`) and `NN-ACTION-*.md` for things the user must do.

- **Closed session folders are frozen history.** The only edits allowed are the user writing answers into an open-questions file, and setting a closing session's `SUMMARY.md` status to `closed`.
- At the end of a round: update `docs/brainstorm/DECISIONS.md` (IDs permanent; superseded rows ⛔ with a pointer), update `docs/brainstorm/README.md`, and close the previous session.
- **Question format:** context, a **🔎 What others do** line, checkbox options marked ⭐⭐ / ⭐ / ⚪ / ⚠️, then `✍️ Custom answer:` and `❓ Clarify:`. A blank answer means the ⭐⭐/⭐ recommendation is used.

## How to write for this user

- **Always compare with how popular harnesses, IDEs and tools handle each topic**, with pros and cons: Claude Code, Codex CLI, OpenCode, Gemini CLI, Cursor, Kiro, Zed, GitHub Spec Kit, Cline, VS Code, Neovim, etc.
- **Rich formatting only:** tables, Mermaid diagrams, clear headings. **Never ASCII-art diagrams** (no box-drawing file trees either; use tables).
- Explain concepts simply. Give a short plain-language answer first, then the detail. The user is learning harness internals.
- Use **production-grade, real-world examples**, never toy ones.
- **Verify time-sensitive facts** (pricing, terms of service, protocol support, licenses, product names, **library versions**) before stating or pinning them. Date them and cite sources.
- **Budget:** about $3/day for AI. The user is on **Claude Pro** (the tightest limits) + **OpenCode Go** ($10/mo). Flag costs. Keep tasks small enough for one session.
- Agent roles (D-113): **Claude Code** is the main coding agent; OpenCode Go models run the observers, judge and apprentice, and only ever through the OpenCode runtime (Product Spec A-1). Check designs against the ban-safe rule (D-041).

## Don't

- **Never send emails, messages or posts on the user's behalf**, and never create or push to public repositories without the user's explicit OK. Draft instructions in an `ACTION` file instead.
- Don't use `.pi/` or `outputs/` as sources.
- Only implement the current task. New ideas go to `PARKING-LOT.md`.
- Never edit a frozen document directly (`PRODUCT-SPEC.md`, `CONSTITUTION.md`, frozen specs). Propose an amendment row and ask.
- Commit only on **task branches** (D-126) or when the user asks; never directly to `main`. Other repo-structure changes need the user's OK.
