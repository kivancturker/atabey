# Original Project Idea (verbatim, session v0.1, 2026-10-07)

> This is the user's original idea exactly as it was pasted. It is the source document for the whole brainstorm. Do not edit it.

**Original title:** AI-Powered IDE & Multi-Agent Development Harness (Or do you have a better title proposition?)

---

I am designing a new AI-assisted software development tool/environment. It acts as a specialized IDE, a multi-agent harness, and a mentorship platform combined. Below are the detailed specifications, features, and technical concerns I have brainstormed. Please review this architecture, provide your feedback, suggest technical approaches (especially for my uncertainties), and help me expand on this design.

## 1. Core User Interface & Layout

* Split-Screen Design: The UI will feature a terminal on the right side and a feature/control panel (with buttons and tool interactors) on the left side.
* Foundation: I am considering forking or utilizing the open-source VS Code codebase to build this tool, as many modern IDE-like applications do. (Is this the most logical approach, or should I build a custom GUI?)
* Built-in Terminal & Harness: The center of the application will host a terminal capable of running TUI (Text User Interface) agent harnesses.
* Git Diff GUI: Any code written by the AI during a session will be visible through a Git Diff-style graphical interface, allowing the user to easily review historical changes.
* Markdown Previewer: A built-in Markdown tool is required. It will be read-only by default, but the user can toggle it to read/write mode.
* Floating Feedback/Capture Tool: A persistent, always-on-top overlay tool for the user to report bugs or request features. It will have a combobox (Bug, Feature, etc.), a description field, and an optional screenshot tool (fullscreen or snipping tool) that intentionally excludes our tool's UI from the capture. (I need ideas on how to implement this screenshot feature without writing heavy native OS code).

## 2. Spec-Driven Development & Roadmap GUI

* Strict Spec Definition: Specs must be perfectly finalized before any implementation begins to prevent scope creep and late backlog creation.
* Interactive Spec Generation: High-tier models (like Claude 3.5 Sonnet / Opus) will generate Markdown spec files in a specific format. The user will interact with these specs via a custom UI (answering multiple-choice questions or inputting text).
* Spec Editing: If the user dislikes a part of the generated spec, they can visually cross it out (strikethrough) and inject a detailed custom instruction.
* GUI-to-Terminal Input Stream: Instead of copying and pasting answers into the terminal, the user will submit their spec modifications via a GUI button. (Technical question: How easy is it to stream GUI inputs directly into standard TUI harnesses?)
* Visual Roadmap: The GUI will display a detailed project roadmap containing main phases and sub-phases. Clicking on any step will reveal detailed insights: what the phase is about, our deductions, and the "why" and "what" of the current task.

## 3. AI "Thought Process" & Documentation Generation

Similar to tools like Claude Code, the AI must transparently explain its decisions and coding structure as it works.

* Live Documentation: The tool will display the file structure and explain the purpose of specific files, classes, methods, and functions.
* Versioning: All architectural decisions will be tracked and attached to specific version numbers, explaining the goal of each version.
* Dual-Level Detail:
   * High-Level: Documentation on features, expected inputs, and outputs.
   * Low-Level (For Engineers): Explanations of design patterns used, file architecture reasoning, and class/method responsibilities.
* API & DB Docs: Automatic generation of API documentation (with edge cases/example outputs) and Database documentation (why a specific DB was chosen, table structures, and their purposes).
* UI/UX Docs: Documentation detailing the reasoning behind design choices and methodologies.
* Refactoring: The system can suggest refactoring and defend its reasoning. Before beginning a complex refactor, it will consult a highly capable model for validation.

## 4. Multi-Agent & Sub-Agent Architecture

Relying on a single agent to write code and explain everything causes delays. We will use a multi-agent orchestration system (potentially via MCP - Model Context Protocol).

* Mini Custom Harness: We will build a custom harness, starting TUI/CLI-based, eventually wrapping it in a GUI. Features will be scalable, existing as separate GitHub repos acting as extensions/plugins.
* Supervisory "Swarm" of Cheap LLMs: We will utilize cheaper, fast models (e.g., GLM 4/5 Flash, Qwen Flash, DeepSeek Flash) connected via API as an observer/supervisor swarm. The more models, the better.
* Cross-Evaluation Protocol:
   1. Each observer model analyzes the written code for problems and notes its deductions.
   2. Contexts are cleared.
   3. The models review each other's deductions. They debate if someone missed a point, if a deduction is flawed, or create mock test scenarios to prove/disprove a vulnerability (similar to opensec.in logic).
* Context Window Hijacking Prevention: To prevent token bloat, the main agent will not pass the entire chat history to the Observer agents. It will only pass Git Diffs and AST (Abstract Syntax Tree) summaries.

## 5. Learning & Mentorship Mode

The tool will act as an interactive mentor, but it must be carefully designed to avoid "Cognitive Overload" (breaking the developer's flow state with constant pop-ups).

* Dedicated Question UI: When the AI has a question for the user, it appears in a specialized UI (multiple choice or custom input).
* Mentorship Features: The AI will explain why it made a decision, ask the user why they think a decision was made (quiz style), and provide similar solutions for different edge cases.
* AI Notebook: The AI stores learnings, notes, and examples in a personal notebook for the user, with the ability to hide answers for active recall study.
* Selectable Modes:
   * Active Mode: Step-by-step execution. The AI asks educational questions and waits for user confirmation before proceeding.
   * Debrief Mode (Recommended): The AI works uninterrupted. Once the session or feature is complete, it triggers a "Code Review & Quiz" post-mortem session to teach the user what it just did.

Questions for the AI Brainstorming Partner: Based on the architecture above, what are the biggest technical bottlenecks? How should I handle the GUI-to-TUI input stream problem? Is VS Code the right base, or should I use Tauri/Electron/Qt? How would you design the multi-agent consensus loop to prevent infinite loops?
