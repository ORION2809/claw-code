# Tool Authoring Guide

## Add a built-in tool

1. Add a `ToolSpec` entry in `crates/tools/src/lib.rs`.
2. Define the input shape as a typed Rust struct with `serde` support.
3. Add an execution branch in `execute_tool`.
4. Keep permission requirements explicit through `PermissionMode`.
5. Add unit tests for schema shape, happy path, and failure behavior.

## Design rules

- Prefer deterministic, typed JSON input over ad hoc string parsing.
- Keep tool outputs concise and machine-readable where practical.
- Return clear errors that can be surfaced directly to users.
- Respect workspace boundaries and existing permission checks.

## Verification

- Run `cargo test -p tools -- --test-threads=1`.
- If the tool is surfaced in the REPL or slash commands, run the relevant crate tests too.
