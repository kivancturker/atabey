# Atabey — Standing Instructions

This repo is in the **brainstorming phase** for **Atabey** *(name decided in v0.4, D-006; a trademark check is still due before going public)*. Atabey is a Tauri desktop app (Rust core + React/TypeScript GUI) that explains any codebase in a rich GUI (purpose #1) and builds features through spec-driven, multi-agent development (purpose #2), with mentorship and practice. There is no application code yet.

## Start of every session (do this first)

Read, in order:

1. `brainstorm/README.md`: current session, status, user action items
2. `brainstorm/DECISIONS.md`: the authoritative decision ledger
3. The latest `brainstorm/vX.Y/SUMMARY.md`
4. The latest session's open-questions file. The user may have written answers into it.

Open older session files only when you need detail. Summaries exist so the full history doesn't have to be re-read.

## Brainstorm workflow (user's standing instruction, 2026-10-07)

- **Every brainstorming session gets its own versioned folder:** `brainstorm/v0.1/`, `brainstorm/v0.2/`, … Increment the minor version per session.
- **Closed session folders are frozen history.** Don't edit them, except that the user writes answers into a session's open-questions file, and a closed session's `SUMMARY.md` status is set to `closed`.
- When the user says the answers are ready (or asks to continue), read **all** their answers thoroughly (checkboxes, custom answers, clarify requests, and free-text ideas), then create the next version folder containing:
  - `SUMMARY.md` (frontmatter: session, date, status, previous; plus key changes and open items)
  - `01-CLARIFICATIONS.md`: answer every `❓ Clarify` and react to every custom answer
  - One or more numbered proposal files (`02-…`, `03-…`): only what changed or is new; link back for the rest. Split by area when there's a lot
  - An open-questions file for the next round (question IDs prefixed with the session, e.g., `v0.3-Q5`)
  - `NN-ACTION-*.md` files for things **the user** must do themselves
- At the end of every session: **update `brainstorm/DECISIONS.md`** (IDs are permanent: never renumber or delete; mark superseded rows ⛔ and point to the new ID), **update `brainstorm/README.md`** (session table, status, action items), and mark the previous session's `SUMMARY.md` `status: closed`.
- Save the user's own pasted ideas verbatim when they provide new source material (as in `v0.1/00-ORIGINAL-IDEA.md`).

## Question file format

Every question has: context, a **🔎 What others do** line, checkbox options marked ⭐⭐ (strongly recommended) / ⭐ (recommended) / ⚪ (viable) / ⚠️ (not recommended), then `✍️ Custom answer:` and `❓ Clarify:` lines. A blank answer means the ⭐ recommendation is used. The user says "answers ready" when done.

## How to write for this user

- **Always compare with how popular harnesses, IDEs and tools handle each topic**, with pros and cons: Claude Code, Codex CLI, OpenCode, Gemini CLI, Cursor, Kiro, Zed, GitHub Spec Kit, Cline, VS Code, Neovim, etc.
- **Rich formatting only:** tables, Mermaid diagrams, clear headings. **Never ASCII-art diagrams.**
- Explain concepts simply. Give a short plain-language answer first, then the detail.
- Use **production-grade, real-world examples**, never toy ones (Animal/Dog/Cat, Foo/Bar).
- **Verify time-sensitive facts** (pricing, terms of service and ban rules, protocol support, licenses, product names) with a web search before stating them. Date them and cite sources in the doc.
- Respect the budget: the user can't spend more than about $3/day on AI and prefers OpenCode Go ($10/mo). Flag costs in proposals.
- Agent roles (D-113): **Claude Code** (the user's own subscription) is the main coding agent; **OpenCode Go** models run the observers, judge and apprentice. Check every design against the ban-safe rule in D-041.
- When brainstorming ends and implementation planning starts, produce a **very detailed technical spec and roadmap** (crates, modules, types, IPC, tasks) before any code. The user asked for this explicitly.

## Don't

- **Never send emails, messages or posts on the user's behalf.** Draft them in an `ACTION` file with instructions; the user sends them.
- Don't use `.pi/` or `outputs/` as sources for the brainstorm (user instruction, 2026-10-07).
- Don't write application code until the user says brainstorming is done and asks for implementation.
- The folder is a git repo (`main`, since 2026-10-09; D-104). Commit only when the user asks. Other repo-structure changes, such as moving `brainstorm/` to `docs/brainstorm/`, still need the user's OK (v0.4-Q10).
