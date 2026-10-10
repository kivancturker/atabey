---
session: v0.3
date: 2026-10-07
doc: open-questions
round: 3
---

# Open Questions: Round 3

> Read first: [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) · [02-PROPOSAL-PLATFORM.md](02-PROPOSAL-PLATFORM.md) · [03-PROPOSAL-QUALITY.md](03-PROPOSAL-QUALITY.md) · [04-PROPOSAL-LEARNING.md](04-PROPOSAL-LEARNING.md) · [05-HOW-WE-BUILD-IT.md](05-HOW-WE-BUILD-IT.md)
> 📌 Also for you: [07-ACTION-ROADMAPSH-EMAIL.md](07-ACTION-ROADMAPSH-EMAIL.md) (no rush).
> Question IDs: **v0.3-Q1 … v0.3-Q27**.

## How to answer (same as before)

1. **Pick:** `- [ ]` → `- [x]` (multi-select only where noted)
2. **Custom answer:** `✍️ Custom answer:` overrides the options
3. **Clarify:** `❓ Clarify:` and I'll answer next session
4. **Skip:** blank means my ⭐⭐ / ⭐ recommendation is used

Then say **"answers ready"**.

| ⭐⭐ Strongly recommended | ⭐ Recommended | ⚪ Viable | ⚠️ Not recommended |
|---|---|---|---|

---

## A. Identity

