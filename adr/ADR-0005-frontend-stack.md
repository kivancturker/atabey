---
id: ADR-0005
title: Frontend stack
status: accepted
date: 2026-10-10
decisions: [D-015, D-091, D-123]
spec: specs/S0-foundations.md §5.1, §5.7
---

# ADR-0005: Frontend stack

## Context
The GUI must be themeable from day one (D-091), strictly typed (Art. 1), and easy for coding agents to write correctly.

## Decision

| Concern | Choice (version on 2026-10-10) |
|---|---|
| Framework | React 19.3 + TypeScript 7.0 (strict) |
| Build | Vite 8.3 |
| Styling | Tailwind CSS 4.3 **mapped to semantic token CSS variables**; Radix primitives for accessible widgets |
| Routing / server state | TanStack Router 1.170 / TanStack Query 5.104 |
| UI state | Zustand 5.0 |
| Diagrams / code | Mermaid 12.1 (strict security level) / Shiki 4.5 |
| Lint + format | Biome 2.5 |
| Tests | Vitest 5 + Testing Library; E2E WebdriverIO + `@wdio/tauri-service` |
| Package manager | pnpm 12 workspaces |

## Alternatives considered

| Option | Pros | Cons |
|---|---|---|
| CSS Modules instead of Tailwind | Explicit, no utility classes | More files; agents are less fluent |
| A full component kit (MUI, Chakra) | Many components | Fights the semantic-token theme system |
| ESLint + Prettier | Most plugins | Slower; two tools and more config |

## Consequences
- **No literal colors** outside `themes/` (CI check, R-19).
- If TypeScript 7's native compiler breaks a tool, fall back to TS 5.9 `tsc` for type-checking only (S0 risk table).
