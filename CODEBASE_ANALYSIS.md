# Claw Code — Deep Codebase Analysis

## 1. Project Overview

**Claw Code** is a clean-room reimplementation of the Claude Code agent harness — Anthropic's CLI tool for agentic AI coding. The project was born out of the Claude Code source exposure incident on March 31, 2026 and aims to rebuild the core harness patterns without copying proprietary code.

The repository contains **two parallel codebases**:

| Layer | Language | Purpose | Maturity |
|-------|----------|---------|----------|
| `src/` | Python | Reference scaffolding, registry mirroring, parity tracking | Feature-complete scaffolding |
| `rust/` | Rust | Production runtime, CLI, and agentic loop | MVP — actively developed |

The Python tree captures *what* the original system does (207 commands, 184 tools, 30 subsystems) via JSON snapshots and metadata. The Rust tree is the *real implementation* — a working CLI with streaming LLM conversations, tool execution, and session management.

---

## 2. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      USER (terminal)                            │
└───────────────────────────┬─────────────────────────────────────┘
                            │
                   ┌────────▼────────┐
                   │   claw-cli      │  ← REPL / one-shot / JSON mode
                   │  (Rust binary)  │
                   └──┬──┬──┬──┬────┘
                      │  │  │  │
        ┌─────────────┘  │  │  └──────────────┐
        ▼                ▼  ▼                  ▼
  ┌──────────┐   ┌────────┐ ┌──────────┐  ┌──────────┐
  │ commands │   │  api   │ │ runtime  │  │  tools   │
  │ (slash)  │   │(LLM)  │ │ (engine) │  │ (18 ops) │
  └──────────┘   └────────┘ └────┬─────┘  └──────────┘
                                 │
                          ┌──────▼──────┐
                          │   plugins   │
                          │  (hooks)    │
                          └─────────────┘
