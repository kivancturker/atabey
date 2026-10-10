---
session: v0.4
date: 2026-10-09
doc: clarifications
answers-to: ../v0.3/06-OPEN-QUESTIONS.md
---

# Clarifications for Your v0.3 Answers

> You accepted the ⭐⭐ / ⭐ recommendation on 24 of 25 questions (Q4 was blank, so its ⭐⭐ applies). You added one custom answer (Q19) and five new ideas (Q27). This file answers each of them briefly and links to the detailed proposals.

## Contents

1. [What you decided (quick table)](#what-you-decided)
2. [Q19: multiple themes later](#q19-multiple-themes-later)
3. [Q27.1: Claude Code is the priority coding agent](#q271-claude-code-is-the-priority-coding-agent)
4. [Q27.2: Observers: use the OpenCode harness or build something new?](#q272-observers-use-the-opencode-harness-or-build-something-new)
5. [Q27.3: An extension-to-extension communication protocol: does it make sense?](#q273-an-extension-to-extension-communication-protocol)
6. [Q27.4: Can I use Claude Code through ACP without licensing problems?](#q274-can-i-use-claude-code-through-acp-without-licensing-problems)
7. [Q27.5: Cancel the roadmap.sh plan](#q275-cancel-the-roadmapsh-plan)
8. [Q27.6: Skip v1 and build the full-GUI version directly?](#q276-skip-v1-and-build-the-full-gui-version-directly)
9. [Q25: git init](#q25-git-init)
10. [Sources](#sources)

---

## What you decided

| Q | Topic | Your pick | Ledger |
|---|---|---|---|
| Q1 | Name | **Atabey** (legal check before going public) | D-006 ✅ |
| Q2 | Themed module names | Yes, always with a plain-English subtitle | D-008 ✅ |
| Q3 | Extension architecture | Core + bundled first-party extensions on one public API; API 0.x until 3+ users | D-027 ✅ |
| Q4 | Extension runtime | *(blank → ⭐⭐)* Backend = separate processes (JSON-RPC/stdio); UI = sandboxed iframes | D-100 ✅ |
| Q5 | Bedesten hosting | GitHub registry repo + install by git URL | D-101 ✅ |
| Q6 | Code intelligence | tree-sitter + live LSP + SCIP snapshots + own cache | D-090 ✅ |
| Q7 | Decision models | Jev via OpenRouter, on when a key exists, escalate-only for security, fail-closed | D-028 ✅ |
| Q8 | Log retention | Ledgers 90 days raw + summaries forever; app logs 14 days | D-029 ✅ |
| Q9 | Deployment | CI only in v1; deploy recipes in v2 | D-102 ✅ |
| Q10 | CI approach | Pipeline Recipes (`pipeline.toml`) + local runner + generated GitHub Actions | D-049a ✅ |
| Q11 | Judge model | **OpenCode Go's strongest model by default, plus an optional "escalate to Opus 5.5 (API key)" button per finding** | D-048 ✅ |
| Q12 | Analyzers | Gitleaks + OSV-Scanner + Trivy + Opengrep + linters → SARIF | D-049b ✅ |
| Q13 | Change narratives | Automatic at feature end + on demand (❓) | D-049 ✅ |
| Q14–16 | Talimhane | v1 as S7 · weak cheap apprentice model · personal first, public at ~10 scenarios (CC BY 4.0) | D-069 ✅ |
| Q17 | First stack pack | `kervan-rust-tauri-react` (dogfooding) | D-092 ✅ |
| Q18 | Tool installer | `mise` | D-103 ✅ |
| Q19 | Theme | Monokai-based dark + **multiple themes later** (see below) | D-091 ✅ |
| Q20 | Process | Spec-Lite + constitution + delta amendments | D-034 ✅ |
| Q21 | Brainstorm structure | Session folders + topic tags | D-081 ✅ |
| Q22 | Build order | S0–S9 as proposed. **Superseded by the v1/v2 merge below** if you accept v0.4-Q1 | D-025 🟡 |
| Q23 | Learner git sync | In S6 | D-065 ✅ |
| Q24 | End of brainstorming | v0.4 consolidation → v0.5 S0 spec → build | D-083 ✅ |
| Q25 | Repo setup | `git init` now; later a monorepo with `docs/brainstorm/` | D-104 ✅ |

---

## Q19: multiple themes later

> *"Later we will be able to choose multiple, keep that in mind."*

**Short answer:** noted. S0 builds a **theme system**, not a single hard-coded theme. Only Monokai ships at first, but adding a theme later is a data file, not a code change.

| Built in S0 | Why it matters later |
|---|---|
| **Semantic tokens** (`surface.panel`, `text.muted`, `diff.added`, `finding.critical`…), never raw colors in components | Any theme only has to fill in the tokens |
| **Theme registry + switcher** in Settings (with only one entry at first) | More themes appear in the list automatically |
| Themes are **data-only contributions** (a JSON/TOML file), loadable from an extension | Community themes through Bedesten in S9 |
| Shiki syntax highlighting reads the same theme | Code and UI always match |

**What others do:** VS Code and Zed ship many themes plus a JSON theme format; Neovim themes are plugins. All three show that **tokens + a registry from day one** makes this cheap. Retrofitting themes onto hard-coded colors is painful. (Decision D-091, updated.)

---

## Q27.1: Claude Code is the priority coding agent

> *"Our coding agent priority is Claude Code, I am using it, and for the observer part I will use the OpenCode Go subscription."*

**Short answer:** understood. This changes D-026 ("first agent: OpenCode"). The roles become:

| Role | Who | Paid by | Notes |
|---|---|---|---|
| **Main coding agent** | **Claude Code** | Your existing Claude subscription | Through ACP (native GUI) with a terminal fallback. See [02 §2](02-PROPOSAL-AGENTS-AND-SESSIONS.md#2-how-atabey-connects-to-claude-code) |
| **Kurultay observers + judge** | OpenCode Go models (3 vendors) | OpenCode Go ($10/mo) | Optional per-finding escalation to Opus 5.5 via API key (pay-per-token, flagged in the cost meter) |
| **Talimhane apprentice** | A cheap OpenCode Go model | OpenCode Go | Deliberately weak |
| **Decision model** | Jev via OpenRouter | ~$0 ($0.042 / M input tokens) | |
| **Second agent (proves the abstraction)** | OpenCode via ACP | OpenCode Go | Needed by S5/S7 anyway |

💰 **Budget note:** the $1/day and $15/month caps (D-047) were designed for pay-per-token calls. Your Claude subscription is a flat fee with usage *limits*, not a per-call price. I propose that the meter shows it separately (v0.4-Q6).

---

## Q27.2: Observers: use the OpenCode harness or build something new?

**Short answer:** use **OpenCode as the observer runtime**, running headless with custom **read-only agents**. Don't build a new harness. Detail: [02 §5](02-PROPOSAL-AGENTS-AND-SESSIONS.md#5-observer-runtime-opencode-headless-not-a-new-harness).

The three main reasons:

1. **OpenCode Go is built for coding-agent clients.** Its docs (checked 2026-10-09) say Go "is designed for OpenCode and other coding agents that produce similar types of requests". Other clients must send typical coding-agent traffic, use their own user agent, and send a stable `x-opencode-session` header. Traffic is "monitored for abuse". A homemade client would have to mimic all of that. OpenCode itself is the safest client.
2. **OpenCode already has what an observer needs:** read/grep/glob tools, LSP, per-agent model selection, and **per-agent permissions** (`edit: deny`, `bash: deny`, `webfetch: deny`). Its built-in **Plan** and **Explore** agents are already read-only precedents.
3. **It keeps D-010 ("wrap existing agents, no custom agent loop").** A new harness is a second product to maintain.

---

## Q27.3: An extension-to-extension communication protocol

> *"One extension can send a message to another in a predefined way. I don't know if it makes sense."*

**Short answer:** it makes a lot of sense. Every serious extensible editor has one (VS Code, JetBrains, Eclipse, Neovim). The important rule: **extensions never talk to each other directly. They talk through the core**, using **typed, versioned contracts**. Then permissions, logging (Defter) and crash isolation all still work.

I named it **Ulak · Extension Messaging** (*ulak* = the Ottoman/Seljuk state courier). It has three message styles:

| Style | Plain meaning | Production example |
|---|---|---|
| **Commands** | "Do this and tell me the result" | Kurultay calls `orhun.symbolContext` to build a context pack for a reviewer |
| **Events** | "This happened, whoever cares" | Quality pipeline publishes `pipeline.completed {status: green}` → Kurultay starts reviewing |
| **Services** | "I provide this typed interface (v1)" | `atabey.findings@1`: Kurultay provides it, Enderun and the Spec view consume it |

Full design: [03-PROPOSAL-ULAK-MESSAGING.md](03-PROPOSAL-ULAK-MESSAGING.md).

---

## Q27.4: Can I use Claude Code through ACP without licensing problems?

*(You wrote "ACL"; I'm reading it as **ACP**, the Agent Client Protocol.)*

**Short answer:** **for your own personal use, yes, today.** It runs the unmodified Claude Code binary and you sign in through Anthropic's own login flow. **Before releasing Atabey publicly, re-check the terms**, because Anthropic changed this policy several times in 2026. I'm not a lawyer; this is a reading of the public documents as of **2026-10-09**.

There are actually three separate questions:

| Question | Answer | Evidence (checked 2026-10-09) |
|---|---|---|
| **1. Copyright license:** can Atabey legally include or launch the pieces? | ✅ **Launching yes, bundling no.** Claude Code is proprietary (used under Anthropic's Consumer Terms for Free/Pro/Max). Atabey must **not bundle or modify** it: you install it yourself. The ACP adapter (`@agentclientprotocol/claude-agent-acp`) is **Apache-2.0**, the same as Atabey, and runs as a separate process, so there's no license conflict | Claude Code legal page; adapter repo |
| **2. Terms of service:** may your *subscription* be used through ACP? | ✅ **Personal use: yes.** Anthropic's legal page says Pro/Max limits "assume ordinary, individual usage of Claude Code **and the Agent SDK**". The adapter is built on the Agent SDK, which "runs the Claude Code binary". After pausing a planned billing change in June 2026, Anthropic said ACP, `claude -p` and Agent SDK usage keep drawing from subscription limits | Legal page; Agent SDK overview; Zed blog (June 16 update) |
| **3. Distribution:** if Atabey becomes public, may it offer Claude sign-in? | ⚠️ **Careful.** Anthropic "does not permit third-party developers to offer Claude.ai login into their own applications", and credentials must never be collected, stored or intermediated. So Atabey must **never have its own "Sign in with Claude" button** and never touch tokens. Sign-in happens inside Claude Code's own flow (the adapter's *terminal login* runs Claude Code's `/login`). JetBrains ships its bundled Claude agent **API-key-only** (`--hide-claude-auth`); Zed allows subscription login | Legal page; Agent SDK overview note; JetBrains plugin page; vimcode issue |

**Naming rule (also from Anthropic's docs):** Atabey may say it "runs Claude Code", but no Atabey feature may be *named* "Claude Code …". In the agent picker, Anthropic's guidelines allow the label **"Claude Agent"**.

**Billing risk to plan for:** the paused June 15 change would have moved ACP / `claude -p` / Agent SDK usage into a **separate monthly credit**, while the terminal CLI kept the full subscription limits. If Anthropic revives it, the **terminal fallback** becomes the budget-safe mode. That's why I recommend building both modes in the same slice ([02 §3](02-PROPOSAL-AGENTS-AND-SESSIONS.md#3-my-opinion-skip-v1-terminal--gui-go-native-first)).

---

## Q27.5: Cancel the roadmap.sh plan

> *"I am not gonna take anything from them, AI can build a good one."*

**Done.** D-066 is updated: skill trees are **fully AI-generated**, with **no roadmap.sh content or links**. The action item is cancelled, and [v0.3/07-ACTION-ROADMAPSH-EMAIL.md](../v0.3/07-ACTION-ROADMAPSH-EMAIL.md) stays only as frozen history.

To keep AI-generated trees good without an outside source, I propose these quality rules (they go into S6's spec):

| Rule | Why |
|---|---|
| Each tree is generated **from your actual code and stack pack** first, generic knowledge second | Trees fit what you really use |
| Each node lists **2–3 primary sources** (official docs, RFCs, well-known OSS code), never copied text | Grounded, verifiable, no copyright issues |
| A second model **reviews the tree for gaps and ordering** (cheap OpenCode Go call) | Catches missing prerequisites |
| Trees are **versioned files**; you can edit or strike nodes | Same "files are truth" rule as everything else |

---

## Q27.6: Skip v1 and build the full-GUI version directly?

> *"If we can bind the most used harnesses with ACP, maybe we should directly start the v2 full GUI version rather than losing time on v1. I wonder your opinion as well."*

**My opinion: yes, with one condition.** Make **native GUI over ACP the default from v1**, and **build the terminal mode in the same slice as a fallback**, not as a separate version first.

Why I changed my mind since v0.3:

| v0.3 assumption | What's different now |
|---|---|
| First agent = OpenCode, whose `serve` + `attach` made "TUI + live GUI on one session" easy | **Claude Code is now first, and it has no serve/attach.** In terminal mode, the GUI can only *observe* it through hooks, which is weaker |
| ACP was "for later" | ACP is **stable v1**, with an **agent registry** (Claude Code, Codex, OpenCode, Gemini CLI, Copilot CLI…), an **official Rust crate**, and many real GUI clients (Zed, JetBrains, Neovim, Emacs, OpenHands, Goose) |
| Native GUI = a lot of extra UI work | ACP already delivers **structured** messages, tool calls, diffs, plans, permission requests and subagent transcripts. Those are exactly the things Atabey wants to render richly anyway |

The condition: the **terminal fallback must exist from day one** because of the billing/policy risk above, and because some TUI-only features (certain slash commands, OpenCode's `/undo`) don't exist in ACP.

Full analysis, risks and the revised roadmap: [02 §3](02-PROPOSAL-AGENTS-AND-SESSIONS.md#3-my-opinion-skip-v1-terminal--gui-go-native-first). Decision: **v0.4-Q1**.

---

## Q25: git init

You approved it, but my `git init` attempt was **blocked by this session's permission settings**, so the folder is still not a repository. You can run it yourself (commands in [07-ACTION-GIT-INIT.md](07-ACTION-GIT-INIT.md)) or allow it and I'll do it next session.

---

## Sources

All checked **2026-10-09**.

- Anthropic, Claude Code [Legal and compliance](https://code.claude.com/docs/en/legal-and-compliance): license, "Authentication and credential use", Pro/Max limits wording
- Anthropic, [Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview): "runs the Claude Code binary", third-party login note, branding guidelines
- Zed, [What Anthropic's New Claude Billing Means for Zed Users](https://zed.dev/blog/anthropic-subscription-changes) (May 14, 2026; June 16 pause update)
- [agentclientprotocol/claude-agent-acp](https://github.com/agentclientprotocol/claude-agent-acp) (Apache-2.0)
- [Subscription ACP Agent plugin for JetBrains](https://plugins.jetbrains.com/plugin/33150-subscription-acp-agent) and [vimcode issue #957](https://github.com/JDonaghy/vimcode/issues/957): terminal auth and `--hide-claude-auth` (secondary sources)
- The Register, [Anthropic clarifies ban on third-party tool access](https://www.theregister.com/2026/02/20/anthropic_clarifies_ban_third_party_claude_access/) (Feb 2026 background)
- OpenCode, [Go docs](https://opencode.ai/docs/go/), [Agents](https://opencode.ai/docs/agents/), [ACP support](https://opencode.ai/docs/acp/)
- Zed, [The ACP Registry is Live](https://zed.dev/blog/acp-registry); ACP [Rust library](https://agentclientprotocol.com/libraries/rust)
