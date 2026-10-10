# Open Questions

> Companion to [01-PROJECT-UNDERSTANDING.md](01-PROJECT-UNDERSTANDING.md).

## How to answer

For each question you can do **any** of the following:

1. **Pick an option:** change `- [ ]` to `- [x]`. Multi-select is allowed where noted.
2. **Give a custom answer:** write it on the `✍️ Custom answer:` line. This overrides the options.
3. **Ask me to clarify:** write your question on the `❓ Clarify:` line and I'll answer before you decide.
4. **Skip:** leave it blank and I'll go with my ⭐ recommendation (and note that in the spec).

When you're done, tell me **"answers ready"** and I'll process the file.

### Legend

| Mark | Meaning |
|---|---|
| ⭐⭐ | **Strongly recommended.** I'd push hard for this |
| ⭐ | Recommended / a good choice |
| ⚪ | Viable, depends on your goals |
| ⚠️ | Not recommended (risky or costly), but your call |

---

## A. Vision & Scope

### Q1. Product name
**Why it matters:** affects repo names, plugin namespaces and branding. It's cheap to decide now and expensive to change later.

- [x] **A. Atelier**: workshop where a master builds and teaches ⭐
- [ ] **B. Conclave**: swarm deliberation focus ⚪
- [ ] **C. Specwright**: spec-first craftsmanship ⚪
- [ ] **D. Keep "Harness" as a codename** and decide later ⭐ *(fine for now)*

✍️ Custom answer: Very cool name suggestion if there is no copyright claim and make me legally responsible I want that.
❓ Clarify:

---

### Q2. Who is the primary user for v1?
**Why it matters:** decides how much teaching, how much automation and how polished the UI needs to be.

- [x] **A. You (solo developer) first, others later.** Dogfood-driven ⭐⭐
- [ ] **B. Junior / learning developers.** Mentorship-first ⚪
- [ ] **C. Experienced engineers wanting rigor** (spec + swarm review) ⚪
- [ ] **D. Small teams** (shared specs, shared roadmap) ⚠️ *(adds multi-user sync, auth and conflicts too early)*

✍️ Custom answer:
❓ Clarify:

---

### Q3. What is the project's end goal?

- [x] **A. Personal tool / learning project** ⚪
- [x] **B. Open-source project** (core open, community plugins) ⭐
- [ ] **C. Commercial product** (closed or open-core) ⚪
- [ ] **D. Open-core:** OSS engine, paid cockpit/cloud features ⭐

✍️ Custom answer: My main goal is to build for myself but I am open to share it.
❓ Clarify:

---

### Q4. Which pillar must work first (MVP focus)? *(pick one)*
**Why it matters:** scope is the #1 risk. Everything else waits for this one.

- [ ] **A. Spec & Roadmap (P2)**, since everything downstream depends on specs ⭐⭐
- [ ] **B. Observer Swarm (P4)**: the most novel technical piece ⭐
- [ ] **C. Mentorship / Debrief (P5)** ⚪
- [ ] **D. IDE shell / GUI (P1)** ⚠️ *(high effort, low differentiation; others already have IDEs)*

✍️ Custom answer: I mean the project is about telling me the high level and also lower level explanation of the project in detail. It will show me a good markdown format and also a markdown viewer. I need a gui but also spec & roadmap as well. Because the main purpose of the project is understanding the code without reading everyline of code instead an agent provide a detailed explanation what the code does, what kind of architectural decisions made, which design patterns used, what directory stucture is used, which files has what purpose, which class does what and which method or function does what, how do we handle localization etc etc. I want to know those without having to look at the whole code. This is the upmost primary purpose, secondary purpose is spec driven development of course but also I need to view everything on gui as well so I am confused about my priorities. 
❓ Clarify:

---

### Q5. Main coding agent: wrap existing or build your own?
**Why it matters:** this is the biggest effort decision in the project.

