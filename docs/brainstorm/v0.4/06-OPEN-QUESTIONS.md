---
session: v0.4
date: 2026-10-09
doc: open-questions
round: 4
---

# Open Questions: Round 4 (the last brainstorming round)

> Read first: [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) · [02-PROPOSAL-AGENTS-AND-SESSIONS.md](02-PROPOSAL-AGENTS-AND-SESSIONS.md) · [03-PROPOSAL-ULAK-MESSAGING.md](03-PROPOSAL-ULAK-MESSAGING.md) · [04-PRODUCT-SPEC-DRAFT.md](04-PRODUCT-SPEC-DRAFT.md) · [05-CONSTITUTION-DRAFT.md](05-CONSTITUTION-DRAFT.md)
> Question IDs: **v0.4-Q1 … v0.4-Q12**. After this round, **v0.5 = the detailed S0 technical spec & roadmap**.

## How to answer (same as before)

1. **Pick:** `- [ ]` → `- [x]` (multi-select only where noted)
2. **Custom answer:** `✍️ Custom answer:` overrides the options
3. **Clarify:** `❓ Clarify:` and I'll answer next session
4. **Skip:** blank means my ⭐⭐ / ⭐ recommendation is used

Then say **"answers ready"**.

| ⭐⭐ Strongly recommended | ⭐ Recommended | ⚪ Viable | ⚠️ Not recommended |
|---|---|---|---|

---

## A. Agents & Sessions

