# Contributing

## Local workflow

1. Work from the `rust/` workspace unless a change is explicitly outside the Rust implementation.
2. Run `cargo fmt --check` before sending changes.
3. Run `cargo test --workspace -- --test-threads=1` before sending changes.
4. Keep command/tool additions covered by focused tests in the crate you changed.

## CLI artifacts

- Generate shell completions with `cargo run -p claw-cli -- completions <shell> --output <path>`.
- Generate the man page with `cargo run -p claw-cli -- manpage --output <path>`.

## Architecture docs

- [Rust architecture](rust/docs/architecture.md)
- [Tool authoring guide](rust/docs/tool-authoring.md)
- [Command authoring guide](rust/docs/command-authoring.md)

## Pull requests

- Keep changes scoped to one feature family when possible.
- Call out user-facing behavior changes in the PR description.
- Mention any intentionally ignored tests or environment-dependent checks.
