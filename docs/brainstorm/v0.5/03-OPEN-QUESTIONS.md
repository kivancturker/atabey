---
session: v0.5
date: 2026-10-10
doc: open-questions
round: 5
---

# Open Questions: Round 5 (S0 spec review & freeze)

> Read first: [S0 spec](../../../specs/S0-foundations.md) · [02-S0-TECH-CHOICES.md](02-S0-TECH-CHOICES.md) · [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md)
> Question IDs: **v0.5-Q1 … v0.5-Q14**. After this round the S0 spec is **frozen** and implementation starts with T-01.

## How to answer (same as before)

1. **Pick:** `- [ ]` → `- [x]` (multi-select only where noted)
2. **Custom answer:** `✍️ Custom answer:` overrides the options
3. **Clarify:** `❓ Clarify:` and I'll answer next session
4. **Skip:** blank means my ⭐⭐ / ⭐ recommendation is used

Then say **"answers ready"**.

| ⭐⭐ Strongly recommended | ⭐ Recommended | ⚪ Viable | ⚠️ Not recommended |
|---|---|---|---|

---

## A. Technical choices

### v0.5-Q1. Typed IPC bindings
**Context:** [02 §1](02-S0-TECH-CHOICES.md#1-typed-ipc-between-rust-and-typescript).
🔎 *What others do:* Spacedrive and Cap use specta/tauri-specta; many smaller Tauri apps hand-write `invoke()` calls.

- [x] **A. tauri-specta (exact pin `2.0.0-rc.25`); ts-rs as the documented fallback** ⭐⭐
- [ ] **B. ts-rs 12 + our own generator for command/event wrappers** ⭐ *(stable crate, ~200 more lines of our code)*
- [ ] **C. TauRPC** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.5-Q2. Where Markdown is parsed
**Context:** [02 §2](02-S0-TECH-CHOICES.md#2-where-markdown-is-parsed).
🔎 *What others do:* Zed parses in Rust; VS Code and Obsidian parse in TS.

- [x] **A. Rust (comrak) → typed AST → React** ⭐⭐
- [ ] **B. TypeScript (remark) in the GUI, plus a separate Rust parser for indexing** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.5-Q3. Frontend stack
**Context:** [02 §3](02-S0-TECH-CHOICES.md#3-frontend-stack).
🔎 *What others do:* modern Tauri apps commonly use React + Tailwind + Radix + TanStack; VS Code and Zed use their own UI frameworks.

- [x] **A. React 19 + TS 7 + Vite 8 + Tailwind 4 (semantic tokens) + Radix + TanStack Router/Query + Zustand + Biome + pnpm** ⭐⭐
- [ ] **B. Same, but CSS Modules instead of Tailwind** ⚪
- [ ] **C. Different** (write which parts) ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.5-Q4. Extension UI isolation
**Context:** [02 §4](02-S0-TECH-CHOICES.md#4-extension-ui-isolation), [Clarifications §3.1](01-CLARIFICATIONS.md#31-extension-iframes-and-a-past-tauri-security-issue-affects-d-100).
🔎 *What others do:* VS Code = separate-origin iframes + postMessage; Obsidian = no isolation; Figma = sandboxed iframes.

- [x] **A. Opaque-origin sandboxed iframes + postMessage bridge, proven by the T-04 spike on macOS + Windows; revisit dedicated webviews before third-party extensions (S9)** ⭐⭐
- [ ] **B. Dedicated webviews per extension panel from the start** ⚪ *(stronger, more work)*

✍️ Custom answer:
❓ Clarify:

---

### v0.5-Q5. First real extension
**Context:** [02 §5](02-S0-TECH-CHOICES.md#5-the-first-real-extension).

- [x] **A. Defter · Ledger Viewer (live, filterable action log)** ⭐⭐
- [ ] **B. Git Pulse** ⚪
- [ ] **C. Something else** (write it) ⚪

✍️ Custom answer:
❓ Clarify:

---

## B. Repository & workflow

### v0.5-Q6. GitHub repo visibility and CI
**Context:** [02 §6](02-S0-TECH-CHOICES.md#6-github-repo-visibility--ci-cost). macOS runners cost $0.062/min on private repos; public repos are free.
🔎 *What others do:* open-source Tauri apps are public with free CI; private projects limit macOS CI to `main`/releases.

- [x] **A. Public repo, full macOS + Windows CI on every push ($0)** ⭐⭐ *(the code and brainstorm history become visible; name not yet trademark-checked)*
- [ ] **B. Private repo; Linux + Windows on every push, macOS on merges to `main` + weekly** ⭐ *(≈ $0–12/month; set the Actions budget to $0 to be safe)*
- [ ] **C. Private repo, full matrix on every push** ⚠️ *(≈ $45–55/month)*

✍️ Custom answer:
❓ Clarify:

---

### v0.5-Q7. Your Claude plan (repeated from v0.4-Q5)
- [x] **Pro**
- [ ] **Max 5x**
- [ ] **Max 20x**
- [ ] **Team / Enterprise**

✍️ Custom answer:

---

### v0.5-Q8. The `outputs/` folder
- [x] **A. Add `outputs/` to `.gitignore`** ⭐⭐ *(not part of Atabey; stays on your disk)*
- [ ] **B. Commit it** ⚪
- [ ] **C. Move it out of this folder** ⚪ *(you do it)*

✍️ Custom answer:

---

### v0.5-Q9. How tasks are delivered
**Context:** [02 §7](02-S0-TECH-CHOICES.md#7-how-we-work-during-implementation).
🔎 *What others do:* Copilot's coding agent always opens PRs; Spec Kit uses a branch per feature.

- [x] **A. One branch + PR per task; CI must be green; you review and merge** ⭐⭐
- [ ] **B. Commit directly to `main` after each task** ⚪

✍️ Custom answer:
❓ Clarify:

---

## C. Amendments & freeze

### v0.5-Q10. Product Spec amendment A-1 (OpenCode Go only through OpenCode)
**Context:** [Clarifications §3.2](01-CLARIFICATIONS.md#32-opencode-go-should-only-be-used-through-opencode-proposed-product-spec-amendment-a-1), [PRODUCT-SPEC §15](../../../PRODUCT-SPEC.md#15-amendments).

- [x] **A. Approve → PRODUCT-SPEC v1.1.0** ⭐⭐
- [ ] **B. Reject** (the gateway may call OpenCode Go directly) ⚠️

✍️ Custom answer:
❓ Clarify:

---

### v0.5-Q11. Constitution amendment A-1 (CC0-1.0, 0BSD, BSL-1.0)
**Context:** [Clarifications §3.3](01-CLARIFICATIONS.md#33-two-licenses-missing-from-constitution-article-10-proposed-amendment-a-1), [CONSTITUTION Amendments](../../../CONSTITUTION.md#amendments).

- [x] **A. Approve → CONSTITUTION v1.1.0** ⭐⭐
- [ ] **B. Reject** (we'd need a different file watcher) ⚠️

✍️ Custom answer:
❓ Clarify:

---

### v0.5-Q12. Freeze the S0 spec
**Context:** [specs/S0-foundations.md](../../../specs/S0-foundations.md): 40 requirements, 22 acceptance criteria, 26 tasks in 6 milestones.

- [x] **A. Freeze as drafted, with this round's answers applied → start T-01** ⭐⭐
- [ ] **B. Freeze with corrections** (write them below)
- [ ] **C. Too big: split S0 into S0a (M1–M3) and S0b (M4–M6)** ⚪

✍️ Custom answer:
❓ Clarify:

---

## D. Anything else?

### v0.5-Q13. Corrections (including anything in "smaller choices already made", [02 §8](02-S0-TECH-CHOICES.md#8-smaller-choices-already-made-in-the-spec))

✍️ Custom answer:

### v0.5-Q14. New ideas (they go to [PARKING-LOT.md](../../../PARKING-LOT.md) now that brainstorming is over)

✍️ Custom answer:
