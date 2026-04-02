# Claw Code Rust Workspace

This directory contains the primary Rust implementation for the `claw` CLI. It is the main product surface for the repository and includes the merged upstream workspace shape plus the local multi-provider and runtime extensions.

## Build and run

```bash
cargo build --release

./target/release/claw
./target/release/claw prompt "explain crates/runtime"
./target/release/claw --model sonnet "review the latest changes"
./target/release/claw login
```

You can also install the binary locally:

```bash
cargo install --path crates/claw-cli --locked
```

## Provider setup

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

## Workspace crates

```text
rust/
|-- Cargo.toml
|-- Cargo.lock
|-- docs/releases/0.1.0.md
`-- crates/
    |-- api/              # Provider clients, auth, streaming
    |-- claw-cli/         # User-facing `claw` binary
    |-- commands/         # Slash command registry and helpers
    |-- compat-harness/   # Compatibility tooling
    |-- lsp/              # LSP support helpers and types
    |-- plugins/          # Plugin discovery, registry, lifecycle
    |-- runtime/          # Sessions, config, permissions, prompts, MCP
    |-- server/           # HTTP and SSE server surface
    `-- tools/            # Built-in tool implementations
```

## Crate responsibilities

- `api`: provider selection, authentication, request and response types, SSE streaming
- `claw-cli`: CLI parsing, REPL loop, one-shot prompt mode, session UX, rendering
- `commands`: slash command definitions, help rendering, command suggestions
- `compat-harness`: compatibility support for upstream manifest extraction
- `lsp`: language-server support helpers and shared types
- `plugins`: plugin discovery, registry handling, lifecycle management
- `runtime`: conversation loop, config loading, permissions, hooks, prompts, MCP, usage, sessions
- `server`: HTTP and SSE serving surface for integration use cases
- `tools`: built-in tool specs and execution for shell, file, search, web, agent, todo, notebook, and skill flows

## Current capabilities

- Interactive REPL and non-interactive prompt execution
- Resume-safe slash commands and saved session inspection
- Multi-provider model support and OAuth login
- Built-in tool execution with plugin-aware registration
- Hook progress, abort, and failure reporting
- Context-aware permission rules
- Auto-compaction, usage reporting, and session persistence
- Workspace-aware instruction loading through `CLAW.md`

## Notes

- This workspace currently builds from source; packaged public releases are still limited.
- Release notes for the current workspace live in `docs/releases/0.1.0.md`.
- The default binary name is `claw`.

## Maintainer

Maintained by [ORION2809](https://github.com/ORION2809). Derived from [instructkr/claw-code](https://github.com/instructkr/claw-code).
