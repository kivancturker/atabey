---
doc: decision-ledger
updated: 2026-10-09
updated-in-session: v0.4
---

# Decision Ledger

> **The single source of truth for what has been decided.** Updated at the end of every session.
> Never delete a row. If a decision changes, mark the old row ⛔ `superseded` and point to the new ID.
> **Topic tags (D-081):** the section headings below are the topics; the app will compute topic views from them.

**Status:** ✅ `decided` · 🟡 `tentative` (recommended, awaiting the user's confirmation) · ⛔ `superseded` · ❓ `open`

## Identity & Goals

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-001 | ~~Codename "Atelier"~~ | ⛔ | v0.1-Q1 | → D-006 |
| D-002 | Primary user: **the user (solo dev)**; others later | ✅ | v0.1-Q2 | |
| D-003 | Personal tool first; open to open-sourcing | ✅ | v0.1-Q3 | |
| D-004 | **Purpose #1 = understand code without reading every line.** Purpose #2 = spec-driven development. GUI = delivery surface | ✅ | v0.1-Q4 | |
| D-005 | Constraints: solo, no deadline, **very low budget**, **type safety**, AI writes most code | ✅ | v0.1-Q29 | |
| D-006 | Name: **Atabey** (Seljuk guardian-tutor of princes) | ✅ | v0.2-Q1 → v0.3-Q1 | Trademark/domain check before going public |
| D-007 | License **Apache-2.0** | ✅ | v0.2-Q19 | Talimhane scenario content: CC BY 4.0 (D-069) |
| D-008 | Themed module names **always with a plain-English subtitle** (Orhun, Kurultay, Divan, Tuğra, Defter, Enderun, Talimhane, Bedesten, Kervan, Ulak) | ✅ | v0.3-Q2 | Ulak added in v0.4 |
| D-109 | **Product Spec + Constitution** drafts (v0.4/04, v0.4/05) become `PRODUCT-SPEC.md` and `CONSTITUTION.md` v1.0 in v0.5 | 🟡 | v0.4-Q8, Q9 | |

## Platform & Architecture

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-010 | **Wrap existing agents**; no custom agent loop | ✅ | v0.1-Q5 | Also applies to observers (D-106) |
| D-011 | **Tauri desktop app from day one** | ✅ | v0.1-Q6 | |
| D-012 | ~~CLI first → VS Code extension → Tauri~~ | ⛔ | v0.1 | → D-013 |
| D-013 | **Vertical slices**: engine + GUI together; CLI = debug tool only | ✅ | v0.1-Q30 | |
| D-014 | **Rust core + TypeScript GUI**, types generated from Rust | ✅ | v0.2-Q2 | |
| D-015 | GUI framework **React + TypeScript** | ✅ | v0.2-Q3 | |
| D-016 | **Cross-platform**: macOS primary, Windows supported, **CI on both from S0**; Linux best-effort | ✅ | v0.1-Q8, v0.2-Q18 | |
| D-017 | v1 uses the agent's own server (OpenCode `serve` + `attach` + SSE); ACP for v2 Native; MCP tools; Claude Code hooks | ✅ | v0.1-Q9, v0.3 | **🔁 v0.4-Q1/Q2 propose D-110**; becomes ⛔ if accepted |
| D-018 | v1 = Terminal + GUI; v2 = Native mode default + one-click switch back | ✅ | v0.2-Q10, Q21 | **🔁 v0.4-Q1 proposes D-111** |
| D-019 | Repo knowledge folder **`.atabey/`** | ✅ | v0.1-Q10 | Name decided (D-006) |
| D-020 | **Files = source of truth + SQLite derived index**; user-level `~/.atabey/` for cross-project data | ✅ | v0.2-Q8 | |
| D-021 | Sandbox: **container for PoC tests**, OS permission prompts for the main agent | ✅ | v0.1-Q11 | + decision gate (D-028) |
| D-022 | **Rich rendering of everything** (no raw Markdown, no ASCII diagrams) | ✅ | v0.1-Q30 | |
| D-023 | Document format: **Markdown + frontmatter + typed fenced blocks** (`atabey:*`), schemas from Rust types, auto-migration | ✅ | v0.2-Q7 | |
| D-024 | ~~Build order S0–S6 (v0.2)~~ | ⛔ | v0.2-Q4 | → D-025 |
| D-025 | Build order **S0 → S1 Orhun → S2 Session → S3 Divan + Spec → S4 Quality → S5 Kurultay → S6 Enderun → S7 Talimhane → S8 Overlay → S9 Bedesten/Kervan = v1.0** | ✅ | v0.3-Q22 | Slice contents revised by **D-112 🟡** |
| D-026 | ~~First agent: OpenCode~~ | ⛔ | v0.2-Q9 | → D-113 |
| D-027 | **Core infrastructure + bundled first-party extensions on one public API** (VS Code/Obsidian style); API 0.x until 3+ extensions use it | ✅ | v0.2-Q21 → v0.3-Q3 | |
| D-028 | **Decision-model wrapper** in core; **Jev via OpenRouter** default; on when a key exists; **escalate-only for security**; **fail-closed** (asks you) when unavailable | ✅ | v0.2-Q21, v0.3-Q7 | In native sessions it answers ACP permission requests (D-110) |
| D-029 | **Log every action, session-wise** (Defter); secrets redacted; **ledgers 90 days raw + summaries forever; app logs 14 days**; "export decisions dataset" later | ✅ | v0.2-Q21, v0.3-Q8 | |
| D-090 | Code intelligence: **tree-sitter (always) + live LSP + SCIP snapshots at feature/phase end**; own cache keyed by content hash + commit | ✅ | v0.3-Q6 | |
| D-091 | **Theme system from S0** (semantic tokens + registry + switcher); **Monokai-based dark** ships first (Shiki highlighting); **multiple themes later**; VS Code theme import in S9 | ✅ | v0.2-Q21, v0.3-Q19 | User: "later we will be able to choose multiple" |
| D-092 | **Tech stack packs** (Kervan); first pack **`kervan-rust-tauri-react`** (dogfooding) | ✅ | v0.2-Q21, v0.3-Q17 | |
| D-100 | Extension runtime: **backend = separate processes (JSON-RPC over stdio, any language); UI = sandboxed iframes with a message API** | ✅ | v0.3-Q4 (blank → ⭐⭐) | WASM option possible later |
| D-101 | **Bedesten** = GitHub registry repo (name → repo, versions, checksums) + install by git URL | ✅ | v0.3-Q5 | |
| D-103 | Stack packs install tools with **`mise`** | ✅ | v0.3-Q18 | |
| D-104 | **Git for this folder now**; later it becomes the Atabey monorepo with `brainstorm/` → `docs/brainstorm/` | ✅ | v0.3-Q25 | `git init -b main` done by the user 2026-10-09; first commit 2026-10-10; `outputs/` decision pending ([action](v0.4/07-ACTION-GIT-INIT.md)). Move timing: 🟡 v0.4-Q10 |
| D-105 | **Ulak · Extension Messaging**: commands + events + typed, versioned services, all through the core broker; contracts generated from Rust types; S0 = commands + events; 1.0 in S9 | 🟡 | v0.3-Q27 → v0.4-Q7 | User's idea |
| D-110 | **ACP-first agent connection from S2**: Claude Code via `claude-agent-acp` (user installs, signs in via Claude Code's own login); OpenCode via `opencode acp`; terminal layout via PTY + Claude hooks bridge + Atabey MCP server; OpenCode server API for headless batch jobs | 🟡 | v0.3-Q27 → v0.4-Q1, Q2 | Supersedes D-017 if accepted |
| D-111 | **v1.0 = native GUI over ACP by default + Terminal layout fallback (one click), both built in S2**; v2.0 = more doc types, CD, public Talimhane, more agents | 🟡 | v0.3-Q27 → v0.4-Q1 | Supersedes D-018 if accepted. Claude's opinion: yes |
| D-112 | Revised slice contents: S0 + theme system + Ulak v0; **S2 = ACP native + terminal fallback**; S5 = OpenCode headless; S7 apprentice via OpenCode/ACP; S9 + Ulak 1.0 + more themes | 🟡 | v0.4-Q1 | See [v0.4/02 §6](v0.4/02-PROPOSAL-AGENTS-AND-SESSIONS.md#6-revised-build-order) |
| D-113 | **Main coding agent: Claude Code** (user's own subscription). OpenCode (OpenCode Go) = observers, judge, apprentice, second agent | ✅ | v0.3-Q27 | Second agent in S2: 🟡 v0.4-Q3 |

## Spec-Driven Development

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-030 | **Freeze + formal amendments** | ✅ | v0.1-Q12 | |
| D-031 | **One spec per feature** + top-level product spec | ✅ | v0.1-Q13 | |
| D-032 | **Parking Lot**, triaged at every phase boundary | ✅ | v0.1-Q14, Q26 | |
| D-033 | **SemVer tied to roadmap phases** | ✅ | v0.1-Q21 | |
| D-034 | Build Atabey itself with **Spec-Lite** (homegrown) + **constitution** (Spec Kit idea) + **delta-style amendments** (OpenSpec idea) | ✅ | v0.2-Q21 → v0.3-Q20 | |

## Observer Swarm & Quality

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-040 | **About 3 observer models from different vendors**, weighted by measured precision | ✅ | v0.1-Q15 | |
| D-041 | Providers: **gateway primary** (OpenRouter / OpenCode Go) + direct APIs; **official binaries for subscriptions** | ✅ | v0.1-Q16 | **Ban-safe rule (refined v0.4):** subscriptions only via the vendor's **unmodified** binary (directly or via the vendor's official SDK), signed in through the **vendor's own flow**; Atabey never reads/stores/forwards credentials, never offers its own vendor sign-in, never bundles/patches vendor agents, no vendor trademarks in feature names; re-check terms before public release |
| D-042 | ~~Swarm runs per completed task~~ | ⛔ | v0.1-Q17 | → D-046 |
| D-043 | Advisory; **confirmed high/critical findings block acceptance** | ✅ | v0.1-Q18 | |
| D-044 | Hard caps + live cost meter; user budget < $3/day | ✅ | v0.1-Q19 | → defaults in D-047 |
| D-045 | Observers **structurally read-only**; the engine writes findings | ✅ | v0.1-Q30 | Mechanism: D-106 |
| D-046 | Swarm runs **per completed feature, after the pipeline (CI) is green** | ✅ | v0.2-Q21 | |
| D-047 | Default caps **$1/day, $15/month, adjustable**, pause and ask at the cap | ✅ | v0.2-Q13 | Subscription metering: 🟡 D-107 |
| D-048 | Swarm tiers: 3 medium models + **judge only for split votes**; judge = **strongest OpenCode Go model**, plus an optional **"escalate to Opus 5.5 (API key)" button per finding** | ✅ | v0.2-Q21, v0.3-Q11 | Escalation is pay-per-token, counted in the caps |
| D-049 | **Change narratives** (diffs grouped by intent + ❓ on every command/script), **generated automatically at feature end + on demand before then** | ✅ | v0.2-Q21, v0.3-Q13 | |
| D-049a | **Pipeline Recipes**: typed `pipeline.toml` + recipes from stack packs + local runner + generated GitHub Actions | ✅ | v0.2-Q21 → v0.3-Q10 | Dagger backend: maybe v2 |
| D-049b | Static analysis: **Gitleaks + OSV-Scanner + Trivy + Opengrep + per-language linters → SARIF**, diff-aware gate, installed by stack packs | ✅ | v0.2-Q21 → v0.3-Q12 | No SonarQube |
| D-102 | **CI only in v1**; deploy recipes via stack packs in v2 | ✅ | v0.3-Q9 | |
| D-106 | **Observer runtime: OpenCode headless** with custom read-only agents (`edit`/`bash`/`webfetch` = deny), in a **separate git worktree** at the feature commit; JSON findings returned, engine writes them | 🟡 | v0.3-Q27 → v0.4-Q4 | User asked: OpenCode harness or new? → OpenCode |
| D-107 | Claude subscription usage shown **separately as plan allowance**; $ caps apply only to pay-per-token calls | 🟡 | v0.4-Q5 | Which Claude plan: ❓ |
| D-108 | Public-release rule: Atabey never ships its own "Sign in with Claude"; re-check Anthropic terms before any public release | 🟡 | v0.4-Q6 | |

## Documentation & Codebase Atlas

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-050 | v1 docs: **ADRs (versioned), code map, high-level feature docs** | ✅ | v0.1-Q20 | |
| D-051 | **Explain any repo**, including existing code; ADRs for old code marked "inferred" | ✅ | v0.2-Q5 | |
| D-052 | **Hybrid generation**: facts + overview eager; file/class/function explanations lazy, cached by content hash | ✅ | v0.2-Q6 | |

## Mentorship & Learning

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-060 | Default mode **Debrief** | ✅ | v0.1-Q22 | |
| D-061 | Notebook + **built-in spaced repetition** (FSRS); no Anki export | ✅ | v0.1-Q23 | |
| D-062 | AI feedback on open answers (no score), **stored** | ✅ | v0.1-Q24 | |
| D-063 | **Cross-project learner profile** | ✅ | v0.1-Q30 | |
| D-064 | **Skill roadmaps** with progress | ✅ | v0.1-Q30 | |
| D-065 | Learner profile **local** in `~/.atabey/learner/`; per-device append-only files; **git sync included in S6** | ✅ | v0.2-Q15, v0.3-Q23 | |
| D-066 | ~~Skill trees AI-generated + links to roadmap.sh; user emails roadmap.sh for permission~~ | ⛔ | v0.2-Q16 | → D-114 |
| D-067 | Assessment: adaptive questions → **Verified**; failures shown gently as **"Not yet" 🌱** with history; **self-tick allowed** (badged "self-assessed") | ✅ | v0.2-Q17 | |
| D-068 | **Production-grade examples only** (your code → OSS systems → realistic scenarios); no toy examples; every lesson includes conventions + failure modes | ✅ | v0.2-Q21 | |
| D-069 | **Talimhane**: practice system thinking by directing a weak agent; mentor grades the *process*; **v1 as S7**; **deliberately weak, cheap apprentice model by default** (user can change); **personal first, public GitHub library at ~10 scenarios (CC BY 4.0)** | ✅ | v0.2-Q21, v0.3-Q14–16 | Adaptive difficulty: later |
| D-114 | Skill trees **fully AI-generated, no roadmap.sh content or links**; grounded in your code + stack pack; 2–3 primary sources per node; second-model gap review; versioned editable files | ✅ | v0.3-Q27 | roadmap.sh email cancelled |

## Feedback Overlay

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-070 | For **testing the app being developed** | ✅ | v0.1-Q25 | |
| D-071 | **On demand only** | ✅ | v0.1-Q30 | |
| D-072 | Message + **optional** capture (full screen or rectangle) | ✅ | v0.1-Q25 | |
| D-073 | **Hide → capture → show → crop frozen frame** | ✅ | v0.1-Q27 | |
| D-074 | Reports linked to the roadmap | ✅ | v0.1-Q26 | |
| D-075 | Atabey **suggests Defect (→ fix now) or Idea (→ Parking Lot)**; you confirm | ✅ | v0.2-Q11 | |
| D-076 | Opened via **global hotkey + menu-bar/tray icon** | ✅ | v0.2-Q12 | |

## Extensibility & Brainstorm

| ID | Decision | Status | Source | Notes |
|---|---|---|---|---|
| D-080 | Extensions = **MCP server + skills + UI manifest**, own repos | ✅ | v0.1-Q28 | Refined by D-027, D-100, D-105 |
| D-081 | **Brainstorm Studio** (Divan): versioned sessions, summaries, ledger, question forms; built in S3; **session folders + topic tags on decisions** (topic views computed in the app) | ✅ | v0.1-Q30, v0.2-Q14, v0.3-Q21 | |
| D-082 | Brainstorm conversations stored as **versioned folders** | ✅ | v0.2 | |
| D-083 | Brainstorming ends: **v0.4 consolidation → v0.5 detailed S0 technical spec & roadmap → build** | ✅ | v0.3-Q24 | |