```

```
┌─────────────────────────────────────────────────────────────────┐
│                  Python scaffolding (src/)                       │
│                                                                 │
│   207 commands ◄── commands_snapshot.json                       │
│   184 tools    ◄── tools_snapshot.json                          │
│   30 subsystem packages ◄── subsystems/*.json                   │
│   Parity audit engine (parity_audit.py)                         │
│   Session/routing/bootstrap simulation                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Rust Workspace (~20K lines)

The Rust workspace at `rust/` is organized as **6 crates** in a modular monolith. Unsafe code is forbidden workspace-wide.

### 3.1 Crate Dependency Graph

```
claw-cli (binary)
  ├── api              (HTTP + SSE streaming to LLM providers)
  ├── runtime          (agentic conversation loop + config + permissions)
  ├── tools            (18 built-in tool implementations)
  ├── commands         (slash command registry)
  ├── plugins          (plugin manifest + hook system)
  └── compat-harness   (TS manifest extraction for parity checking)
```

### 3.2 Crate Breakdown

#### `api` — LLM Provider Abstraction (~1,500 lines)

Multi-provider HTTP client with SSE streaming support.

- **Providers**: Anthropic Claude (opus/sonnet/haiku), Xai Grok, OpenAI
- **Auth**: API key + OAuth with PKCE token refresh
- **Key types**: `ProviderClient`, `MessageStream`, `StreamEvent` (TextDelta, ToolUse, Usage, MessageStop)
- **Pattern**: `ProviderClient::from_model("opus")` factory — model alias resolution (e.g. `opus` → `claude-opus-4-6`)
- **Dependencies**: reqwest, tokio, serde

#### `runtime` — Core Agentic Engine (~5,300 lines)

The heart of the system. Orchestrates the conversation loop, permissions, sessions, and config.

- **`ConversationRuntime<C, T>`**: Generic over `ApiClient` + `ToolExecutor`. Implements the main loop:
  1. Stream API request
  2. Parse assistant response + tool use blocks
  3. Permission-check each tool
  4. Execute tools (with pre/post hooks)
  5. Feed results back → repeat until stop
- **`Session`**: Timestamped conversation state, persisted to `~/.claw/sessions/`
- **`ConfigLoader`**: Hierarchical config merge (workspace `.claw.json` → `~/.claude/` → env vars → CLI flags)
- **`PermissionPolicy`**: Three modes — read-only, workspace-write, danger-full-access
- **`SystemPromptBuilder`**: Loads CLAUDE.md + project context + system instructions
- **MCP support**: Stdio, SSE, WebSocket, SDK, managed proxy transports
- **Session compaction**: Summarizes old turns to manage token budgets

#### `claw-cli` — Terminal Interface (~3,600 lines)

User-facing REPL, one-shot CLI, and markdown rendering.

- **`LiveCli`**: Main app — holds runtime, config, REPL state
- **Modes**: Interactive REPL, one-shot prompt, JSON output
- **Rendering**: Markdown → terminal with ANSI colors, syntax-highlighted code blocks (syntect), box-drawn tables, tool result panels
- **Line editing**: rustyline with tab completion for `/` commands, shift+enter for multiline
- **Session management**: resume, list, switch sessions
- **Known debt**: `main.rs` is 3,159 lines (monolith) — a refactoring plan exists in `TUI-ENHANCEMENT-PLAN.md`

#### `commands` — Slash Command Registry (~470 lines)

Static metadata for 15+ slash commands.

| Command | Purpose |
|---------|---------|
| `/help` | Show all commands |
| `/status` | Session state (model, tokens, cost) |
| `/cost` | Cumulative token cost |
| `/compact` | Compress session history |
| `/clear` | Fresh session |
| `/model [name]` | Show/switch model |
| `/permissions [mode]` | Show/switch permission mode |
| `/config [section]` | Display configuration |
| `/memory` | Show CLAUDE.md |
| `/diff` | Git diff |
| `/export [path]` | Export conversation JSON |
| `/session list\|switch\|resume` | Manage sessions |
| `/agents` | List agents |
| `/skills` | List skills |
| `/plugins` | List plugins |

#### `tools` — Built-in Tool Implementations (~3,500 lines)

18 tools across 5 categories, each with JSON Schema input definitions:

| Category | Tools | Permission |
|----------|-------|-----------|
| **File** | ReadFile, WriteFile, EditFile | read-only / workspace-write |
| **Search** | GlobSearch, GrepSearch | read-only |
| **Execution** | Bash, Repl | danger-full-access |
| **Web** | WebSearch, WebFetch | read-only |
| **Task/Agent** | Agent, TodoWrite, NotebookEdit | varies |
| **Reference** | Skill, ToolSearch | read-only |

#### `plugins` — Extension System (~500 lines)

Plugin infrastructure for third-party extensibility.

- **Manifest-driven**: `plugin.json` declares tools, commands, hooks, permissions, lifecycle events
- **Hook system**: `PreToolUse` can deny tool execution; `PostToolUse` can modify results
- **Three sources**: builtin (shipped), bundled (separate version), external (user-installed)
- **Execution**: Plugins run as separate processes via `Command::new()` with stdin/stdout JSON I/O

#### `compat-harness` — Parity Checking (~200 lines)

Extracts command/tool manifests from upstream TypeScript source via regex parsing. Ensures the Rust CLI exposes the same surfaces as the reference implementation.

### 3.3 Key External Dependencies

| Category | Crate | Purpose |
|----------|-------|---------|
| Async | tokio | Async runtime |
| HTTP | reqwest | API client |
| Serialization | serde, serde_json | JSON types |
| Terminal | rustyline | REPL + history |
| Terminal | crossterm | ANSI control |
| Markdown | pulldown-cmark | MD parsing |
| Highlighting | syntect | Syntax colors |
| Filesystem | glob, walkdir | File search |
| Crypto | sha2 | Config hashing |
| Text | regex | Search |

---

## 4. Python Scaffolding (~60+ modules)

The Python tree (`src/`) is **not a full implementation** — it's a reference scaffold that mirrors the original system's registries and provides introspection tooling.

### 4.1 Core Modules

| Module | Role |
|--------|------|
| `main.py` | CLI dispatcher with 20+ subcommands |
| `runtime.py` | `PortRuntime` — routes prompts, bootstraps sessions |
| `query_engine.py` | `QueryEnginePort` — conversation state, turn budgets |
| `commands.py` | Loads 207 commands from `commands_snapshot.json` |
| `tools.py` | Loads 184 tools from `tools_snapshot.json` |
| `execution_registry.py` | Wraps commands/tools as executable `Mirrored*` instances |
| `session_store.py` | JSON session persistence in `.port_sessions/` |
| `models.py` | Frozen dataclasses: `Subsystem`, `PortingModule`, `UsageSummary` |
| `permissions.py` | Tool deny-listing by name or prefix |
| `parity_audit.py` | Coverage comparison against archived TypeScript |
| `bootstrap_graph.py` | 7-stage startup pipeline simulation |

### 4.2 Subsystem Packages (30 packages)

All are **placeholder packages** that expose metadata from `reference_data/subsystems/{name}.json`:

```
Agent systems:      assistant, buddy, coordinator
Execution:          bootstrap, bridge, entrypoints, remote, upstreamproxy
Config/state:       constants, schemas, state, keybindings
UI/output:          components, outputStyles, screens
Extensibility:      hooks, plugins, skills
Integration:        cli, services, types, utils
Special modes:      memdir, moreright, native_ts, vim, voice
```

Each package exports: `ARCHIVE_NAME`, `MODULE_COUNT`, `SAMPLE_FILES`, `PORTING_NOTE`.

### 4.3 Reference Data Snapshots

| File | Entries | Content |
|------|---------|---------|
| `commands_snapshot.json` | 207 | Full command registry (name, hints, responsibilities) |
| `tools_snapshot.json` | 184 | Full tool registry (name, schema, permission, family) |
| `archive_surface_snapshot.json` | 1,902 files | Original TS file tree structure |

### 4.4 Data Flow

```
User Prompt
   ↓
PortRuntime.route_prompt(prompt)
   ├── Tokenize prompt
   ├── Score against command/tool names + hints
   └── Return top N RoutedMatch objects
   ↓
PortRuntime.bootstrap_session(prompt)
   ├── Build PortContext (workspace metadata)
   ├── Run SetupReport (platform, startup steps)
   ├── Create QueryEnginePort session
   ├── Execute matched commands/tools via ExecutionRegistry
   └── Return RuntimeSession with history + results
   ↓
QueryEnginePort.submit_message(prompt, commands, tools)
   ├── Check max turns / budget limits
   ├── Track usage projections
   ├── Store in transcript
   └── Return TurnResult
   ↓
Session persisted to .port_sessions/{session_id}.json
```

---

## 5. Parity Gap Analysis

The Rust port has a solid foundation but is **not at feature parity** with the original TypeScript system.

### 5.1 What Rust Has

| Feature | Status |
|---------|--------|
| Anthropic API + OAuth | Working |
| Multi-provider (Claude, Grok, OpenAI) | Working |
| SSE streaming | Working |
| Core agentic tool loop | Working |
| 18 built-in tools | Working |
| Session persistence + resume | Working |
| CLAUDE.md discovery | Working |
| MCP stdio bootstrap | Working |
| Hierarchical config | Working |
| Permission modes | Working |
| Markdown terminal rendering | Working |
| Syntax highlighting | Working |

### 5.2 Major Gaps

| Feature | TS Has | Rust Status |
|---------|--------|-------------|
| Plugin system (load/install/lifecycle) | Full | Manifest-only, no runtime |
| Hook execution (PreToolUse/PostToolUse) | Full | Config parsed, not executed |
| CLI command breadth | 207 commands | ~15 commands |
| Tool breadth | 184 tools | 18 tools |
| Skills registry + bundled pipeline | Full | Local file only |
| LSP integration | Yes | Missing |
| MCP transports (SSE/WS/SDK) | Full | Stdio only |
| Remote/SSH/teleport modes | Yes | Missing |
| Task/Team management tools | Yes | Missing |
| Structured I/O transports | Yes | Missing |
| Agent orchestration | Full | Basic |

---

## 6. CLI Usage

### Rust CLI

```bash
# Interactive REPL
claw

# One-shot prompt
claw --model opus "explain this code"

# With specific permission mode
claw --permission-mode workspace-write

# Resume a session
claw --session=abc123

# JSON output mode
claw --output json "list files"
```

### Python CLI (introspection)

```bash
# Porting summary
python -m src.main summary

# Workspace manifest
python -m src.main manifest

# Subsystem listing
python -m src.main subsystems --limit 16

# Command/tool search
python -m src.main commands --limit 10 --query review
python -m src.main tools --limit 10 --query MCP

# Parity audit
python -m src.main parity-audit

# Route a prompt to matching commands/tools
python -m src.main route "review MCP tool" --limit 5

# Bootstrap a full session
python -m src.main bootstrap "review MCP tool" --limit 5

# Multi-turn loop
python -m src.main turn-loop "review MCP tool" --max-turns 2
```

---

## 7. Architectural Patterns

### 7.1 Rust Patterns

| Pattern | Where | Why |
|---------|-------|-----|
| **Generic trait boundaries** | `ConversationRuntime<C: ApiClient, T: ToolExecutor>` | Testability, swappable providers |
| **Factory from model alias** | `ProviderClient::from_model("opus")` | User-friendly model selection |
| **Hierarchical config merge** | `ConfigLoader` | Workspace → user → env → CLI precedence |
| **Permission-gated execution** | `PermissionPolicy::authorize()` | Safety sandbox for tools |
| **Hook pipeline** | `HookRunner` (PreToolUse → execute → PostToolUse) | Extensibility via plugins |
| **SSE stream parsing** | Custom `sse.rs` parser | Handles both `\n\n` and `\r\n\r\n` |
| **Session compaction** | `compact.rs` | Manages context window by summarizing old turns |
| **No unsafe code** | Workspace lint `forbid(unsafe_code)` | Memory safety guarantee |

### 7.2 Python Patterns

| Pattern | Where | Why |
|---------|-------|-----|
| **Registry-driven** | Commands/tools loaded from JSON snapshots | No hardcoded registry |
| **Frozen dataclasses** | All model types | Immutability |
| **Token-based routing** | `PortRuntime.route_prompt()` | Prompt → command/tool matching |
| **Trust-gated init** | `deferred_init.py` | Lazy plugin/skill/MCP loading |
| **Session compaction** | `transcript.py` | Budget-aware turn summarization |

---

## 8. Testing

### Rust

```bash
cd rust/
cargo fmt                                       # formatting
cargo clippy --workspace --all-targets -- -D warnings  # linting
cargo test --workspace                          # unit + integration tests
```

The `api` crate has dedicated tests in `rust/crates/api/tests/`.

### Python

```bash
python -m unittest discover -s tests -v
```

Test file `tests/test_porting_workspace.py` covers:
- Manifest generation
- Query engine summaries
- CLI subcommand execution (summary, parity-audit, commands, tools, route, bootstrap, etc.)
- Subsystem metadata loading
- Session persistence round-trip
- Tool permission filtering
- Multi-turn loop execution
- Remote/SSH/teleport modes

---

## 9. Project Provenance

The project originated from the Claude Code source exposure on March 31, 2026. The author (Sigrid Jin / @instructkr) performed a clean-room port:

1. **No direct code copy** — only architectural patterns and registries were captured
2. **JSON snapshots** preserve command/tool metadata without including proprietary logic
3. **Python scaffolding** serves as a reference implementation for the Rust port
4. **Parity audits** track coverage ratios without distributing original source

The porting workflow was orchestrated using **oh-my-codex (OmX)** — a multi-agent workflow layer built on OpenAI's Codex — using `$team` mode for parallel code review and `$ralph` mode for persistent execution loops.

---

## 10. Key Files Reference

| File | Purpose |
|------|---------|
| `CLAUDE.md` | Agent instructions for working with this repo |
| `PARITY.md` | Detailed gap analysis (TS vs Rust) |
| `README.md` | Project backstory, layout, quickstart |
| `rust/Cargo.toml` | Workspace manifest |
| `rust/TUI-ENHANCEMENT-PLAN.md` | Planned refactoring for the monolithic CLI |
| `rust/CONTRIBUTING.md` | Contribution guidelines |
| `src/main.py` | Python CLI entry point |
| `src/reference_data/` | JSON snapshots of original registries |
| `tests/test_porting_workspace.py` | Python test suite |

---

## 11. Summary

Claw Code is a **dual-language agent harness** project:

- The **Python layer** mirrors and documents 207 commands, 184 tools, and 30 subsystems from the original Claude Code, providing a comprehensive reference for what needs to be built.
- The **Rust layer** is a working, memory-safe implementation with multi-provider LLM support, streaming conversations, 18 built-in tools, session management, and a full terminal UI — but it covers roughly 10% of the original system's surface area.
- The project is **actively bridging the gap** — the compat-harness crate extracts TS manifests for automated parity checking, and the parity audit system tracks progress against the original.

The Rust implementation follows strong engineering principles: no unsafe code, generic trait boundaries for testability, hierarchical config, permission-sandboxed tool execution, and a plugin/hook architecture designed for extensibility.
