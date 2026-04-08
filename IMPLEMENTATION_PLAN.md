# Claw Code — Complete Implementation Plan

> Full-stack roadmap to bring the Rust port from MVP (~10% parity) to production-grade Claude Code alternative (~90%+ parity).

---

## Table of Contents

1. [Current State Summary](#1-current-state-summary)
2. [Phase 0: Structural Cleanup](#2-phase-0-structural-cleanup)
3. [Phase 1: Hook Execution & Plugin Runtime](#3-phase-1-hook-execution--plugin-runtime)
4. [Phase 2: Tool Expansion](#4-phase-2-tool-expansion)
5. [Phase 3: Command Expansion](#5-phase-3-command-expansion)
6. [Phase 4: MCP Transport Completion](#6-phase-4-mcp-transport-completion)
7. [Phase 5: Agent Orchestration & Tasks](#7-phase-5-agent-orchestration--tasks)
8. [Phase 6: TUI Polish](#8-phase-6-tui-polish)
9. [Phase 7: Remote, SSH & Structured I/O](#9-phase-7-remote-ssh--structured-io)
10. [Phase 8: Skills Registry & Bundled Pipeline](#10-phase-8-skills-registry--bundled-pipeline)
11. [Phase 9: Security Hardening](#11-phase-9-security-hardening)
12. [Phase 10: Distribution & Ecosystem](#12-phase-10-distribution--ecosystem)
13. [Complete Tool Inventory](#13-complete-tool-inventory)
14. [Complete Command Inventory](#14-complete-command-inventory)
15. [Dependency Graph](#15-dependency-graph)
16. [Milestone Targets](#16-milestone-targets)
17. [Risk Register](#17-risk-register)
18. [Verification Plan](#18-verification-plan)

---

## 1. Current State Summary

### Rust Workspace

| Crate | Lines | Status |
|---|---|---|
| `api` | ~1,500 | Multi-provider HTTP client + SSE streaming (Claude, Grok, OpenAI) |
| `runtime` | ~5,300 | `ConversationRuntime<C, T>` — sessions, config, permissions, MCP, compaction |
| `claw-cli` | ~3,600 | `LiveCli` REPL + one-shot + JSON mode. **Monolith: `main.rs` = 3,159 lines** |
| `commands` | ~470 | 21 slash commands with static metadata |
| `tools` | ~3,500 | 18 built-in tools (file, search, execution, web, task/agent, reference) |
| `plugins` | ~500 | Manifest parsing only — no runtime hook/plugin execution |
| `compat-harness` | ~200 | Compat layer for testing |

### Current Coverage

- **Tools**: 18/184 target tool modules (10%)
- **Commands**: 21/75 unique target commands (28%)
- **Hooks**: Config parsed, **not executed** at runtime
- **Plugins**: Manifests loaded, **no lifecycle management**
- **MCP**: stdio transport only; SSE/WebSocket/SDK/managed proxy missing
- **Skills**: Local `SKILL.md` file reading only; no registry, bundled pipeline, or MCP skill builders
- **TUI**: Inline scrolling output only; no status bar, no full-screen mode, no themes

### Known Bugs

| Bug | Severity | Location |
|---|---|---|
| JSON output leaks human-readable tool-result lines | Medium | `claw-cli/src/main.rs` |
| Hooks parsed but never fired | High | `runtime/src/conversation.rs` |
| Legacy `CliApp` in `app.rs` duplicates `LiveCli` | Low | `claw-cli/src/app.rs` |
| `main.rs` is a 3,159-line monolith | Medium | `claw-cli/src/main.rs` |
| Streaming `{}` prefix on tool input | Low | `claw-cli/src/main.rs` (partial fix) |

---

## 2. Phase 0: Structural Cleanup

**Goal**: Break the monolith, remove dead code, fix known bugs, establish module structure.
**Effort**: 2–3 weeks
**Prerequisite**: None

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 0.1 | Break `main.rs` monolith | Extract `LiveCli` struct/impl into `app.rs`, session management into `session_mgr.rs`, report formatting into `format.rs` | L | `main.rs` < 200 lines; each new module < 800 lines |
| 0.2 | Remove legacy `CliApp` | Delete or merge the unused `CliApp` in `app.rs` with `LiveCli` | S | No `CliApp` struct in codebase |
| 0.3 | Consolidate arg parsing | Unify the hand-rolled `parse_args()` and clap-based `args.rs` into one parser | S | Single arg parsing path |
| 0.4 | Create `tui/` module namespace | Add `crates/claw-cli/src/tui/mod.rs` for all enhanced TUI components | S | Module compiles |
| 0.5 | Fix JSON output cleanliness | Suppress human-readable tool-result lines when `--output-format json` | M | JSON mode outputs only valid JSON objects |
| 0.6 | Wire hook config into runtime | Plumb parsed hook config from `config.rs` into `ConversationRuntime` so Phase 1 can execute them | M | Hook config accessible in conversation loop |

**Target module structure after Phase 0**:
```
crates/claw-cli/src/
├── main.rs              # Entrypoint + arg dispatch (~100–200 lines)
├── args.rs              # Consolidated CLI argument parsing
├── app.rs               # LiveCli struct, REPL loop, turn execution
├── format.rs            # Report formatting (status, cost, model, etc.)
├── session_mgr.rs       # Session CRUD: create, resume, list, switch, persist
├── init.rs              # Repo initialization (unchanged)
├── input.rs             # Line editor (unchanged)
├── render.rs            # TerminalRenderer, Spinner (unchanged)
└── tui/
    └── mod.rs           # TUI module root (empty scaffold)
```

---

## 3. Phase 1: Hook Execution & Plugin Runtime

**Goal**: Make hooks actually fire; bootstrap plugin lifecycle.
**Effort**: 3–4 weeks
**Prerequisite**: Phase 0.6

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 1.1 | `PreToolUse` hook execution | Before each tool call, run matching hook commands. Support `deny`, `rewrite`, and `passthrough` outcomes | L | Hook can block a tool call; test with a deny-all hook |
| 1.2 | `PostToolUse` hook execution | After each tool call, run matching hook commands. Support result mutation and logging | M | Hook can modify tool output; test with an append hook |
| 1.3 | Hook timeout & error handling | Hooks that exceed timeout are killed; hook failures are logged but don't crash the runtime | M | Hanging hook doesn't block conversation |
| 1.4 | Plugin manifest validation | Validate plugin manifests against a schema on load; reject invalid manifests with clear errors | S | Invalid manifest produces actionable error |
| 1.5 | Plugin lifecycle commands | Implement `/plugin install`, `/plugin list`, `/plugin enable`, `/plugin disable`, `/plugin remove` | L | Can install a plugin from a local path and enable/disable it |
| 1.6 | Plugin-provided tools/hooks | Allow plugins to register additional tools and hooks that integrate with the runtime | L | Plugin tool appears in tool registry; plugin hook fires |
| 1.7 | `/hooks` command | Show all registered hooks, their source (user config, plugin, project), and their status | S | Output lists all hooks with source attribution |
| 1.8 | `/reload-plugins` command | Hot-reload plugin manifests without restarting the REPL | M | Plugin changes take effect without restart |

---

## 4. Phase 2: Tool Expansion

**Goal**: Expand from 18 to 50+ built-in tools, covering the most critical tool families.
**Effort**: 6–8 weeks
**Prerequisite**: Phase 0 (clean tool registration path)

### Currently Implemented (18 tools)

| # | Tool | Permission |
|---|---|---|
| 1 | `bash` | DangerFullAccess |
| 2 | `read_file` | ReadOnly |
| 3 | `write_file` | WorkspaceWrite |
| 4 | `edit_file` | WorkspaceWrite |
| 5 | `glob_search` | ReadOnly |
| 6 | `grep_search` | ReadOnly |
| 7 | `WebFetch` | ReadOnly |
| 8 | `WebSearch` | ReadOnly |
| 9 | `TodoWrite` | WorkspaceWrite |
| 10 | `Skill` | ReadOnly |
| 11 | `Agent` | DangerFullAccess |
| 12 | `ToolSearch` | ReadOnly |
| 13 | `NotebookEdit` | WorkspaceWrite |
| 14 | `Sleep` | ReadOnly |
| 15 | `SendUserMessage` | ReadOnly |
| 16 | `Config` | WorkspaceWrite |
| 17 | `StructuredOutput` | ReadOnly |
| 18 | `REPL` | DangerFullAccess |

### Tools to Add (Priority Order)

| ID | Tool | Family | Description | Effort | Priority |
|---|---|---|---|---|---|
| 2.1 | `AskUserQuestion` | Interactive | Prompt user with a question and collect response | S | P0 |
| 2.2 | `ListMcpResources` | MCP | List available resources from connected MCP servers | M | P0 |
| 2.3 | `MCPTool` | MCP | Execute a tool provided by an MCP server | L | P0 |
| 2.4 | `ReadMcpResource` | MCP | Read a resource from an MCP server | M | P0 |
| 2.5 | `McpAuth` | MCP | Manage MCP server authentication and credentials | M | P1 |
| 2.6 | `LSPTool` | IDE | Language Server Protocol integration for diagnostics, completions, symbols | XL | P1 |
| 2.7 | `TaskCreate` | Tasks | Create a background task/agent with isolated context | L | P1 |
| 2.8 | `TaskGet` | Tasks | Retrieve status and output of a running task | S | P1 |
| 2.9 | `TaskList` | Tasks | List all active and completed tasks | S | P1 |
| 2.10 | `TaskOutput` | Tasks | Stream output from a running task | M | P1 |
| 2.11 | `TaskStop` | Tasks | Cancel a running task | S | P1 |
| 2.12 | `TaskUpdate` | Tasks | Update a running task's parameters | S | P1 |
| 2.13 | `TeamCreate` | Teams | Create a team of agents with shared context | L | P2 |
| 2.14 | `TeamDelete` | Teams | Delete a team and clean up resources | S | P2 |
| 2.15 | `RemoteTrigger` | Remote | Trigger a remote action or webhook | M | P2 |
| 2.16 | `ScheduleCronCreate` | Scheduling | Create a scheduled cron job | M | P2 |
| 2.17 | `ScheduleCronList` | Scheduling | List active cron schedules | S | P2 |
| 2.18 | `ScheduleCronDelete` | Scheduling | Remove a scheduled cron job | S | P2 |
| 2.19 | `SyntheticOutput` | Internal | Generate synthetic tool output for testing/replay | S | P2 |

### Tool Implementation Pattern

Each new tool should follow this pattern in `crates/tools/src/lib.rs`:

1. Add `ToolSpec` entry to `mvp_tool_specs()` with name, description, JSON schema, permission
2. Add execution branch in `execute_tool()` match
3. Add unit tests for schema validation, happy path, error path
4. Wire permission into `GlobalToolRegistry`

---

## 5. Phase 3: Command Expansion

**Goal**: Expand from 21 to 50+ slash commands, covering the most-used TS command families.
**Effort**: 4–6 weeks
**Prerequisite**: Phase 0 (clean command dispatch)

### Currently Implemented (21 commands)

`/help`, `/status`, `/compact`, `/model`, `/permissions`, `/clear`, `/cost`, `/resume`, `/config`, `/memory`, `/init`, `/diff`, `/version`, `/bughunter`, `/commit`, `/pr`, `/issue`, `/ultraplan`, `/teleport`, `/debug-tool-call`, `/export`, `/session`

### Commands to Add (Priority Order)

| ID | Command | Description | Effort | Priority | Depends On |
|---|---|---|---|---|---|
| 3.1 | `/agents` | List, configure, and manage agent definitions | M | P0 | Phase 5 |
| 3.2 | `/hooks` | List all hooks and their status | S | P0 | Phase 1.7 |
| 3.3 | `/mcp` | Manage MCP server connections (list, connect, disconnect) | M | P0 | Phase 4 |
| 3.4 | `/plugin` | Plugin lifecycle management (install, enable, disable, remove) | M | P0 | Phase 1.5 |
| 3.5 | `/skills` | List and manage skill definitions | M | P0 | Phase 8 |
| 3.6 | `/plan` | Start a structured planning session | M | P1 | — |
| 3.7 | `/review` | Code review the current workspace changes | M | P1 | — |
| 3.8 | `/tasks` | List and manage background tasks | M | P1 | Phase 5 |
| 3.9 | `/add-dir` | Add a directory to the workspace scope | S | P1 | — |
| 3.10 | `/login` | Authenticate with Anthropic or other providers | M | P1 | — |
| 3.11 | `/logout` | Clear stored authentication credentials | S | P1 | — |
| 3.12 | `/doctor` | Run diagnostics on the environment (API keys, tools, MCP servers) | M | P1 | — |
| 3.13 | `/context` | Show or manage conversation context window usage | M | P1 | — |
| 3.14 | `/summary` | Generate a summary of the conversation so far | M | P1 | — |
| 3.15 | `/theme` | Switch color theme | S | P2 | Phase 6 |
| 3.16 | `/stats` | Show detailed session statistics (turns, tool calls, time) | S | P2 | — |
| 3.17 | `/rewind` | Revert to a previous point in the conversation | M | P2 | — |
| 3.18 | `/share` | Share a conversation or export for collaboration | M | P2 | — |
| 3.19 | `/sandbox-toggle` | Toggle sandboxed execution mode | S | P2 | — |
| 3.20 | `/security-review` | Run a security scan on workspace files | M | P2 | — |
| 3.21 | `/usage` | Show API usage and billing information | S | P2 | — |
| 3.22 | `/tag` | Tag a conversation for later retrieval | S | P2 | — |
| 3.23 | `/vim` | Toggle vim keybinding mode for input | M | P3 | — |
| 3.24 | `/voice` | Toggle voice input/output mode | XL | P3 | — |
| 3.25 | `/stickers` | Decorative sticker reactions | S | P3 | — |

---

## 6. Phase 4: MCP Transport Completion

**Goal**: Full MCP transport parity — stdio + SSE + WebSocket + SDK + managed proxy.
**Effort**: 4–6 weeks
**Prerequisite**: Phase 0

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 4.1 | SSE transport | Implement MCP server-sent events transport for HTTP-based MCP servers | L | Can connect to an SSE MCP server and execute tools |
| 4.2 | WebSocket transport | Implement MCP WebSocket transport for real-time bidirectional communication | L | Can connect to a WS MCP server |
| 4.3 | MCP SDK client | Implement the MCP SDK client protocol for server discovery and capability negotiation | M | Auto-discovers server capabilities |
| 4.4 | Managed proxy transport | Support connecting through Anthropic's managed MCP proxy | L | Can use managed proxy for server connections |
| 4.5 | MCP connection manager | Centralized manager for multiple concurrent MCP connections with health checks | M | Can connect to multiple MCP servers simultaneously |
| 4.6 | MCP auth layer | OAuth and token-based authentication for MCP servers | M | Authenticated connections persist across sessions |
| 4.7 | `/mcp` command implementation | Full MCP management: list servers, connect, disconnect, show tools, show resources | M | Complete MCP server lifecycle management |
| 4.8 | MCP tool integration | Auto-register tools from connected MCP servers into the `GlobalToolRegistry` | M | MCP tools appear in `/help` and are callable |

---

## 7. Phase 5: Agent Orchestration & Tasks

**Goal**: Sub-agent spawning, background tasks, team coordination.
**Effort**: 5–7 weeks
**Prerequisite**: Phase 2 (agent tool), Phase 1 (hooks)

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 5.1 | Task spawning runtime | Background task execution with isolated `ConversationRuntime` instances | XL | Can spawn a task that runs in background |
| 5.2 | Task lifecycle management | Start, monitor, pause, resume, stop, and clean up tasks | L | All task state transitions work |
| 5.3 | Task output streaming | Real-time output from background tasks visible in main REPL | L | Can tail a running task |
| 5.4 | Agent type registry | Named agent types (planner, reviewer, etc.) with preset system prompts | M | `/agents` lists available agent types |
| 5.5 | Team coordination | Multiple agents sharing context with a coordinator agent | XL | Can create a team, send work, get results |
| 5.6 | Handoff protocol | Structured metadata exchange between parent and child agents | M | Agent results include structured handoff data |
| 5.7 | Task persistence | Tasks survive session restarts; can resume from last checkpoint | L | Resume a task after restart |
| 5.8 | Resource limits | CPU/memory/time limits per task to prevent runaway agents | M | Task killed after exceeding limits |

---

## 8. Phase 6: TUI Polish

**Goal**: Modern terminal experience — status bar, live markdown, themes, collapsible output.
**Effort**: 4–6 weeks
**Prerequisite**: Phase 0 (clean module structure)

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 6.1 | Status bar with live tokens | Bottom-pinned status bar: model, permission mode, session ID, token count, cost | M | Updates in real-time during streaming |
| 6.2 | Live markdown rendering | Incremental markdown rendering during streaming (headings, bold, code) | L | Assistant response renders rich markdown as it streams |
| 6.3 | Remove artificial stream delay | Eliminate the 8ms-per-chunk sleep in `stream_markdown` | S | Immediate streaming output |
| 6.4 | Collapsible tool output | Long tool results (>15 lines) truncated with expand option | M | Large bash output doesn't flood screen |
| 6.5 | Syntax-highlighted tool results | Apply syntect highlighting to bash output, file contents, REPL results | M | Code in tool results is colored |
| 6.6 | Tool call timeline | Summary line after multi-tool turns: `🔧 bash → ✓ │ read_file → ✓ (3 tools, 1.2s)` | S | Compact tool summary visible |
| 6.7 | Diff-aware edit display | Show colored unified diff when `edit_file` succeeds | M | Edit changes visible at a glance |
| 6.8 | Colored `/diff` output | Parse git diff and render with red/green coloring | M | Diff is readable without external tools |
| 6.9 | Named color themes | `dark`, `light`, `solarized`, `catppuccin` themes via `/theme` command | M | Can switch themes |
| 6.10 | Thinking indicator | Distinct visual for extended thinking mode (pulsing dots, different icon) | S | Thinking mode is visually distinct |
| 6.11 | Terminal resize handling | Detect terminal size changes and adjust layout | M | Output adapts to window resize |
| 6.12 | Internal pager | Scroll through long `/status`, `/config`, `/memory` output with j/k/q | M | Long output is navigable |
| 6.13 | Full-screen TUI mode (stretch) | Optional `ratatui`-based alternate-screen with split panes, scrollback | XL | `--tui` flag launches full-screen mode |

---

## 9. Phase 7: Remote, SSH & Structured I/O

**Goal**: Non-interactive and remote usage patterns.
**Effort**: 4–5 weeks
**Prerequisite**: Phase 0 (JSON output fix), Phase 4 (transports)

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 7.1 | Structured JSON I/O protocol | Clean JSON protocol for machine consumption (events, tool calls, results) | L | External tool can drive Claw via JSON |
| 7.2 | Remote transport layer | Accept connections from remote clients (IDE extensions, web UIs) | L | VS Code extension can connect to Claw |
| 7.3 | SSH session support | Tunnel conversations over SSH for headless server usage | M | Can run Claw on a remote server via SSH |
| 7.4 | Pipe mode | Accept input from stdin, output to stdout for shell scripting | M | `echo "explain this" │ claw --pipe` works |
| 7.5 | HTTP server mode | Optional HTTP server for REST API access to Claw | L | Can POST a message and get a response |
| 7.6 | Event bus | Internal event bus for loose coupling between components | L | Components communicate via events, not direct calls |

---

## 10. Phase 8: Skills Registry & Bundled Pipeline

**Goal**: Full skill discovery, bundled skills, MCP-based skill loading.
**Effort**: 3–4 weeks
**Prerequisite**: Phase 4 (MCP), Phase 2 (Skill tool)

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 8.1 | Skill discovery & registry | Scan configured skill directories, index skills by name/tag/description | M | `/skills` lists all discoverable skills |
| 8.2 | Bundled skills | Ship a set of built-in skills (code review, planning, TDD, security) | M | Bundled skills available out of the box |
| 8.3 | MCP skill builders | Load skills from MCP servers with tool/resource bindings | L | Skill from MCP server is usable |
| 8.4 | Skill hot-reload | Detect skill file changes and reload without restart | M | Edit SKILL.md → immediately reflected |
| 8.5 | Session/team memory for skills | Persist skill-related context across sessions and team members | M | Skill state survives session restart |
| 8.6 | `/skills` command | Browse, search, inspect, and invoke skills interactively | M | Interactive skill management |

---

## 11. Phase 9: Security Hardening

**Goal**: Production-grade security for tool execution and credential management.
**Effort**: 3–4 weeks
**Prerequisite**: Phase 1 (hooks for security policies)

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 9.1 | Sandboxed tool execution | Optional sandboxing for `bash` and file tools (seccomp, namespace, or WASM) | XL | Sandboxed bash can't access `/etc/passwd` |
| 9.2 | Credential vault | Secure storage for API keys and MCP tokens (OS keychain integration) | L | Credentials encrypted at rest |
| 9.3 | Policy engine | Declarative security policies: deny tool patterns, restrict paths, limit network | L | Policy blocks tool call matching deny rule |
| 9.4 | Audit logging | Append-only audit log of all tool executions with timestamps | M | Every tool call logged with context |
| 9.5 | Rate limiting | Configurable rate limits per tool and per session | M | Rapid repeated tool calls throttled |
| 9.6 | Secret scanning | Detect and warn about secrets in tool inputs/outputs | M | API key in bash output triggers warning |
| 9.7 | Permission escalation audit | Log when permissions are elevated; require explicit confirmation | S | Elevation from ReadOnly to DangerFullAccess logged |

---

## 12. Phase 10: Distribution & Ecosystem

**Goal**: Make Claw installable, maintainable, and ready for community contribution.
**Effort**: 3–4 weeks
**Prerequisite**: Phases 0–3 minimum

| ID | Task | Description | Effort | Acceptance Criteria |
|---|---|---|---|---|
| 10.1 | Cross-platform binaries | CI pipeline producing binaries for Linux x64/arm64, macOS x64/arm64, Windows x64 | L | Release artifacts for all platforms |
| 10.2 | `cargo install` support | Publish to crates.io or provide `cargo install --git` instructions | M | `cargo install claw-code` works |
| 10.3 | Homebrew formula | Tap + formula for macOS/Linux installation | M | `brew install claw-code` works |
| 10.4 | Shell completions | Generate completions for bash, zsh, fish, PowerShell | M | Tab completion for CLI flags |
| 10.5 | Man pages | Generate man pages from CLI metadata | S | `man claw` works on Unix |
| 10.6 | Changelog automation | Auto-generate changelog from conventional commits | S | CHANGELOG.md stays current |
| 10.7 | Plugin marketplace scaffold | Basic plugin registry (GitHub-based initially) with search and install | L | `claw plugin install author/plugin` works |
| 10.8 | Contributor documentation | Architecture guide, tool authoring guide, command authoring guide | M | New contributor can add a tool in < 1 hour |
| 10.9 | Integration test suite | End-to-end tests with mocked API for the full conversation loop | L | CI runs E2E tests on every PR |

---

## 13. Complete Tool Inventory

### Target Tool Families (from TS reference: 30 families, 184 module entries)

| Family | Tools | Rust Status |
|---|---|---|
| **AgentTool** | Agent | ✅ Implemented |
| **AskUserQuestionTool** | AskUserQuestion | ❌ Missing |
| **BashTool** | bash | ✅ Implemented |
| **ConfigTool** | Config | ✅ Implemented |
| **FileReadTool** | read_file | ✅ Implemented |
| **FileWriteTool** | write_file | ✅ Implemented |
| **FileEditTool** | edit_file | ✅ Implemented |
| **GlobTool** | glob_search | ✅ Implemented |
| **GrepTool** | grep_search | ✅ Implemented |
| **LSPTool** | LSP diagnostics, symbols, completions | ❌ Missing |
| **ListMcpResourcesTool** | ListMcpResources | ❌ Missing |
| **MCPTool** | MCPTool (proxy to MCP server tools) | ❌ Missing |
| **McpAuthTool** | McpAuth | ❌ Missing |
| **NotebookEditTool** | NotebookEdit | ✅ Implemented |
| **ReadMcpResourceTool** | ReadMcpResource | ❌ Missing |
| **RemoteTriggerTool** | RemoteTrigger | ❌ Missing |
| **ScheduleCronTool** | CronCreate, CronList, CronDelete | ❌ Missing |
| **SendMessageTool** | SendUserMessage | ✅ Implemented |
| **SkillTool** | Skill | ✅ Implemented |
| **SleepTool** | Sleep | ✅ Implemented |
| **StructuredOutputTool** | StructuredOutput | ✅ Implemented |
| **SyntheticOutputTool** | SyntheticOutput | ❌ Missing |
| **TaskTool** | TaskCreate, TaskGet, TaskList, TaskOutput, TaskStop, TaskUpdate | ❌ Missing |
| **TeamTool** | TeamCreate, TeamDelete | ❌ Missing |
| **TodoWriteTool** | TodoWrite | ✅ Implemented |
| **ToolSearchTool** | ToolSearch | ✅ Implemented |
| **WebFetchTool** | WebFetch | ✅ Implemented |
| **WebSearchTool** | WebSearch | ✅ Implemented |
| **REPLTool** | REPL | ✅ Implemented |
| **PowerShellTool** | (subsumed under bash) | ✅ Implemented |

**Summary**: 18/30 families have at least one tool implemented. 12 families are entirely missing.

---

## 14. Complete Command Inventory

### Target Commands (from TS reference: 75 unique commands)

| Command | Rust Status | Phase |
|---|---|---|
| `/add-dir` | ❌ | 3 |
| `/agents` | ❌ | 5 |
| `/bug` | ❌ | 3 |
| `/bughunter` | ✅ | — |
| `/clear` | ✅ | — |
| `/commit` | ✅ | — |
| `/compact` | ✅ | — |
| `/config` | ✅ | — |
| `/context` | ❌ | 3 |
| `/cost` | ✅ | — |
| `/debug-tool-call` | ✅ | — |
| `/diff` | ✅ | — |
| `/doctor` | ❌ | 3 |
| `/export` | ✅ | — |
| `/help` | ✅ | — |
| `/hooks` | ❌ | 1 |
| `/init` | ✅ | — |
| `/install` | ❌ | 10 |
| `/issue` | ✅ | — |
| `/listen` | ❌ | 3 |
| `/login` | ❌ | 3 |
| `/logout` | ❌ | 3 |
| `/mcp` | ❌ | 4 |
| `/memory` | ✅ | — |
| `/model` | ✅ | — |
| `/permissions` | ✅ | — |
| `/plan` | ❌ | 3 |
| `/plugin` | ❌ | 1 |
| `/pr` | ✅ | — |
| `/reload-plugins` | ❌ | 1 |
| `/resume` | ✅ | — |
| `/review` | ❌ | 3 |
| `/rewind` | ❌ | 3 |
| `/sandbox-toggle` | ❌ | 9 |
| `/security-review` | ❌ | 9 |
| `/session` | ✅ | — |
| `/share` | ❌ | 3 |
| `/skills` | ❌ | 8 |
| `/stats` | ❌ | 3 |
| `/status` | ✅ | — |
| `/stickers` | ❌ | 3 |
| `/summary` | ❌ | 3 |
| `/tag` | ❌ | 3 |
| `/tasks` | ❌ | 5 |
| `/teleport` | ✅ | — |
| `/theme` | ❌ | 6 |
| `/ultraplan` | ✅ | — |
| `/upgrade` | ❌ | 10 |
| `/usage` | ❌ | 3 |
| `/version` | ✅ | — |
| `/vim` | ❌ | 3 |
| `/voice` | ❌ | 3 |

**Summary**: 21/52 listed commands implemented (40%). Many TS commands are aliases or variants (e.g., `review`/`ultrareview`) — unique functional commands tracked above.

---

## 15. Dependency Graph

```
Phase 0 (Structural Cleanup)
    │
    ├──→ Phase 1 (Hooks & Plugins)
    │        │
    │        ├──→ Phase 5 (Agent Orchestration) ──→ Phase 5 Tools/Commands
    │        └──→ Phase 9 (Security Hardening)
    │
    ├──→ Phase 2 (Tool Expansion) ──→ Phase 8 (Skills Registry)
    │
    ├──→ Phase 3 (Command Expansion)
    │
    ├──→ Phase 4 (MCP Transports) ──→ Phase 8 (Skills Registry)
    │        │
    │        └──→ Phase 7 (Remote/SSH/Structured I/O)
    │
    ├──→ Phase 6 (TUI Polish)
    │
    └──→ Phase 10 (Distribution) ← depends on Phases 0–3 minimum
```

**Critical path**: Phase 0 → Phase 1 → Phase 5 (enables agent tools, task tools, team tools)
**Parallel paths**: Phases 2, 3, 4, 6 can run concurrently after Phase 0.

---

## 16. Milestone Targets

| Milestone | Phases | Parity Target | Key Deliverables |
|---|---|---|---|
| **M1 — Foundation** | 0 | ~15% | Clean module structure, no monolith, bugs fixed |
| **M2 — Hooks Live** | 0 + 1 | ~25% | Hooks execute, basic plugin lifecycle |
| **M3 — Tool Coverage** | 0 + 2 | ~35% | 50+ tools (MCP tools, task tools, ask-user) |
| **M4 — Command Coverage** | 0 + 3 | ~45% | 40+ commands (/plan, /review, /doctor, etc.) |
| **M5 — MCP Parity** | 0 + 4 | ~55% | All MCP transports, connection manager |
| **M6 — Agent Power** | 0–5 | ~65% | Background tasks, teams, agent types |
| **M7 — TUI + Skills** | 0–6 + 8 | ~75% | Status bar, themes, skill registry |
| **M8 — Remote + Security** | 0–9 | ~85% | Structured I/O, sandboxing, audit log |
| **M9 — Ship It** | 0–10 | ~90%+ | Cross-platform binaries, docs, plugin marketplace |

---

## 17. Risk Register

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| `main.rs` refactor introduces regressions | High | Medium | Existing test coverage is strong; run full suite after each extraction |
| MCP protocol changes upstream | Medium | Medium | Pin MCP protocol version; abstract transport layer |
| Hook execution introduces security holes | High | Low | Hooks run in subprocess with timeout; no access to runtime internals |
| Plugin system becomes attack surface | High | Medium | Validate manifests strictly; sandbox plugin tool execution |
| Full-screen TUI scope creep | Medium | High | Ship inline TUI (Phases 0–5) before starting full-screen mode |
| Agent orchestration complexity | High | Medium | Start with simple fork-join; add coordination incrementally |
| Multi-provider API drift | Medium | Medium | Provider-specific adapters behind `ProviderClient` trait |
| Community contribution friction | Medium | Low | Investment in Phase 10.8 (contributor docs) reduces barrier |
| Licensing/legal risk from TS source exposure origin | High | Low | Clean-room implementation only; no TS code copied |

---

## 18. Verification Plan

### Per-Phase Verification

Each phase must pass before the next begins:

```bash
# From rust/ directory
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Coverage Targets

| Milestone | Coverage Target |
|---|---|
| M1 | 70% (baseline) |
| M3 | 75% |
| M5 | 80% |
| M9 | 85% |

### Integration Testing

- **Tool tests**: Each new tool gets happy-path + error-path + permission-denied tests
- **Hook tests**: PreToolUse deny, rewrite, passthrough; PostToolUse mutation; timeout; cascade
- **MCP tests**: Mock MCP server for each transport (stdio, SSE, WS)
- **E2E tests**: Full conversation loop with mocked API for critical flows

### Manual Testing Checkpoints

| Checkpoint | Scope |
|---|---|
| After Phase 0 | Full REPL regression: all 21 existing commands, tool calls, sessions |
| After Phase 1 | Hook deny/rewrite flow; plugin install/enable/disable |
| After Phase 4 | Connect to a real MCP server via SSE; execute MCP tool |
| After Phase 6 | Visual review in: Linux (xterm), macOS (Terminal.app, iTerm2), Windows (Windows Terminal), tmux, SSH |
| After Phase 9 | Security audit: escape sandbox, credential leak, path traversal |

---

*Generated: 2026-04-02 | Repository: claw-code | Branch: dev/rust*
