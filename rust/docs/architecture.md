# Rust Architecture

## Workspace shape

- `api`: provider-specific request/response handling and streaming clients
- `runtime`: conversation loop, permissions, hooks, MCP config, sessions, sandbox support
- `claw-cli`: entrypoints, REPL UX, rendering, session management, output formatting
- `commands`: slash-command registry, parsing, usage text, helper reports
- `tools`: built-in tool definitions and execution paths
- `plugins`: plugin manifest validation, registry, lifecycle hooks, plugin tools
- `lsp`: language-server client support and shared types
- `server`: HTTP and SSE serving surface
- `compat-harness`: workspace manifest extraction and compatibility helpers

## Main flow

1. `claw-cli` parses CLI arguments into `CliAction`.
2. `LiveCli` builds a `ConversationRuntime` with the current model, permissions, and tool registry.
3. `runtime` orchestrates assistant turns, tool calls, hook execution, and session updates.
4. `tools` handles built-in tools and plugin/MCP-integrated surfaces.
5. `commands` owns slash-command metadata and non-REPL helper reports.

## Extension points

- Add tools in `crates/tools/src/lib.rs`.
- Add slash commands in `crates/commands/src/lib.rs` and wire REPL behavior in `crates/claw-cli/src/app.rs`.
- Add runtime capabilities in `crates/runtime/` behind typed configuration and tests.