### v0.3-Q1. Product name
**Context:** [Clarifications → Q1](01-CLARIFICATIONS.md#q1-a-new-name-from-ottoman-seljuk-and-göktürk-history). Quick web check only, not a legal search.
🔎 *What others do:* dev tools use one short, distinctive brand (Zed, Warp, Kiro, Cursor).

- [x] **A. Atabey**: Seljuk guardian-tutor of princes (the predecessor of the Ottoman *lala*); no software collision found ⭐⭐
- [ ] **B. Enderun**: the Ottoman palace school ⭐
- [ ] **C. Lala** ⚠️ *(lala.ai, a local reasoning dev tool, and a "Lala AI" education app already exist)*
- [ ] **D. Tonyukuk**: Göktürk counsellor of Bilge Kağan ⚪ *(unique, but hard to spell internationally)*
- [ ] **E. Orhun**: the Orkhon inscriptions ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q2. Themed module names (Orhun, Kurultay, Divan, Tuğra, Defter, Enderun, Talimhane, Bedesten, Kervan)?
🔎 *What others do:* most tools use plain feature names; Kubernetes (pods, helm) and Rust (crates, cargo) show themed vocabularies can work **when every term has a clear subtitle**.

- [x] **A. Yes, themed names, always shown with a plain-English subtitle** (e.g., "Kurultay · Observer Swarm") ⭐⭐
- [ ] **B. Plain English names only** ⚪
- [ ] **C. Themed names only for some** (write which in the custom answer) ⚪

✍️ Custom answer:
❓ Clarify:

---

## B. Platform

### v0.3-Q3. Extension architecture
**Context:** [Platform → §1](02-PROPOSAL-PLATFORM.md#1-core--extensions-like-neovim).
🔎 *What others do:* VS Code and Obsidian = core + bundled first-party extensions; Neovim = tiny core (but LSP/tree-sitter moved into core); Eclipse = everything is a plugin (known for complexity).

- [x] **A. Core infrastructure + bundled first-party extensions on the same public API; API unstable (0.x) until 3+ extensions use it** ⭐⭐
- [ ] **B. Everything is an extension from day one (Eclipse-style)** ⚠️
- [ ] **C. Monolith first, extract extensions later** ⚪ *(faster start; extraction is usually painful)*

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q4. How do extensions run?
🔎 *What others do:* VS Code = separate extension-host process + sandboxed webviews; Zed = WebAssembly sandbox; Neovim = in-process Lua; MCP/LSP = separate processes over JSON-RPC.

- [ ] **A. Backend parts as separate processes (JSON-RPC over stdio, any language); UI parts in sandboxed iframes with a message API** ⭐⭐
- [ ] **B. WebAssembly sandbox for backend parts** ⚪ *(safest; fewer languages and libraries)*
- [ ] **C. In-process Rust plugins** ⚠️ *(one bad extension crashes the app; recompile for every version)*
- [ ] **D. A now, add B later for lightweight extensions** ⭐

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q5. Extension library (Bedesten) hosting
🔎 *What others do:* Zed = a central GitHub registry repo; Neovim (lazy.nvim) = any git URL; Claude Code = marketplaces as git repos; VS Code = Microsoft's marketplace server.

- [x] **A. A GitHub registry repo (index of name → repo, versions, checksums) + install by git URL for anything unlisted** ⭐⭐
- [ ] **B. Git URLs only, no registry** ⚪
- [ ] **C. Own marketplace server** ⚠️ *(you'd run a backend)*

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q6. Code intelligence layers
**Context:** [Platform → §4](02-PROPOSAL-PLATFORM.md#4-code-intelligence-tree-sitter-lsp-and-caching).
🔎 *What others do:* Sourcegraph = SCIP snapshots per commit; VS Code/Zed = live LSP only; Aider = tree-sitter only; Cursor = embeddings + Merkle-tree hashing.

- [x] **A. tree-sitter (always) + live LSP (when available) + SCIP snapshots at feature/phase end; our own cache keyed by content hash + commit** ⭐⭐
- [ ] **B. tree-sitter + live LSP only (no snapshots)** ⚪ *(simpler; no "time machine")*
- [ ] **C. tree-sitter only** ⚠️ *(no precise "used by")*

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q7. Decision models: default setup
**Context:** [Platform → §3](02-PROPOSAL-PLATFORM.md#3-decision-models-jev-and-future-ones). Jev costs effectively nothing ($0.042 per million input tokens) through the OpenRouter key we already plan to use.
🔎 *What others do:* Claude Code and Codex use rules + approval prompts + OS sandboxes; nobody uses a dedicated decision model yet.

- [x] **A. Wrapper in core; Jev via OpenRouter as the default backend; on by default when a key exists; escalate-only for security; fail-closed (ask you) when unavailable** ⭐⭐
- [ ] **B. Same, but the cheap-LLM fallback is also on when there's no decision-model key** ⭐
- [ ] **C. Off by default** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q8. Log retention (Defter)
🔎 *What others do:* Claude Code and Codex keep session JSONL files until you delete them; most apps rotate technical logs after a few days or weeks.

- [x] **A. Session ledgers 90 days raw + summaries forever; app logs 14 days; secrets always redacted; "export decisions dataset" later** ⭐⭐
- [ ] **B. Keep everything forever** ⚪ *(best for training data; disk grows)*
- [ ] **C. 30 days for everything** ⚪

✍️ Custom answer:
❓ Clarify:

---

## C. Quality Pipeline

### v0.3-Q9. Deployment (CD) in v1?
🔎 *What others do:* most AI IDEs stop at CI; Vercel/Netlify-style platforms make deploy one click but only for their own hosting.

- [x] **A. CI only in v1; deploy recipes via stack packs in v2** ⭐⭐
- [ ] **B. Basic deploy recipes in v1** (write which targets: Docker, Vercel, Fly.io, app stores…) ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q10. CI approach
**Context:** [Quality → §3](03-PROPOSAL-QUALITY.md#3-cicd-without-relying-on-the-ais-imagination).
🔎 *What others do:* Jenkins = self-hosted server; GitHub Actions = YAML; Dagger = CI as an SDK in containers.

- [x] **A. Atabey Pipeline Recipes: typed `pipeline.toml` + recipes from stack packs + local runner + generated GitHub Actions** ⭐⭐
- [ ] **B. A, plus Dagger as an optional backend later** ⭐
- [ ] **C. GitHub Actions YAML written by the AI** ⚠️
- [ ] **D. Jenkins** ⚠️ *(server to maintain)*

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q11. Kurultay judge model (for split votes only)
**Context:** [Quality → §5](03-PROPOSAL-QUALITY.md#5-kurultay-the-swarm-refined).
🔎 *What others do:* CodeRabbit/Greptile use one strong model + linters; research on LLM judges shows position/self-preference bias (so we anonymize).

- [ ] **A. Strongest model in your OpenCode Go plan** ⭐⭐ *(included in the subscription)*
- [ ] **B. Claude Opus 5.5 via API key, within the daily cap** ⚪ *(best quality; costs real money)*
- [ ] **C. No judge: majority vote only** ⚪
- [x] **D. A by default, B as an optional "escalate" button per finding** ⭐

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q12. Static analysis toolset
**Context:** [Quality → §4](03-PROPOSAL-QUALITY.md#4-static-analysis-guardrails). SonarQube's analyzers are now source-available (not open source), and it needs a Java server.
🔎 *What others do:* GitHub = CodeQL + SARIF; GitLab = SAST templates; CodeRabbit = many linters feeding an LLM.

- [x] **A. Gitleaks + OSV-Scanner + Trivy + Opengrep + per-language linters → SARIF, diff-aware gate, installed by stack packs** ⭐⭐
- [ ] **B. A + SonarQube Community Build as optional** ⚪
- [ ] **C. SonarQube as the main tool** ⚠️

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q13. Change narratives (diff + command ❓ explanations)
**Context:** [Quality → §2](03-PROPOSAL-QUALITY.md#2-change-narratives-explaining-diffs-and-commands).
🔎 *What others do:* CodeRabbit = PR walkthroughs; Claude Code = live one-line command descriptions; Warp = explain on demand.

- [x] **A. Generated automatically at feature end (your idea) + on demand when you click ❓ before then** ⭐⭐
- [ ] **B. Only at feature end** ⚪
- [ ] **C. Live, during the session** ⚠️ *(costlier; distracting)*

✍️ Custom answer:
❓ Clarify:

---

## D. Learning

### v0.3-Q14. When does Talimhane ship?
**Context:** [Learning → §2](04-PROPOSAL-LEARNING.md#2-talimhane-practice-ground-for-system-thinking).
🔎 *What others do:* CodeCrafters and Exercism are standalone products; none are built into an IDE.

- [x] **A. v1, as slice S7 (after the Mentor)** ⭐⭐
- [ ] **B. v2** ⚪ *(v1 ships sooner)*
- [ ] **C. Earlier, right after the Agent Session** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q15. Talimhane apprentice model
- [x] **A. Deliberately weak, cheap model by default; you can pick another** ⭐⭐
- [ ] **B. Adaptive: weaker as your skill grows** ⭐ *(later improvement)*
- [ ] **C. Strong model** ⚠️ *(it solves the problem for you)*

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q16. Sharing Talimhane scenarios
🔎 *What others do:* Exercism and CodeCrafters publish tracks openly on GitHub; LeetCode keeps problems proprietary.

- [x] **A. Personal first; public GitHub library once you have ~10 good scenarios (content under CC BY 4.0)** ⭐⭐
- [ ] **B. Public from the start** ⚪
- [ ] **C. Never public** ⚪

✍️ Custom answer:
❓ Clarify:

---

## E. Stack Packs & Theme

### v0.3-Q17. First Kervan stack pack
🔎 *What others do:* VS Code ships language support for its own stack first (TypeScript); dogfooding is the usual first step.

- [x] **A. `kervan-rust-tauri-react`: Atabey's own stack (dogfooding)** ⭐⭐
- [ ] **B. A + one more you use often** (write which) ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q18. How stack packs install tools
🔎 *What others do:* VS Code Dev Containers = Docker; mise/asdf = version managers; Nix = reproducible but steep.

- [x] **A. `mise` (cross-platform tool version manager, MIT)** ⭐⭐
- [ ] **B. Dev Containers** ⚪ *(isolated; needs Docker; slower)*
- [ ] **C. Print instructions, you install manually** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q19. Theme
- [x] **A. One Monokai-based dark theme (from VS Code's MIT-licensed Monokai) on semantic tokens; Shiki highlighting; VS Code theme import in S9** ⭐⭐
- [ ] **B. A different base dark theme** (write which: One Dark, Dracula, Tokyo Night…) ⚪

✍️ Custom answer: Later we will be able to choose multiple keep that in mind
❓ Clarify:

---

## F. Process

### v0.3-Q20. How we build Atabey itself
**Context:** [How we build it → §1–2](05-HOW-WE-BUILD-IT.md#1-spec-lite-a-homegrown-process-for-one-developer).
🔎 *What others do:* Spec Kit (constitution + slash commands), Kiro (IDE gates), BMAD (many personas), OpenSpec (change deltas).

- [x] **A. Spec-Lite (homegrown) + constitution idea from Spec Kit + delta-style amendments from OpenSpec** ⭐⭐
- [ ] **B. GitHub Spec Kit** ⚪
- [ ] **C. OpenSpec** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q21. Brainstorm structure
**Context:** [Clarifications → Q14](01-CLARIFICATIONS.md#q14-session-folders-vs-topic-threads).

- [x] **A. Session folders + topic tags on decisions (topic views computed in the app)** ⭐⭐
- [ ] **B. Session folders only** ⚪
- [ ] **C. Session folders + maintained topic files** ⚠️ *(more upkeep, contradictions)*

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q22. Revised build order
**Context:** [How we build it → §3](05-HOW-WE-BUILD-IT.md#3-revised-build-order-v1-and-v2).

- [x] **A. S0 Foundations → S1 Orhun → S2 Agent Session → S3 Divan + Spec → S4 Quality → S5 Kurultay → S6 Enderun → S7 Talimhane → S8 Overlay → S9 Bedesten/Kervan = v1.0; Native mode = v2** ⭐⭐
- [ ] **B. Same, but move S8 Overlay earlier** (you test GUI apps a lot) ⚪
- [ ] **C. Different order** (write it) ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q23. Learner profile git sync in v1
**Context:** [Clarifications → Q15](01-CLARIFICATIONS.md#q15-is-git-sync-for-the-learner-profile-cheap-enough-for-v1). Estimated at 1–2 small tasks thanks to per-device append-only files.

- [x] **A. Yes, include it in S6** ⭐⭐
- [ ] **B. Later** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q24. When does brainstorming end?
**Context:** [How we build it → §4](05-HOW-WE-BUILD-IT.md#4-when-does-brainstorming-end).

- [x] **A. v0.4 = consolidation (Product Spec + Constitution drafts); v0.5 = very detailed S0 technical spec & roadmap; then build** ⭐⭐
- [ ] **B. Keep brainstorming a few more rounds first** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.3-Q25. Repository setup
**Context:** this folder (`harness/`) isn't a git repository yet, so the brainstorm history isn't version-controlled.
🔎 *What others do:* nearly all OSS projects keep design docs in the same repo as the code (`docs/`, RFC folders).

- [x] **A. `git init` this folder now (I'd do it on your OK); later it becomes the Atabey monorepo, with `brainstorm/` moving to `docs/brainstorm/`** ⭐⭐
- [ ] **B. Keep this folder for planning only; create a separate code repo later** ⚪
- [ ] **C. No git for now** ⚠️

✍️ Custom answer:
❓ Clarify:

---

## G. Anything else?

### v0.3-Q26. Corrections to v0.3

✍️ Custom answer:

### v0.3-Q27. New ideas

✍️ Custom answer: Our coding agent priority is claude code, I am using it and for the observer part I will use opencode go subscription but idk if I should use the opencode harness or create something new.

We can create an extension communication protocol, like one extension can send message to other one in a predefined way. I don't know if it make sense.

Can I use claude code with using acl without licensing problems, like on the v2 gui access claude code via acl.

Cancel the roadmap.sh plan. I am not gonna take anything from them, AI can build a good one.

If we can bind most used harness with acp may be we should directly start the v2 full gui version rather than losing time on v1. I wonder you opinon as well.
