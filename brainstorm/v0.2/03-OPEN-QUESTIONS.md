---
session: v0.2
date: 2026-10-07
doc: open-questions
round: 2
---

# Open Questions: Round 2

> Read first: [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) (answers to your ❓) and [02-UPDATED-PROPOSAL.md](02-UPDATED-PROPOSAL.md) (what changed).
> Question IDs in this file are **v0.2-Q1 … v0.2-Q19**.

## How to answer (same as before)

1. **Pick an option:** change `- [ ]` to `- [x]`. Multi-select only where noted.
2. **Custom answer:** write on the `✍️ Custom answer:` line. It overrides the options.
3. **Clarify:** write on the `❓ Clarify:` line and I'll answer in the next session.
4. **Skip:** leave it blank and I'll use my ⭐⭐ / ⭐ recommendation.

Then tell me **"answers ready"**.

| Mark | Meaning |
|---|---|
| ⭐⭐ | Strongly recommended |
| ⭐ | Recommended |
| ⚪ | Viable, depends on your goals |
| ⚠️ | Not recommended, but your call |

Each question includes a **🔎 What others do** line, as you asked.

---

## A. Identity & Stack

### v0.2-Q1. Product name (after the legal check)
**Context:** an IDE called *InterSystems Atelier* existed (now deprecated). See [Clarifications → Q1](01-CLARIFICATIONS.md#q1-is-atelier-legally-safe).
🔎 *What others do:* most projects start with a codename and rename before launch (e.g., Codex CLI and Claude Code shipped with names chosen at launch; many OSS tools rename after a trademark notice, which is painful once there are users).

- [ ] **A. Keep "Atelier" as a codename; choose the public name before sharing publicly** ⭐⭐
- [ ] **B. Commit to "Atelier" as the final name now** ⚠️ *(real overlap in the IDE space)*
- [x] **C. Pick a different name now.** Write ideas in the custom answer and I'll brainstorm and check them ⚪

✍️ Custom answer: There are too many atelier related tools so I don't like it. Lets make it more related to ottoman era, middle asian turkish era like Gokturks, or seljuk empire. My thought is lala, lala's are mentors for ottoman princes it can be lala related name, or a different one I don't have any but no atelier. 
❓ Clarify:

---

### v0.2-Q2. Engine language
**Context:** see [Clarifications → Q7](01-CLARIFICATIONS.md#q7-rust-vs-typescript-for-the-engine).
🔎 *What others do:* Codex CLI, Zed, Warp and Goose's core use **Rust**; Claude Code, Gemini CLI and OpenCode use **TypeScript**. Desktop apps that touch the OS a lot lean to Rust.

- [x] **A. Rust core + TypeScript GUI, with types generated from Rust** ⭐⭐
- [ ] **B. TypeScript engine as a Bun/Node sidecar inside Tauri** ⚪ *(faster to iterate; two runtimes; bigger app)*
- [ ] **C. Start TypeScript, port hot paths to Rust later** ⚠️ *(likely means a rewrite, as Codex did)*

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q3. GUI framework (inside Tauri)
🔎 *What others do:* the Codex app and VS Code-based tools use **React**. Many Tauri apps use **Svelte** or **Solid** for small bundles. Zed uses its own Rust UI (not an option for us).

- [x] **A. React + TypeScript** ⭐⭐ *(largest ecosystem: mermaid, Monaco, xterm.js, diff viewers; the stack AI models write most reliably)*
- [ ] **B. Svelte + TypeScript** ⚪ *(smaller and simpler; smaller ecosystem)*
- [ ] **C. SolidJS + TypeScript** ⚪ *(very fast; smallest ecosystem)*
- [ ] **D. Vue + TypeScript** ⚪

✍️ Custom answer:
❓ Clarify:

---

## B. Codebase Atlas (Purpose #1)

### v0.2-Q4. Confirm the build order
**Context:** [Updated proposal → §10](02-UPDATED-PROPOSAL.md#10-revised-build-order-vertical-slices).
🔎 *What others do:* DeepWiki and Code Wiki started with *understanding*; Kiro and Spec Kit started with *specs*; Cursor and Claude Code started with *agents*.

- [x] **A. S0 Foundations (renderer, opens `brainstorm/`) → S1 Atlas → S2 Agent Session → S3 Brainstorm + Spec → S4 Swarm → S5 Mentor → S6 Overlay** ⭐⭐
- [ ] **B. Same, but Brainstorm Studio before the Atlas** ⚪ *(you'd brainstorm in the app sooner; Purpose #1 comes later)*
- [ ] **C. Agent Session first** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q5. Should Atelier explain existing codebases (not created with Atelier)?
🔎 *What others do:* DeepWiki, Code Wiki and Cody all work on any repo. Kiro/Spec Kit docs only exist for what was specced in them.

- [x] **A. Yes, any repo. Decisions (ADRs) for old code are marked "inferred"** ⭐⭐
- [ ] **B. Only projects created inside Atelier** ⚠️ *(you couldn't use it on your existing projects)*

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q6. When are explanations generated?
**Context:** [Updated proposal → §3.2](02-UPDATED-PROPOSAL.md#32-how-its-generated-facts-first-ai-second). Your budget is under $3/day.
🔎 *What others do:* DeepWiki generates the whole wiki up front (they pay for it); Cody and Cursor answer on demand and keep nothing; Swimm docs are written by humans.

- [x] **A. Hybrid: facts + overview eagerly; file/class/function explanations lazily when opened, cached by content hash** ⭐⭐
- [ ] **B. Everything eagerly ("explain the whole repo" up front)** ⚠️ *(expensive on large repos)*
- [ ] **C. Everything lazily (even the overview)** ⚪ *(cheapest; the first open is slower)*
- [ ] **D. A, plus an optional "explain everything" button with a cost estimate shown first** ⭐

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q7. Document format
**Context:** [Updated proposal → §4](02-UPDATED-PROPOSAL.md#4-foundation-rich-document-renderer--the-atelier-document-format).
🔎 *What others do:* Kiro, Spec Kit and Cline use **plain Markdown**; Stripe uses **Markdoc** (validated custom tags); doc sites use **MDX**; Notion/Jupyter use **JSON**.

- [x] **A. Markdown + frontmatter (`atelier: doc@1`) + typed fenced blocks, schemas generated from Rust types, auto-migration** ⭐⭐
- [ ] **B. Markdoc-style custom tags** ⚪
- [ ] **C. MDX** ⚠️ *(AI-written files would execute code)*
- [ ] **D. JSON documents** ⚠️ *(unreadable outside the app)*

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q8. Storage (re-confirm)
**Context:** [Clarifications → Q10](01-CLARIFICATIONS.md#q10-files-vs-database). You picked A (files only) and wanted the trade-offs.
🔎 *What others do:* Claude Code, Codex, Kiro, Spec Kit and Cline keep **knowledge in files**; Cursor keeps chat history in **SQLite** and users struggle to back it up or move it.

- [ ] **A. Files only** ⚪ *(your v0.1 pick)*
- [x] **B. Files = source of truth + SQLite as a derived, rebuildable index (plus a user-level `~/.atelier/` for cross-project data)** ⭐⭐

✍️ Custom answer:
❓ Clarify:

---

## C. Agents & Providers

### v0.2-Q9. First agent to integrate (Slice S2)
🔎 *What others do:* Zed and JetBrains integrate many agents through **ACP**; Conductor-style apps started with Claude Code only.

- [x] **A. OpenCode via ACP** ⭐⭐ *(open source, native ACP, works with your **OpenCode Go** subscription, no ban risk)*
- [ ] **B. Codex via app-server** ⚪ *(richest protocol; needs a ChatGPT plan or API key)*
- [ ] **C. Claude Code via Terminal mode (official binary + hooks + MCP)** ⚪ *(the subscription-safe path; less structured)*
- [ ] **D. Claude Code via ACP adapter with an API key** ⚠️ *(pay per token; hard on your budget)*

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q10. Default session mode
**Context:** [Clarifications → Q6 + Q30](01-CLARIFICATIONS.md#q6--q30-gui-and-engine-head-to-head).
🔎 *What others do:* Zed and the Codex app are **native GUI** by default; Conductor-style tools and Warp are **terminal-first**.

- [x] **A. Native mode when the agent supports it, Terminal mode as automatic fallback** ⭐⭐
- [ ] **B. Always Terminal mode (two-pane) for v1, native later** ⚪
- [ ] **C. You choose per session** ⚪

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q11. Feedback overlay: defect vs idea routing
**Context:** [Clarifications → Parking Lot](01-CLARIFICATIONS.md#q14--q26-what-a-parking-lot-is).
🔎 *What others do:* Scrum treats defects against the current sprint goal as part of the work, and new requests as backlog. Linear triages everything through one inbox.

- [x] **A. Atelier suggests Defect (→ fix now) or Idea (→ Parking Lot); you confirm with one click** ⭐⭐
- [ ] **B. Everything goes to the Parking Lot** ⚪ *(simplest, but even real bugs wait)*
- [ ] **C. Everything goes to the current task** ⚠️ *(scope creep)*

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q12. How is the feedback overlay opened?
🔎 *What others do:* CleanShot X, Shottr and the Windows Snipping Tool use a **global hotkey + menu-bar/tray icon**; Jam.dev/Marker.io use a **browser extension button**.

- [x] **A. Global hotkey + menu-bar/tray icon** ⭐⭐
- [ ] **B. Button inside Atelier only** ⚪
- [ ] **C. A + B** ⭐

✍️ Custom answer:
❓ Clarify:

---

## D. Budget

### v0.2-Q13. Default money caps (pay-per-token APIs only; subscriptions are tracked as quota %)
**Context:** [Clarifications → Q16 + Q19](01-CLARIFICATIONS.md#q16--q19-providers-ban-risk-and-your-budget). OpenCode Go at $10/mo ≈ $0.33/day is tracked separately.
🔎 *What others do:* Claude Code and Codex show usage and limits; Cline and Roo show per-task cost; few tools enforce **hard** caps by default.

- [ ] **A. $0.50/day, $10/month hard caps** ⚪
- [x] **B. $1/day, $15/month hard caps** ⭐⭐
- [ ] **C. $2/day, $30/month hard caps** ⚪

✍️ Custom answer: Ofcourse we should be able to adjust the caps but it can be the default for now.
❓ Clarify:

---

## E. Brainstorm Studio

### v0.2-Q14. Brainstorm structure
**Context:** [Updated proposal → §5](02-UPDATED-PROPOSAL.md#5-pillar-c-brainstorm-studio-new). This repo's `brainstorm/` folder is the prototype.
🔎 *What others do:* Claude Code uses `CLAUDE.md` + auto-compaction; ChatGPT/Claude Projects use opaque memory; Cline overwrites a memory bank in place. None keep versioned sessions + a decision ledger.

- [x] **A. Keep the current structure (session folders, SUMMARY.md, DECISIONS.md ledger, question sets) and build it into the app in S3** ⭐⭐
- [ ] **B. Same, but add per-topic threads (e.g., "storage", "swarm") across sessions** ⚪ *(better navigation; more structure to maintain)*

✍️ Custom answer:
❓ Clarify: You recommend A but B is also seems like make sense though, I need pros and cons for this.

---

## F. Mentor & Learning

### v0.2-Q15. Where does the cross-project learner profile live?
🔎 *What others do:* Claude Code keeps user-level memory in `~/.claude/`; Codex in `~/.codex/`; roadmap.sh and Duolingo keep progress in a cloud account.

- [ ] **A. Local `~/.atelier/learner/` (files + SQLite), never uploaded** ⭐⭐
- [x] **B. A + optional sync via a private git repo you own** ⭐ *(use it on both your Mac and Windows machines)*
- [ ] **C. Cloud account** ⚠️ *(needs a backend; premature)*

✍️ Custom answer: Starting with A but keeping B in mind in the future if it is hard to implement. If its easy to implement and doesn't require much effort we can handle in v1.
❓ Clarify:

---

### v0.2-Q16. Skill roadmaps source
**Context:** [Updated proposal → §6.2](02-UPDATED-PROPOSAL.md#62-skill-roadmaps-the-roadmapsh-question). roadmap.sh content is copyrighted: personal use is OK, redistribution is not without consent.
🔎 *What others do:* roadmap.sh curates by hand; Duolingo and Exercism curate their own trees; AI tutors generate trees on demand (inconsistent quality).

- [ ] **A. AI-generated trees (strong model, reviewed by you) + links to roadmap.sh per topic** ⭐⭐
- [x] **B. A, and also email roadmap.sh's author to ask permission to use their trees** ⭐
- [ ] **C. Import roadmap.sh trees locally for personal use only** ⚠️ *(breaks if you ever share Atelier)*

✍️ Custom answer: First lets start by ai generated trees I will email the roadmap.sh's founder when I find the time. Btw you will not email him using my thing maybe you can guide me who to email and the email in a .md file give me instructions but do not email yourself.
❓ Clarify:

---

### v0.2-Q17. When is progress on a skill node counted?
🔎 *What others do:* roadmap.sh: you tick it yourself. Duolingo/Exercism: you pass exercises.

- [x] **A. Evidence-based: met in your code + passed quiz cards + answered "why" questions; you can also tick manually** ⭐⭐
- [ ] **B. Manual ticks only** ⚪
- [ ] **C. Quiz results only** ⚪

✍️ Custom answer: Enough questions to test users skill, and if you think they are eligible let them pass if they are not they should be able to tick themselves, for failures we need to show they are failed like you tried it and failed indicator on the tree. It shouldn't crush their ego way but an indicator.
❓ Clarify:

---

## G. Platform & Sharing

### v0.2-Q18. When do Windows builds start?
🔎 *What others do:* Zed and Codex added Windows late and it was painful; Claude Code supported Windows earlier.

- [x] **A. From S0: CI builds and smoke tests on macOS + Windows** ⭐⭐
- [ ] **B. After S2** ⚪
- [ ] **C. After v1** ⚠️

✍️ Custom answer:
❓ Clarify:

---

### v0.2-Q19. Open-source license (only matters when you share)
🔎 *What others do:* Codex CLI uses **Apache-2.0**; Gemini CLI uses **Apache-2.0**; OpenCode uses **MIT**; Zed uses **GPL/AGPL** (with Apache for parts).

- [ ] **A. Decide later, when you share** ⭐⭐
- [ ] **B. MIT** ⚪ *(simplest, most permissive)*
- [x] **C. Apache-2.0** ⭐ *(permissive + patent protection)*
- [ ] **D. GPL/AGPL** ⚪ *(forks must stay open)*

✍️ Custom answer:
❓ Clarify:

---

## H. Anything else?

### v0.2-Q20. Corrections to v0.2
Anything in [01-CLARIFICATIONS.md](01-CLARIFICATIONS.md) or [02-UPDATED-PROPOSAL.md](02-UPDATED-PROPOSAL.md) I got wrong or you want changed?

✍️ Custom answer:

### v0.2-Q21. New ideas for the next session

✍️ Custom answer: On the first version we will use terminal + our gui but v2 will be completely gui but as a fallback we need a simple button to switch back to first versions. 

What about if AI is able to explain the git diffs? Is it too much. Like we will show bunch of git diffs and there is an explanation of what this git diff achieves, etc. Also the intention of the script exections like bash scripts, custom pythong scripts and executed terminal commands. There will be something like question mark at the box of that script execution, on top right position there is a question mark about what it does and explanation. (At the end of feature implementation).

After something like CI/CD finishes without any problem and we complete the whole feature and ready to go to the next one thats when we will start our observer agents and other custom agents.

I can change my mind in the future about those ide feature implementations. Therefore its crucial to implement in a scalable way like we will install the ide with only core features after that there will be something like plugin library or extension library, I don't know the difference, but it will be something like that also our plugins will be stored in the github. It will automatically install from plugin/extensin library. What about this there will be a certain way to implement those plugins and extensions, there will be agent SKILLs for implementing those plugins or extensions. It is open for discussion which features can be core features and also making it something like plugin/extension kind of thing pays off or limits our ability to develop. What others do? Which makes more sense, it makes sense to me but I wonder your thought. It will be like neovim.

Also we will use a simple spec driven development approach to implement this ide application, make your suggestion for a solo developer like me. I don't like external spec driven development toolkits but if you insist I would contemplate about it. You will explain all the technical details, and roadmap in a very detailed way just remember that, I am telling you to remeber we will keep brainstorming with you for now.

That education tool should explain with realworld production grade examples not like animal, cat, dog class type of way. It should be like a real way real complicated system examples. What is the common conventions etc.

Maybe we can use LSP to show how codes interract with each other where does this method or class beign used etc. Does lsp and treesitter kind of programs have some kind of cache if they don't we can make our own caches per feature implementation phases (or git commits, or whatever idk).

How about this medium segment ai's will share their findings, in somewhere (multiple agents do multiple once). Then they will look at each others findings and contamplate, or we can use something like opus 5.5 like capable models to do it.

We can make this tool to easiy create ci/cd pipeline. Maybe there is a predefined way to achieve this or let AI decide all of it. We should think about it. Maybe we can create a small tool for building ci/cd for us so AI don't have to rely on its own capability to create ci/cd pipeline. Jenkins is coming to my mind idk what is in yours and also can we do such a thing like a ci/cd sdk or cli, plugin or extension or whatever to create those pipeline without too much reliance of ai. I want ai to have an easier time to deploy that ci/cd. Those ci/cd can be remote or local, we should decide.

We can use free and open source with permissive linceses static code analysis tools like sonarqube to include even more strictire guardrails to the agents. Here is what is in my mind. Especially at the end of ci/cd cycle even ifthey fail or not we will make those things work and supply to the coding agent. I don't know what is the industry standard and common convention for this kind of thing but we should think about it as well.

For the learning section I want an interactive tool, I want our users to focus more on the system thinking rather than the direct code thinking. To achieve this we can create a small tool to make small agentic coding sessions on a predefined questions, real life production grade problems and providing it a solution with agentic coding. AI will create such code examples and real life problems, user solve it with prompting, then mentor AI will see our solutions how did we think about when we solve by looking at our prompts what the prompt generate what happend when AI give a bad solution, how did we set guardrail prompts so AI is almost impossible to make mistakes. Which means we will improve more on software engineering and computer science skills rather than coding skills. People will think in system and computer science details rather than syntax memorization.

Maybe we can share those real life production grade question on the internet and everyone can solve on their own. We can use dumber models to solve problems so user can navigate the problem their selves more. AI can generate follow up question what happens if that happens instead of that how would you solve that kind of things?

There is a new tool called jew its not a language model but more like a decision model you gave a promopt and also a multiple options it will choice the option. What can be done with something like jev. Btw it shouldn't be just jev, in the future they will bring more capable models like jev as a decision models like maybe openai and antropic does that. So keep that in mind there should be a wrapper instead of hard coding the jev. Also make a research how can we use jev in our use cases. Some of are something like before executing terminal command jev decide wether the command is malicios or not etc. Also because this tool require an api or something it should also become optional not mandatory but nice to have. If there is no api key for this than our tool is unable to use decision models. It is turned on by default.

All the actions taken by the tool is logged somewhere. Especially the session wise. Because I want to be able to troubleshoot the problems using the logs. Also it can be used to train some kind of my own model.

First theme will be dark mode but in the future we will create different themes like vscode theme etc. But it will be dark mode theme first. (that monokai or something)

There can be a techstack section, like for a specific tech stack it can install necessary tools and SKILLs for our ide. It is also can have it own library. Like a tech stack library.