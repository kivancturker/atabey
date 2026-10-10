# Atabey Brainstorm

> Brainstorming history for **Atabey** *(the Seljuk guardian-tutor of princes; "Atelier" was dropped in v0.3)*: a Tauri desktop app that explains any codebase in a rich GUI and builds features through spec-driven, multi-agent development, with built-in mentorship and practice.
> **Phase:** brainstorming **complete** (2026-10-10). Now: **S0 spec review**, then implementation. No code yet.

## Where we are

| | |
|---|---|
| **Current session** | [v0.5](v0.5/SUMMARY.md) (open): S0 spec review & freeze |
| **Waiting on** | User's answers in [v0.5/03-OPEN-QUESTIONS.md](v0.5/03-OPEN-QUESTIONS.md) |
| **All decisions** | [DECISIONS.md](DECISIONS.md) |
| **Project documents** | [PRODUCT-SPEC.md](../../PRODUCT-SPEC.md) (frozen) · [CONSTITUTION.md](../../CONSTITUTION.md) (frozen) · [S0 spec](../../specs/S0-foundations.md) (draft) · [ROADMAP.md](../../ROADMAP.md) · [PARKING-LOT.md](../../PARKING-LOT.md) |
| **Next planned** | Freeze S0 → T-01 (workspace skeleton) |

## 📌 Action items for the user

| Item | Status |
|---|---|
| Put this folder under git ([steps](v0.4/07-ACTION-GIT-INIT.md)) | ✅ done (`outputs/`: v0.5-Q8) |
| Create the GitHub repo + push + $0 Actions budget ([steps](v0.5/04-ACTION-GITHUB-REPO.md)) | ⏳ after v0.5-Q6 |
| Trademark/domain check of "Atabey" before going public | ⏳ later |
| ~~Email roadmap.sh's founder~~ | ❌ cancelled (v0.3-Q27, D-114) |

## Sessions

| Version | Date | Title | Status |
|---|---|---|---|
| [v0.1](v0.1/SUMMARY.md) | 2026-10-07 | Initial idea, first proposal, first question round | closed |
| [v0.2](v0.2/SUMMARY.md) | 2026-10-07 | Answers processed, clarifications, priorities re-ordered, new pillars | closed |
| [v0.3](v0.3/SUMMARY.md) | 2026-10-07 | New name, extensions, quality pipeline, decision models, Talimhane, build process | closed |
| [v0.4](v0.4/SUMMARY.md) | 2026-10-09 | Claude Code first, native-first v1 over ACP, Ulak messaging, Product Spec + Constitution drafts | closed |
| [v0.5](v0.5/SUMMARY.md) | 2026-10-10 | Brainstorming ends; Product Spec + Constitution frozen; detailed S0 spec drafted | open |

## How this folder works

```mermaid
flowchart LR
  A["User answers the open-questions file<br/>(in the latest session folder)"] --> B["Claude reads the answers"]
  B --> C["New folder vX.Y+1<br/>clarifications · proposals · new questions · SUMMARY"]
  C --> D["Update DECISIONS.md + this README<br/>close the previous session"]
  D --> A
```

| Rule | Why |
|---|---|
| **One folder per session** (`v0.1`, `v0.2`, …) | Full history; nothing is overwritten |
| **Closed session folders are frozen.** The only exception: the user writes answers into that session's open-questions file | You can always see how ideas evolved |
| **Every session has a `SUMMARY.md`** | New sessions read summaries, not full history (cheaper; avoids context decay) |
| **`DECISIONS.md` is the living truth.** IDs never change; superseded rows stay | Decisions never get lost or contradicted |
| **Numbered files** give the reading order; a session may have several proposal files | |

### Reading order for a fresh session

1. This README
2. [DECISIONS.md](DECISIONS.md)
3. The latest session's `SUMMARY.md`
4. The latest session's open-questions file (and the user's answers in it)
5. Older files **only if needed** for detail

> This folder is the **manual prototype of the Brainstorm Studio** ("Divan", decision D-081). Once the app exists, it will render these files as rich pages and question forms.
