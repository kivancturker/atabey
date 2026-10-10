# Atabey

> *Atabey: the Seljuk guardian-tutor of princes.*

A desktop app (Tauri 2 · Rust · React/TypeScript) that **explains any codebase in a rich GUI**, **builds features through spec-driven, multi-agent development**, and **teaches you while it does**.

**Status:** early development. Slice **S0 · Foundations** is being built ([spec](specs/S0-foundations.md), [roadmap](ROADMAP.md)). Nothing is usable yet.

| Document | What it is |
|---|---|
| [PRODUCT-SPEC.md](PRODUCT-SPEC.md) | Vision, pillars, scope (frozen v1.1.0) |
| [CONSTITUTION.md](CONSTITUTION.md) | Non-negotiable engineering rules (frozen v1.1.0) |
| [ROADMAP.md](ROADMAP.md) | Slices S0 → S9 = v1.0 |
| [specs/](specs/) | One frozen spec per slice |
| [adr/](adr/) | Architecture decision records |
| [docs/brainstorm/](docs/brainstorm/README.md) | How every decision was made (5 design sessions + decision ledger) |
| [AGENTS.md](AGENTS.md) | How coding agents (and humans) work on this repo |

## Development setup

Requires [mise](https://mise.jdx.dev) and rustup.

```bash
mise install
```

```bash
pnpm install
```

```bash
mise run check
```

## License

[Apache-2.0](LICENSE)