- [x] **A. Wrap existing agents** (Claude Code via Agent SDK first, then Codex / Gemini CLI / OpenCode via ACP) ⭐⭐
- [ ] **B. Build our own minimal agent loop from day one** ⚠️ *(months of work that doesn't differentiate you)*
- [ ] **C. Wrap existing now, build our own later as an optional backend** ⭐
- [ ] **D. Agent-agnostic only:** we never run an agent, we just observe a terminal ⚠️ *(loses the structured control; see §4.1)*

✍️ Custom answer:
❓ Clarify:

---

## B. Platform & Architecture

### Q6. GUI foundation
- [ ] **A. Engine + CLI first → VS Code extension → Tauri cockpit later** ⭐⭐
- [x] **B. Tauri cockpit app directly** (your described layout, custom from day one) ⭐
- [ ] **C. Electron cockpit app** ⚪
- [ ] **D. Fork VS Code / Code-OSS** ⚠️ *(permanent upstream-merge burden)*
- [ ] **E. Qt** ⚠️

✍️ Custom answer:
❓ Clarify: I need a little bit clarification on this. Yes without engine and cli gui doesn't mean anything but also the gui is the main purpose as well. For now I picked the second choice.

---

### Q7. Engine language
- [ ] **A. TypeScript** (Node or Bun): one language with the extension, MCP and Agent SDK ⭐⭐
- [ ] **B. Python**: strong AI ecosystem, weaker for terminal and desktop integration ⚪
- [ ] **C. Rust**: fast, single binary, pairs with Tauri; slower to iterate ⚪
- [ ] **D. Mixed:** TS engine + Rust for Tauri/sandbox pieces later ⭐

✍️ Custom answer:
❓ Clarify: This needs a little clarification with advantages and disadvantages I kind of want a fast application but most of the time its io operation and the main bottleneck is the model speed but also there is some cli applications like codex they use rust. It kind of give me vibes that I should pick rust for engine but absolutely use the typescript for gui but I need your detailed reasons as well. This is a matter we need to discuss in detail.

---

### Q8. Target operating systems for v1 *(multi-select)*
- [x] **A. macOS** ⭐ *(you're on macOS; dogfood here first)*
- [ ] **B. Windows** ⚪
- [ ] **C. Linux** ⚪

✍️ Custom answer:
❓ Clarify: I am a mac user, it will primarily mac but I will use it also in the windows environment. So I want a cross paltform.

---

### Q9. How should the GUI send input to agents?
**Context:** §4.1 in the understanding doc.

- [x] **A. Structured protocol (SDK/ACP) + MCP `ask_user` tools; PTY injection only as fallback** ⭐⭐
- [ ] **B. PTY injection with bracketed paste for everything** ⚠️ *(fragile; no knowledge of TUI state)*
- [ ] **C. Both equally**: user chooses per agent ⚪

✍️ Custom answer:
❓ Clarify: I don't have enough knowledge but I used different sdks related to other kind of softwae and I like the sdk structure, but also I need more detail about what are those and why to use one on another also what other common harness providers like claude code, codex etc uses its important as well.

---

### Q10. Where should project knowledge live (specs, ADRs, docs, notebook)?
- [x] **A. In the repo under `.harness/`** (plain Markdown/YAML, Git-versioned) ⭐⭐
- [ ] **B. Local database only** (SQLite) ⚪
- [ ] **C. Repo files + SQLite index for search/costs** ⭐
- [ ] **D. Cloud backend** ⚠️ *(premature for v1)*

✍️ Custom answer:
❓ Clarify: I am in between the first and second selection second one also kind of make sense but I want to hear pros and cons and why would a database a bad idea also again what other harness providers do thats also important factor for me. For now I selected first but clarification needed.

---

### Q11. Sandboxing for agent commands and observer PoC tests
- [x] **A. Container (Docker/Podman) for PoC tests; OS-level permission prompts for the main agent** ⭐⭐
- [ ] **B. Everything in a container, including the main agent** ⭐ *(safest, more setup friction)*
- [ ] **C. No sandbox in v1; trust and review** ⚠️ *(observers run AI-generated exploit code)*

✍️ Custom answer:
❓ Clarify:

---

## C. Spec-Driven Development

### Q12. How strict is "spec freeze"?
- [x] **A. Freeze + formal amendments** (versioned, user-approved change requests) ⭐⭐
- [ ] **B. Absolute freeze**: any change means a new spec and a restarted phase ⚠️ *(blocks work whenever reality disagrees with the spec)*
- [ ] **C. Soft freeze**: agent may adjust minor details and log them ⚪

✍️ Custom answer:
❓ Clarify: First one sounds cool but I don't understand what the freeze and amendmendts mean. I want to clarify it and also what other harness providers do.

---

### Q13. Spec granularity
- [x] **A. One spec per feature** (e.g. `spec-003-auth.md`), plus a top-level product spec ⭐⭐
- [ ] **B. One big spec per project** ⚠️ *(hard to freeze/amend partially)*
- [ ] **C. One spec per roadmap phase** ⚪

✍️ Custom answer:
❓ Clarify:

---

### Q14. What happens to ideas that come up mid-implementation?
- [x] **A. Parking Lot**: captured, triaged after the current phase ⭐⭐
- [ ] **B. Agent decides whether to include them** ⚠️ *(this is exactly scope creep)*
- [ ] **C. Immediately prompt the user** ⚪ *(breaks flow)*

✍️ Custom answer:
❓ Clarify: I don't want scope creep I picked the first one but I need explanation about parking lot again what other harness do. (You know what from now on you should tell me how other harness or popular ides do certain things pros and cons about them as well)

---

## D. Observer Swarm

### Q15. How many observer models, and how chosen?
- [x] **A. 3–5 diverse models** (different vendors/families), weighted by measured precision ⭐⭐
- [ ] **B. As many as possible** ("the more the better") ⚠️ *(correlated errors, more false positives, more cost)*
- [ ] **C. User-configurable list, default 3** ⭐

✍️ Custom answer: I mean more than 3 may be a bit too much but I agree with the different model vendors should be used.
❓ Clarify:

---

### Q16. How do you want to access the cheap models?
- [x] **A. Through a gateway** (OpenRouter / LiteLLM): one key, many models ⭐⭐
- [x] **B. Direct provider APIs** (your own keys for each provider) ⚪
- [ ] **C. Gateway + optional local models** (Ollama / LM Studio) ⭐
- [ ] **D. Local models only** ⚪ *(free per call, but needs a strong machine and is usually lower quality)*

✍️ Custom answer: Yes gateway is primary one but I also want a couple of most common providers like opencode go, claude, codex etc if they do not pose threat to getting ban on their platform.
❓ Clarify:

---

### Q17. When does the swarm review run?
- [ ] **A. Per checkpoint (agent turn), async in the background** ⚪ *(thorough, costly)*
- [x] **B. Per completed task** (before you accept it) ⭐⭐
- [ ] **C. On demand only** (button) ⚪
- [ ] **D. B + C**: automatic per task, manual anytime ⭐

✍️ Custom answer: Per completed task makes much more sense, I thought it might be after every decision it took like change in a file or invoking a terminal command etc but per task is more cost efficient otherwise it could be more costly I kind of agree with you in this.
❓ Clarify:

---

### Q18. What power do observers have?
- [ ] **A. Advisory**: findings go to the main agent and you; nothing is blocked ⚪
- [x] **B. Advisory, but confirmed `high`/`critical` findings block task acceptance until fixed or overridden by you** ⭐⭐
- [ ] **C. Observers may auto-fix** ⚠️ *(two agents editing the same code; loses accountability)*

✍️ Custom answer:
❓ Clarify:

---

### Q19. Budget limits
- [x] **A. Hard cap per review + daily cap, with a live cost meter in the GUI** ⭐⭐
- [ ] **B. Soft warnings only** ⚪
- [ ] **C. No limits** ⚠️

If A or B, what rough daily budget is acceptable? (e.g. $1 / $5 / $20): More than 3 dolar definitely damages my wallet. Also I primarly want to connect something like open code go and use their models because I heard they take 10 dolar but provide almost like 60 dolar wort of things which is very cool but again even 3 dollar is too much for daily for me.

✍️ Custom answer:
❓ Clarify:

---

## E. Documentation

### Q20. Which docs are in v1 scope? *(multi-select)*
- [x] **A. ADRs (decisions + why), versioned** ⭐⭐
- [x] **B. Code map** (files / classes / methods and their purpose) ⭐
- [x] **C. High-level feature docs** (inputs/outputs) ⭐
- [ ] **D. API docs** with edge cases and examples ⚪
- [ ] **E. DB docs** (why this DB, tables, purposes) ⚪
- [ ] **F. UI/UX design-rationale docs** ⚪
- [ ] **G. Refactor proposals with strong-model validation** ⚪

✍️ Custom answer: Hmm all of them is appealing but I need an actual finished product, in the future I want all of them but currently First three ok and kind of the point.
❓ Clarify:

---

### Q21. How should versions be defined?
- [x] **A. SemVer tied to roadmap phases** (phase done → minor bump) ⭐⭐
- [ ] **B. Every accepted task = new version** ⚪ *(very granular)*
- [ ] **C. Manual: user tags versions** ⚪

✍️ Custom answer:
❓ Clarify:

---

## F. Mentorship

### Q22. Default mentorship mode
- [x] **A. Debrief** (work uninterrupted, review + quiz afterwards) ⭐⭐
- [ ] **B. Active** (step-by-step, waits for confirmation) ⚪
- [ ] **C. Silent** (no teaching; docs only) ⚪
- [ ] **D. Per-project setting, default Debrief** ⭐

✍️ Custom answer:
❓ Clarify: I need more clarification on the selections please but first one appealing default that please.

---

### Q23. Notebook & active recall
- [x] **A. Markdown cards in `.harness/notebook/` with hidden answers + spaced repetition (FSRS/SM-2)** ⭐⭐
- [ ] **B. Markdown cards, no scheduling** ⚪
- [ ] **C. Export to Anki as well** ⭐ *(nice add-on)*
- [ ] **D. Global notebook across all projects** (in addition to per-project) ⚪

✍️ Custom answer: I can include a feature like anki later so I don't want the normal anki please.
❓ Clarify:

---

### Q24. Should open-ended "why do you think…?" answers be graded?
- [x] **A. Yes: AI gives feedback against the recorded decision rationale (no numeric score)** ⭐⭐
- [ ] **B. Yes, with scores and progress tracking** ⚪
- [ ] **C. No: show the model answer only** ⚪

✍️ Custom answer: Also ai feedback will be recorded for the future use.
❓ Clarify:

---

## G. Feedback Overlay

### Q25. What is the floating feedback tool reporting on?
**Why it matters:** this is the part of the idea I'm least sure I understood.

- [x] **A. The app *being developed*** (user reports a bug in their own project → becomes a task for the agent) ⭐⭐
- [ ] **B. Our tool itself** (bug reports and feature requests for the IDE/harness) ⚪
- [ ] **C. Both**, with a target selector ⭐

✍️ Custom answer: Ok here is the idea when I test an application we are developing, especially with something with graphic like web, desktop, mobile or game. I want a floating feedback tool it will have a message section I will write my thought on it also for the additional context I will capture the screen (without showing the floating tool) and this will be additional contex but it will be optional no enforcement. I can also capture within a limited space like a within a rectangle capture on screen.
❓ Clarify:

---

### Q26. Where does a submitted report go?
- [x] **A. Into the Parking Lot / backlog, linked to the roadmap; triaged before the next phase** ⭐⭐
- [ ] **B. Straight to the main agent as a new task** ⚠️ *(bypasses spec discipline)*
- [ ] **C. GitHub Issues** ⚪
- [ ] **D. A + optional sync to GitHub Issues** ⭐

✍️ Custom answer:
❓ Clarify: I still don't know parking lot but I guess it is a backlog we don't touch wihtout reaching a specific version.

---

### Q27. Screenshot implementation
**Context:** §4.4 in the understanding doc.

- [x] **A. Hide → capture → show, then a frozen-frame snipping UI** (no native code) ⭐⭐
- [ ] **B. OS content-protection flag** (window excluded from captures) ⚪ *(verify on current macOS)*
- [ ] **C. Delegate to OS tools** (`screencapture -i`, Snipping Tool) ⚪ *(fastest MVP)*
- [ ] **D. Add app-aware capture for web apps** (Playwright: screenshot + DOM + console logs) ⭐ *(add-on)*

✍️ Custom answer: First one kind of make sense I was thinking how can we implement for all the os but that probably resolve that.
❓ Clarify:

---

## H. Extensibility

### Q28. Plugin model
- [x] **A. Plugins = MCP servers (agent tools) + optional UI manifest (panels/commands)**, each in its own repo ⭐⭐
- [ ] **B. Custom plugin API only** ⚠️ *(reinvents MCP; plugins won't work with other agents)*
- [ ] **C. No plugins in v1; design for them later** ⭐ *(keeps scope down)*

✍️ Custom answer:
❓ Clarify: I selected A but clarify please. I don't know how things work.

---

## I. Anything I Missed?

### Q29. Constraints I don't know about
Timeline, team size, budget, deadlines, must-use technologies, things you've already tried?

✍️ Custom answer: Timeline: No timeline, Team size: solo, Budget: definitely not much, Deadlines: Again no but why tommorrow, Must-use technologies: I believe in type safety but no must use ai will develop anyways, Things I tried: I didn't tried.

### Q30. Anything in [01-PROJECT-UNDERSTANDING.md](01-PROJECT-UNDERSTANDING.md) I got wrong?

✍️ Custom answer: I don't want first complete the cli then move to the gui type of thing they should go head to head as much as it can. I don't want to finish it to start gui when necessary parts complete start gui.

That feedback floating thing will be appear when I want it to be. Not from starting the harness.

I hate to look at simple text like markdown without formatting, or mermaid diagram with ascii etc we will make everything gui related as much as possible. I should be able to easily read everything and not confuse about it. We also should create a structure for this. Also those kind of sturctures probably has versions because if a version mismatch happens it will be ugly i don't know what you think.

Maybe for the later versions or current version I want a full gui application there will be no ugly terminal cli/tui etc just purely gui and that gui connected to most of the harness. But sometimes those harness changes things and its hard to keep up so there will be a backup structure which is the thing we currently decided (two window one terminal other for other things)

I guess we must prevent observers to edit files just be able to read them and only edit the .harness parts.

I also want a brainstorming tool, as well. Brainstorming cost too much token and it took a lot of token so we should create a structure to reliably brainstorm, like storing in somewhere in someway, summerizing for context decay removal also cost reduction.

For the mentorship part we can create something that remember what I learned so far not just for one particular project but the whole projects while I use this harness ide application. Also we may include roadmap (for learning skills) like in the roadmap.sh we can even take (if there is no legal responsibility) from roadmap.sh that roadmap trees and use it as our own. Also maybe a capable ai model can create a reliable skill based roadmaps like AI engineering, backend, devops, frontend roadmaps. What I learned so far also my progress acording to that roadmap.

For further discussion lets divide those conversations in to the versions but in to the different folders. 

For the later discussion please give me examples like what other favorite harness providers does pros and cons of each selection etc.