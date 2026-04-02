# Claw Code

<p align="center">
  <img src="assets/clawd-hero.jpeg" alt="Claw Code" width="300" />
</p>

<p align="center">
  <strong>A Rust coding-agent CLI with multi-provider support, local tooling, and Claw branding.</strong>
</p>

<p align="center">
  <a href="https://github.com/ORION2809/claw-code"><img src="https://img.shields.io/badge/GitHub-ORION2809%2Fclaw--code-181717?logo=github&style=for-the-badge" alt="GitHub" /></a>
</p>

> [!IMPORTANT]
> This fork tracks the newer Rust workspace shape from upstream while preserving the local multi-provider stack, `claw` branding, richer hook and permission features, and plugin-aware runtime behavior.

## What this fork adds

- Multi-provider model access through the Rust `api` crate, including Claude-compatible, Grok, and OpenAI-compatible flows
- `claw`-specific branding and config paths such as the `claw` binary, `CLAW_*` variables, and `.claw` project files
- Local runtime extensions for permission rules, hook progress and abort handling, auto-compaction, and plugin lifecycle management
- The newer upstream workspace surface, including the `lsp` and `server` crates plus the expanded CLI and slash-command layers

## Quick Start

```bash
cd rust
cargo build --release

./target/release/claw
./target/release/claw prompt "explain this codebase"
./target/release/claw --model sonnet prompt "review the latest changes"
```

OAuth login is also available:

```bash
./target/release/claw login
```

## Provider configuration

Anthropic-compatible:

```bash
export ANTHROPIC_API_KEY="..."
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
```

Grok:

```bash
export XAI_API_KEY="..."
export XAI_BASE_URL="https://api.x.ai"
```

OpenAI-compatible:

```bash
export OPENAI_API_KEY="..."
export OPENAI_BASE_URL="https://api.openai.com/v1"
```

## Repository layout

```text
.
|-- rust/
|   |-- Cargo.toml
|   |-- docs/releases/0.1.0.md
|   `-- crates/
|       |-- api/              # Provider clients, auth, SSE streaming
|       |-- claw-cli/         # Main CLI binary (`claw`)
|       |-- commands/         # Slash commands and command discovery
|       |-- compat-harness/   # Compatibility tooling
|       |-- lsp/              # LSP support helpers and types
|       |-- plugins/          # Plugin discovery, registry, lifecycle
|       |-- runtime/          # Session loop, config, permissions, prompts, MCP
|       |-- server/           # HTTP and SSE server surface
|       `-- tools/            # Built-in tool implementations
|-- src/                      # Python analysis and parity workspace
|-- tests/                    # Python verification
|-- CLAW.md                   # Project instruction and memory file
`-- README.md
```

## Feature snapshot

| Feature | Status |
|---------|--------|
| Multi-provider API support | Yes |
| OAuth login and logout | Yes |
| Interactive REPL and one-shot prompts | Yes |
| Slash commands and resume-safe commands | Yes |
| Built-in tools plus plugin tool registry | Yes |
| Hook progress, abort handling, and post-failure hooks | Yes |
| Context-aware permission rules | Yes |
| Auto-compaction and session persistence | Yes |
| LSP and server crates from the newer Rust workspace | Yes |

## Notes

- The Rust workspace is the primary product surface in this repository.
- The Python `src/` tree remains useful for local analysis, parity tracking, and supporting research workflows.
- Release notes for the current Rust workspace live at `rust/docs/releases/0.1.0.md`.

## Acknowledgements

This repository is derived from [instructkr/claw-code](https://github.com/instructkr/claw-code). The local rewrite, merge work, and surrounding workflow automation were developed with heavy AI-assisted iteration.

## Ownership and affiliation disclaimer

- Maintained by [ORION2809](https://github.com/ORION2809)
- Not affiliated with, endorsed by, or maintained by Anthropic
- Does not claim ownership of the original Claude Code source material
