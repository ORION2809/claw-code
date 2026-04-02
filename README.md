# 🦞 Claw Code

<p align="center">
  <img src="assets/clawd-hero.jpeg" alt="Claw Code" width="300" />
</p>

<p align="center">
  <strong>A high-performance, multi-provider CLI agent harness — built in Rust</strong>
</p>

<p align="center">
  <a href="https://github.com/ORION2809/claw-code"><img src="https://img.shields.io/badge/GitHub-ORION2809%2Fclaw--code-181717?logo=github&style=for-the-badge" alt="GitHub" /></a>
</p>

> [!IMPORTANT]
> **This fork has been upgraded with a superior merge** that combines the latest upstream Rust improvements with multi-provider API support, enhanced plugin infrastructure, and advanced permission/hook systems. See the [Upgrade Details](#upgrade-details) section below.

---

## Upgrade Details

This repository has been upgraded from the original [instructkr/claw-code](https://github.com/instructkr/claw-code) with a comprehensive **superior merge** that ports all improvements from the `dev/rust` branch while preserving and extending the unique multi-provider capabilities of the `main` branch.

### What was upgraded

| Area | Changes |
|------|---------|
| **Hook System** | Complete rewrite (~280 → ~860 lines). Added `HookAbortSignal`, `HookProgressReporter`, `PostToolUseFailure` hooks, JSON-parsed hook output, abort signal polling, progress event reporting |
| **Permission System** | Added `PermissionOverride`, `PermissionContext`, `PermissionRule` with rule-based matching. Rewrote `authorize_with_context` for context-aware authorization |
| **Auto-Compaction** | Automatic conversation compaction at 200K token threshold with `AutoCompactionEvent` reporting |
| **Plugin Lifecycle** | Full `PluginRegistry` integration, `new_with_plugins()` constructor, `shutdown_registered_plugins()`, `Drop`-based cleanup |
| **Tool System** | `RegisteredTool` abstraction with `RegisteredToolHandler` (Builtin/Plugin), unified `GlobalToolRegistry`, `normalize_registry_tool_name` |
| **CLI Integration** | `HookAbortMonitor` (Ctrl+C → abort signal), `CliHookProgressReporter`, `prepare_turn_runtime()` pattern, auto-compaction notices |
| **Configuration** | `RuntimePermissionRuleConfig`, permission rules in config loader, `post_tool_use_failure` hook config |

### What was preserved (multi-provider superiority)

- **Multi-provider API** — `ProviderClient` enum supporting Claude, Grok, and OpenAI-compatible endpoints
- **OAuth authentication** — Full OAuth login/logout flow
- **Claw Code branding** — `claw` binary, `CLAW_` environment variables, `.claw` config paths
- **Commands crate** — Multi-provider model discovery, legacy command support, help text generation
- **Model aliases** — `opus`, `sonnet`, `haiku` resolving to latest versions across providers

### Files modified

- `rust/crates/runtime/src/config.rs` — Permission rule config, hook config extensions
- `rust/crates/runtime/src/permissions.rs` — Context-aware authorization, permission rules
- `rust/crates/runtime/src/hooks.rs` — Complete hook pipeline rewrite
- `rust/crates/runtime/src/lib.rs` — Updated public API exports
- `rust/crates/runtime/src/conversation.rs` — Auto-compaction, plugin lifecycle, hook integration
- `rust/crates/tools/src/lib.rs` — RegisteredTool, unified tool registry
- `rust/crates/claw-cli/src/main.rs` — Abort monitor, progress reporter, runtime construction

---

## Quick Start

```bash
# Build
cd rust/
cargo build --release

# Run interactive REPL
./target/release/claw

# One-shot prompt
./target/release/claw prompt "explain this codebase"

# With specific model
./target/release/claw --model sonnet prompt "fix the bug in main.rs"
```

## Configuration

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# Or use a proxy / OpenAI-compatible endpoint
export ANTHROPIC_BASE_URL="https://your-proxy.com"
```

Or authenticate via OAuth:

```bash
claw login
```

## Repository Layout

```text
.
├── rust/                               # Rust workspace (primary implementation)
│   ├── crates/
│   │   ├── api/                        # Multi-provider API client + SSE streaming
│   │   ├── claw-cli/                   # Main CLI binary (claw)
│   │   ├── commands/                   # Slash command registry
│   │   ├── compat-harness/             # TS manifest extraction
│   │   ├── plugins/                    # Plugin system
│   │   ├── runtime/                    # Conversation runtime, config, permissions, hooks
│   │   └── tools/                      # Built-in tool implementations
│   └── Cargo.toml
├── src/                                # Python porting workspace (reference)
├── tests/                              # Python verification
└── README.md
```

## Features

| Feature | Status |
|---------|--------|
| Multi-provider API (Claude, Grok, OpenAI) | ✅ |
| API + SSE streaming | ✅ |
| OAuth login/logout | ✅ |
| Interactive REPL (rustyline) | ✅ |
| Tool system (bash, read, write, edit, grep, glob) | ✅ |
| Web tools (search, fetch) | ✅ |
| Sub-agent orchestration | ✅ |
| Hook system (Pre/Post ToolUse + abort signals) | ✅ |
| Hook progress reporting | ✅ |
| Permission rules (context-aware authorization) | ✅ |
| Plugin system with registry | ✅ |
| Auto-compaction (200K token threshold) | ✅ |
| RegisteredTool abstraction (Builtin + Plugin) | ✅ |
| CLAW.md / project memory | ✅ |
| Config file hierarchy (.claw.json) | ✅ |
| Session persistence + resume | ✅ |
| Extended thinking (thinking blocks) | ✅ |
| Cost tracking + usage display | ✅ |
| Git integration | ✅ |
| Markdown terminal rendering (ANSI) | ✅ |
| Model aliases (opus/sonnet/haiku) | ✅ |
| Slash commands (/status, /compact, /clear, etc.) | ✅ |
| Skills registry | ✅ |

## Acknowledgements

This project is forked from [instructkr/claw-code](https://github.com/instructkr/claw-code), originally created by Sigrid Jin ([@instructkr](https://github.com/instructkr)). The original Python rewrite and Rust port were orchestrated using [oh-my-codex (OmX)](https://github.com/Yeachan-Heo/oh-my-codex).

## Ownership / Affiliation Disclaimer

- This repository is maintained by [ORION2809](https://github.com/ORION2809).
- This repository does **not** claim ownership of the original Claude Code source material.
- This repository is **not affiliated with, endorsed by, or maintained by Anthropic**.
