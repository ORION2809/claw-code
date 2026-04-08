# Command Authoring Guide

## Add a slash command

1. Add the command spec in `crates/commands/src/lib.rs`.
2. Extend `SlashCommand::parse` and `SlashCommand::name`.
3. Update any direct-CLI canonicalization in `crates/claw-cli/src/args.rs`.
4. Wire user-visible behavior in `crates/claw-cli/src/app.rs`.
5. Add or update tests in both `commands` and `claw-cli` when parsing/help behavior changes.

## Design rules

- Keep the slash-command registry as the source of truth for discoverability.
- Treat aliases as first-class when users are likely to type them.
- Prefer concise reports with stable labels so tests stay readable.
- If a command is REPL-only, make that explicit in direct CLI errors.

## Verification

- Run `cargo test -p commands -- --test-threads=1`.
- If REPL handling changed, also run `cargo test -p claw-cli --bin claw -- --test-threads=1`.