### v0.4-Q1. Native-first v1?
**Context:** [02 §3](02-PROPOSAL-AGENTS-AND-SESSIONS.md#3-my-opinion-skip-v1-terminal--gui-go-native-first), your Q27 idea.
🔎 *What others do:* Zed and JetBrains are native-only over ACP; Warp and Conductor-style apps are terminal-first; almost nobody offers both on one project.

- [x] **A. v1.0 = native GUI over ACP by default + Terminal layout fallback, both built in S2** ⭐⭐
- [ ] **B. Keep the v0.3 plan: v1 Terminal + GUI, v2 Native** ⚪ *(faster S2, session UI rebuilt later)*
- [ ] **C. Native only, no terminal fallback** ⚠️ *(no escape if Anthropic changes billing)*

✍️ Custom answer:
❓ Clarify:

---

### v0.4-Q2. How Claude Code is connected
**Context:** [02 §2](02-PROPOSAL-AGENTS-AND-SESSIONS.md#2-how-atabey-connects-to-claude-code), [Clarifications Q27.4](01-CLARIFICATIONS.md#q274-can-i-use-claude-code-through-acp-without-licensing-problems).
🔎 *What others do:* Zed = ACP adapter with subscription login; JetBrains = ACP adapter, API key only when bundled.

- [x] **A. `claude-agent-acp` adapter (you install it; sign-in via Claude Code's own login) + hooks bridge for the terminal layout** ⭐⭐
- [ ] **B. Terminal + hooks only** ⚪ *(safest, least rich)*
- [ ] **C. Our own driver for `claude -p` stream-json** ⚠️ *(duplicates the adapter)*
- [ ] **D. Agent SDK with an API key** ⚠️ *(pay-per-token; breaks the budget)*

✍️ Custom answer:
❓ Clarify:

---

### v0.4-Q3. Second agent in S2 (to prove the ACP abstraction)
🔎 *What others do:* Zed launched ACP with two agents (Gemini CLI, then Claude Code) to keep the abstraction honest.

- [x] **A. OpenCode via `opencode acp`** ⭐⭐ *(needed for Kurultay and Talimhane anyway)*
- [ ] **B. Codex CLI** ⚪ *(needs a ChatGPT plan or API key)*
- [ ] **C. None in S2; Claude Code only** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.4-Q4. Observer (Kurultay) runtime
**Context:** [02 §5](02-PROPOSAL-AGENTS-AND-SESSIONS.md#5-observer-runtime-opencode-headless-not-a-new-harness), your Q27 question.
🔎 *What others do:* OpenCode Plan/Explore and Claude Code Explore are read-only by tool restriction; CodeRabbit runs its own hosted pipeline.

- [x] **A. OpenCode headless with custom read-only agents, in a separate worktree; engine writes findings** ⭐⭐
- [ ] **B. A new minimal harness calling OpenCode Go endpoints directly** ⚠️ *(must mimic coding-agent traffic; breaks D-010)*
- [ ] **C. Claude Code subagents** ⚠️ *(same vendor as the coder; uses your Claude limits)*

✍️ Custom answer:
❓ Clarify:

---

### v0.4-Q5. Your Claude plan and the budget meter
**Context:** the $1/day and $15/month caps are for pay-per-token calls. A subscription has usage *limits*, not per-call prices.
🔎 *What others do:* Claude Code shows plan usage with `/usage`; Zed and Cursor show subscription usage separately from pay-as-you-go spend.

- [x] **A. Show subscription usage separately (plan allowance, not $); $ caps apply only to pay-per-token calls** ⭐⭐
- [ ] **B. Count the subscription's monthly price inside the $3/day budget too** ⚪

✍️ Custom answer *(please also write which Claude plan you use: Pro, Max 5x, Max 20x…)*:
❓ Clarify:

---

### v0.4-Q6. Claude sign-in if Atabey becomes public
**Context:** [Clarifications Q27.4, row 3](01-CLARIFICATIONS.md#q274-can-i-use-claude-code-through-acp-without-licensing-problems).

- [x] **A. Personal use now; before any public release, re-check Anthropic's terms. Atabey never ships its own "Sign in with Claude"; users install the adapter and sign in through Claude Code** ⭐⭐
- [ ] **B. Public builds are API-key-only for Claude (JetBrains-style); subscription only in personal builds** ⚪

✍️ Custom answer:
❓ Clarify:

---

## B. Extensions

### v0.4-Q7. Ulak · Extension Messaging
**Context:** [03-PROPOSAL-ULAK-MESSAGING.md](03-PROPOSAL-ULAK-MESSAGING.md), your Q27 idea.
🔎 *What others do:* VS Code = commands + exports; JetBrains = extension points + message bus; Neovim = autocmd events; Zed = none.

- [x] **A. Commands + events + typed versioned services, all through the core broker; contracts generated from Rust types; S0 = commands + events** ⭐⭐
- [ ] **B. Commands + events only (no services)** ⚪ *(simpler, less typed)*
- [ ] **C. No inter-extension messaging** ⚠️

✍️ Custom answer:
❓ Clarify:

---

## C. Consolidation drafts

### v0.4-Q8. Product Spec draft
**Context:** [04-PRODUCT-SPEC-DRAFT.md](04-PRODUCT-SPEC-DRAFT.md). It becomes `PRODUCT-SPEC.md` v1.0 in v0.5.

- [x] **A. Accept as drafted (with my answers to this round applied)** ⭐⭐
- [ ] **B. Accept with corrections** (write them below)

✍️ Custom answer:
❓ Clarify:

---

### v0.4-Q9. Constitution draft
**Context:** [05-CONSTITUTION-DRAFT.md](05-CONSTITUTION-DRAFT.md). Pay special attention to **Article 10 (licenses)**.

- [x] **A. Accept as drafted** ⭐⭐
- [ ] **B. Accept with corrections** (write them below)

✍️ Custom answer:
❓ Clarify:

---

### v0.4-Q10. When does `brainstorm/` move to `docs/brainstorm/`?
**Context:** D-104 (monorepo later).
🔎 *What others do:* Rust RFCs and Python PEPs live in dedicated folders/repos; most apps keep design docs under `docs/`.

- [x] **A. At the start of v0.5, together with creating `PRODUCT-SPEC.md` and `CONSTITUTION.md` at the root** ⭐⭐
- [ ] **B. When S0 code starts** ⚪
- [ ] **C. Never; keep `brainstorm/` at the root** ⚪

✍️ Custom answer:
❓ Clarify:

---

## D. Anything else?

### v0.4-Q11. Corrections to v0.4

✍️ Custom answer:

### v0.4-Q12. New ideas (last call before the S0 spec; later ideas go to the Parking Lot)

✍️ Custom answer:
