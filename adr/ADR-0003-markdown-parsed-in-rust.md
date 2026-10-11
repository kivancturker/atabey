---
id: ADR-0003
title: Markdown is parsed in Rust (comrak)
status: accepted
date: 2026-10-10
decisions: [D-023, D-122]
spec: specs/S0-foundations.md R-10..R-17, §5.6
---

# ADR-0003: Markdown is parsed in Rust (comrak)

## Context
Atabey documents are *data* (decisions, questions, findings) as well as pages. The core must index them (SQLite), validate typed `atabey:*` blocks against schemas generated from Rust types, detect question cards, and serve the same structure to the CLI and later to agents.

## Decision
Parse Markdown + YAML frontmatter **in Rust** with **comrak 0.56** (GFM) and **serde-saphyr 1.3** (`serde_yaml` is deprecated) into a typed `Document` AST. The GUI renders the AST with React components; Mermaid and Shiki run in the GUI.

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| **Rust (comrak)** | One parser for rendering, indexing, validation, CLI | Fewer ready-made plugins; AST crosses IPC |
| TS (unified/remark) | Huge plugin ecosystem | A second parser in Rust would still be needed for indexing → two parsers that can disagree |

## Consequences
- The golden corpus (all of `docs/brainstorm/`) is the regression test for the parser (AC-6).
- Raw HTML is escaped (R-17), so no HTML sanitizer is needed in S0.
