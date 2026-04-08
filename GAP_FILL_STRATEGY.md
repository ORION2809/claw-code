# Claw Code — Gap Fill Strategy
## Optimal Approaches to Close Residual Gaps Post-Implementation-Plan

> This document analyses each structural gap identified in the residual gap analysis, cross-references it against the implementation plan's current phases, and proposes the most optimal — and in many cases non-obvious — engineering or strategic approaches to narrow or close each gap.

---

## Framing: Gap Categories

Before diving in, it's critical to separate the gaps by their *nature*, because the fill strategy differs fundamentally per type:

| Gap Type | Fillable? | Best Lever |
|---|---|---|
| Missing infrastructure | ✅ Fully | Engineering |
| Missing prompt intelligence | ⚠️ Partially | Prompt engineering + heuristics |
| Missing ecosystem | ⚠️ Partially | Community + integrations |
| Intrinsic model tuning | ❌ Not directly | Workarounds only |
| Network effects | ❌ Not directly | Growth strategy |

---

## Gap 1 — Model Orchestration (Hidden Prompt Engineering)

### Root Cause

The IMPL plan builds *infrastructure* around the model (tools, transports, hooks). It does not address *how the model is guided*. Official Claude Code has deeply tuned system prompts, dynamic context injection, and model-specific heuristics baked in.

### Optimal Fill Strategy

**1.1 — Build a Prompt Engineering Layer (Phase 0.5 addition)**

Create a `prompt_engine` crate that owns all system prompt construction. This should not be a static string — it must be a *composable pipeline*:

```
PromptEngine
├── BaseSystemPrompt         // core identity and behavior rules
├── TaskContextInjector      // injects current task type (code review, refactor, explain)
├── ToolAvailabilityPrimer   // summarises which tools are active and when to use them
├── WorkspaceContextPrimer   // injects repo language, file tree summary, git state
├── ModelAdapter             // adapts prompts per provider (claude vs openai vs grok)
└── GuardrailLayer           // appends safety/scope constraints
```

Each component is independently testable and replaceable. The `ModelAdapter` module is the highest-leverage piece: Claude, GPT-4o, and Grok respond to different prompt patterns for tool use. Detecting provider and adapting the prompt framing recovers a significant portion of the quality gap.

**1.2 — Tool Selection Priming**

Before every conversation turn, inject a compact *tool hint block* into the system prompt dynamically:

```
Active tools: [read_file, grep_search, bash, LSP]
Current task context: code debugging
Preferred tool sequence: LSP diagnostic → grep_search → read_file → bash
```

This is cheap to implement and measurably reduces incorrect tool selection. It does not replicate learned routing but approximates it through explicit instruction.

**1.3 — Failure Recovery via Re-framing Hooks**

In the IMPL plan, hooks handle `PreToolUse` and `PostToolUse`. Extend this with a `OnToolFailure` hook type that:

1. Detects tool call failures (wrong args, permission denied, bad output)
2. Re-injects context into the next turn: `"Previous tool call failed with: [reason]. Consider using [alternative_tool] instead."`
3. Limits retry depth to 3 with exponential backoff in prompt verbosity

This covers ~60% of the "recovery intelligence" gap without requiring model-level changes.

**1.4 — Per-Provider Heuristic Config Files**

Ship provider-specific TOML configs (`claude.toml`, `openai.toml`, `grok.toml`) that define:
- Preferred tool calling format
- Max parallel tool calls
- Prompt length thresholds
- Known failure patterns and their re-frame templates

These become community-improvable over time, creating a feedback loop that partially compensates for the official system's internally tuned heuristics.

---

## Gap 2 — Agent Intelligence vs Agent Infrastructure

### Root Cause

Phase 5 builds the scaffolding (task spawning, registries, team coordination). But agents are still just parallel instances of the same model loop — they don't have specialised roles or adaptive delegation.

### Optimal Fill Strategy

**2.1 — Role-Specialised Agent Personas**

Each agent type should carry a *role prompt overlay* that scopes its behaviour:

| Agent Role | Prompt Overlay Focus |
|---|---|
| `Planner` | Decompose goals, output structured task lists |
| `Executor` | Execute one task at a time, emit structured output |
| `Reviewer` | Critique output against acceptance criteria |
| `Debugger` | Focused on error tracing, log reading, diagnostic tools |
| `Synthesiser` | Merge outputs, resolve conflicts, produce final deliverable |

This does NOT require model fine-tuning. A well-constructed role prompt overlay, injected via the `PromptEngine` from Gap 1, moves agent behaviour meaningfully toward specialised collaboration.

**2.2 — Structured Agent Communication Protocol**

Agents in Phase 5 communicate via the task system. Introduce a structured message envelope:

```json
{
  "from": "planner-agent-01",
  "to": "executor-agent-02",
  "message_type": "task_assignment",
  "payload": {
    "task": "Refactor auth module",
    "acceptance_criteria": ["passes existing tests", "no new TODOs"],
    "context_refs": ["src/auth/mod.rs", "ARCHITECTURE.md"]
  }
}
```

This transforms agents from "parallel scripts" to *protocol-aware collaborators*. The structure also makes agent communication observable and debuggable — a major operational advantage over opaque unofficial parity.

**2.3 — Planning Depth via Hierarchical Plan Trees**

The `/plan` and `/ultraplan` commands exist in the IMPL plan but are treated as linear outputs. Extend them to produce *tree-structured plans* stored as typed data structures:

```
PlanNode {
  id, title, description,
  subtasks: Vec<PlanNode>,
  dependencies: Vec<PlanNodeId>,
  assigned_agent: Option<AgentRole>,
  status: Pending | InProgress | Complete | Failed
}
```

During execution, the runtime walks this tree, assigns agents to leaf nodes, and propagates status upward. This delivers hierarchical planning without any model-level changes — only structured output parsing and a tree executor.

---

## Gap 3 — MCP Ecosystem Density

### Root Cause

The IMPL plan achieves protocol compatibility (all transports). But compatibility without servers is a socket with nothing plugged in. The official ecosystem has dozens of pre-built integrations.

### Optimal Fill Strategy

**3.1 — Bundle a Core MCP Server Set**

Ship Claw Code with a bundled `mcp-servers/` directory containing first-party lightweight MCP servers for the 10 highest-value integrations:

| Server | Value |
|---|---|
| `filesystem-mcp` | Enhanced file ops beyond built-in tools |
| `git-mcp` | Deep git history, blame, branch ops |
| `github-mcp` | Issues, PRs, Actions via GitHub API |
| `postgres-mcp` | Query, introspect, migrate databases |
| `docker-mcp` | Container management |
| `http-mcp` | Generic HTTP client with auth |
| `sqlite-mcp` | Local database ops |
| `browser-mcp` | Headless browser for scraping/testing |
| `k8s-mcp` | Kubernetes cluster interaction |
| `slack-mcp` | Notifications and channel reads |

These don't need to be feature-complete on day one. A minimal but working version of each is dramatically more valuable than a complete spec with no implementations.

**3.2 — MCP Server Discovery Registry**

Build a lightweight `registry.json` (community-hosted) that catalogs known MCP servers with:
- name, description, transport type
- GitHub URL
- verified/community tag
- install command

In Claw Code, implement `/mcp search <keyword>` and `/mcp install <name>` that pull from this registry. This becomes a flywheel: as Claw Code grows, the registry grows, making the ecosystem gap self-closing.

**3.3 — Capability Negotiation Improvement**

The gap doc identifies static capability handling as a weakness. Implement a `CapabilityNegotiator` that:
1. On MCP connection, queries the server's capability manifest
2. Builds a *runtime capability map* for that session
3. Injects the capability map into the `ToolAvailabilityPrimer` (from Gap 1.2) so the model knows what's available and what isn't

This closes the "suboptimal use of MCP tools" gap without requiring changes to the MCP protocol itself.

---

## Gap 4 — Plugin System Quality and Safety

### Root Cause

Phase 1 builds the plugin lifecycle. Phase 10 adds a marketplace. But plugin quality, trust, and isolation remain weak compared to a curated system.

### Optimal Fill Strategy

**4.1 — Plugin Trust Tiers**

Introduce a three-tier plugin trust model directly in the plugin manifest:

| Tier | Criteria | Permissions Granted |
|---|---|---|
| `core` | Bundled with Claw Code | Full tool + hook access |
| `verified` | Community reviewed, signed | Full tool access, limited hook access |
| `community` | Unsigned, user-installed | Restricted tools only, no PreToolUse hooks |

The runtime enforces permission boundaries per tier. This gives users a clear mental model and reduces the security surface without requiring a formal audit process for every plugin.

**4.2 — WASM Sandboxing for Plugin Isolation**

The IMPL plan mentions "best-effort sandboxing." The optimal fill here is WASM-based plugin execution. Compile plugins to WASM and run them in a `wasmtime` or `wasmer` runtime. This gives:
- Memory isolation by default
- CPU time limits
- No filesystem/network access unless explicitly granted via WASM interface types
- Rust-native (both `wasmtime` and `wasmer` have first-class Rust APIs)

This is a non-trivial engineering investment (~3–4 weeks) but it closes the plugin isolation gap almost entirely and makes the security posture defensible for enterprise use.

**4.3 — Plugin Manifest Schema Versioning**

Version the plugin manifest schema from day one (`schema_version: "1.0"`). This enables backwards-compatible evolution of the plugin API without breaking existing plugins — a problem that unofficial ecosystems historically encounter badly when they try to add security features retroactively.

---

## Gap 5 — Security and Governance (Enterprise Critical)

### Root Cause

Phase 9 adds sandboxing, audit logs, and a policy engine. But the gap doc correctly identifies that these are *configurable* rather than *inherently enforced* — a critical distinction for regulated industries.

### Optimal Fill Strategy

**5.1 — Policy-as-Code with a Rule Engine**

Instead of a configurable policy engine, implement a *declarative policy language* (a small DSL or YAML-based rule format) that is enforced at the runtime level:

```yaml
# .clawcode/policy.yaml
rules:
  - id: no-prod-writes
    action: deny
    condition:
      tool: write_file
      path_matches: "/prod/*"

  - id: require-review-on-deploy
    action: require_approval
    condition:
      tool: bash
      command_matches: "kubectl apply*"
```

The runtime parses this at startup and enforces rules inside the `PreToolUse` hook path — which already exists after Phase 1. This is low additional implementation cost but high governance value.

**5.2 — Structured Audit Log with Integrity Guarantees**

The IMPL plan includes audit logs. Elevate these to *append-only structured logs* with optional HMAC signing per entry:

```json
{
  "ts": "2026-04-02T14:22:01Z",
  "session_id": "sess_abc123",
  "event": "tool_call",
  "tool": "bash",
  "args": {"command": "rm -rf /tmp/build"},
  "outcome": "allowed",
  "policy_rule_applied": null,
  "hmac": "sha256:..."
}
```

HMAC signing means audit logs can be verified as untampered — a requirement for SOC2 and similar certifications. The signing key can be user-managed or enterprise-managed.

**5.3 — Data Classification Tags**

Add a lightweight data classification layer to the tool output pipeline:

```
ToolOutput
├── content: String
├── classification: Public | Internal | Confidential | Secret
└── source_path: Option<String>
```

When a tool reads a file that matches a `confidential_paths` pattern in the policy config, its output is tagged as `Confidential`. The model still sees the content, but the audit log records the classification, and classified content is never sent to telemetry or shared sessions.

---

## Gap 6 — UX and Interaction Intelligence

### Root Cause

Phase 6 delivers a functional TUI. But the gap remains in *cognitive load reduction* — the official system does smart summarisation, implicit context pruning, and optimised latency perception.

### Optimal Fill Strategy

**6.1 — Implicit Context Summarisation**

Implement an automatic context compaction trigger that fires not just when the context window is near-full (as in the IMPL plan's existing `/compact` command) but *proactively* at a configurable threshold (e.g., 60% of context window used):

1. Summarise completed tool sequences into a single "task completed" block
2. Retain only the final output and error state, not intermediate steps
3. Append the summary as an assistant message and truncate the history

This mimics the official system's implicit context pruning without any model-level access. The user never has to invoke `/compact` manually.

**6.2 — Progress Perception Layer**

Latency *perception* is as important as actual latency. Add a `ProgressPerception` module to the TUI that:
- Shows incremental thinking steps ("Searching files...", "Reading auth module...", "Writing patch...")
- Distinguishes between model thinking time and tool execution time visually
- Shows token consumption in real-time so users understand *why* a long response is long

This is a pure TUI addition with no API changes required but dramatically improves the subjective experience.

**6.3 — Smart Input Completion**

Add context-aware slash-command and file-path completion to the input layer. When the user types `/`, show available commands filtered by current context (e.g., show `/tasks` only if active tasks exist). When the user types a file path, autocomplete from the current workspace tree. This is a standard readline extension and closes a major UX gap with minimal engineering cost.

---

## Gap 7 — Performance and Token Efficiency

### Root Cause

The IMPL plan includes basic compaction. The gap doc identifies that the official system has infrastructure-level latency optimisations and advanced token compression that cannot be replicated client-side.

### Optimal Fill Strategy

**7.1 — Semantic Context Pruning**

Before each API call, run a local *relevance scoring pass* on the conversation history:

1. Embed each message using a lightweight local model (e.g., `fastembed` or a small GGUF via `llama.cpp` bindings)
2. Score each message's relevance to the current user input
3. Drop or summarise messages below a relevance threshold before sending to the API

This is not token compression — it is *intelligent context selection*. It reduces tokens sent per call by 20–40% on long sessions and improves response quality by keeping the context focused.

**7.2 — Response Streaming Optimisation**

The IMPL plan has streaming support. Extend it with *predictive rendering*: begin rendering markdown structure (headers, code blocks) as soon as their opening tokens arrive, rather than waiting for each block to close. This is a pure TUI change that makes streaming responses *feel* significantly faster.

**7.3 — Tool Result Compression**

Large tool outputs (file reads, bash output, web fetches) are frequently sent back to the model verbatim. Add a `ToolOutputCompressor` that:
- For file reads > 200 lines: sends a summary + relevant excerpt rather than the full file
- For bash output > 100 lines: sends first 30, last 30, and a line count summary
- For web fetches: strips HTML boilerplate, extracts main content only

This is the single highest-ROI token efficiency improvement available client-side. It reduces average tokens per turn by 30–50% on file-heavy workflows.

---

## Gap 8 — Skills and Knowledge Layer

### Root Cause

Phase 8 builds a skill registry and bundled skills. The residual gap is in *skill intelligence* — skills that adapt their behaviour dynamically rather than running statically.

### Optimal Fill Strategy

**8.1 — Skill Parameterisation**

Skills in the current plan are essentially read-only prompt files (`SKILL.md`). Extend the skill spec to support *typed input parameters*:

```yaml
# skill.yaml
name: refactor-module
description: Refactor a Rust module for readability
parameters:
  - name: target_module
    type: file_path
    required: true
  - name: style_guide
    type: enum
    options: [clippy, custom]
    default: clippy
```

When the model invokes a skill, the runtime resolves parameters, validates types, and injects resolved values into the skill's prompt template. This transforms skills from static documents into *typed callable procedures*.

**8.2 — Skill Composition**

Allow skills to declare dependencies on other skills:

```yaml
depends_on:
  - read-architecture-docs
  - identify-test-coverage
```

The runtime executes dependency skills first, passes their outputs as context to the dependent skill. This enables complex multi-step skills without requiring custom code — skills become composable building blocks.

**8.3 — Skill Performance Telemetry (Local)**

Track skill execution outcomes locally (no external telemetry):
- Did the skill complete without error?
- Did the user manually correct the output?
- How many tokens did the skill consume?

Aggregate this into a local `skill_stats.json`. Over time, this data reveals which skills underperform and guides prioritisation of improvements — approximating the official system's feedback loop from real-world usage.

---

## Gap 9 — Reliability and Edge Case Handling

### Root Cause

The IMPL plan has error handling. But the gap doc notes that the official system has been hardened by thousands of real-world scenarios. Claw Code starts from zero real-world exposure.

### Optimal Fill Strategy

**9.1 — Chaos Testing Framework**

Add a `chaos` feature flag to the test harness that randomly injects:
- Tool call timeouts
- Partial streaming responses
- Malformed JSON from API
- MCP server disconnections mid-session
- Context window overflow scenarios

Run this suite continuously in CI. This proactively discovers edge cases before real users do, compressing the "limited real-world exposure" timeline.

**9.2 — Structured Error Taxonomy**

Define a typed `ClawError` enum with rich metadata rather than stringly-typed errors:

```rust
pub enum ClawError {
    ToolCallFailed { tool: String, reason: ToolFailReason, retry_eligible: bool },
    ContextWindowExceeded { tokens_used: usize, limit: usize },
    McpDisconnected { server: String, last_successful_call: Option<Instant> },
    // ...
}
```

Each variant includes `retry_eligible`, `user_actionable_hint`, and `telemetry_category`. This makes error handling consistent across the codebase and gives users actionable feedback rather than raw Rust panics or opaque strings.

**9.3 — Session Replay**

Implement session serialisation that records not just conversation history but the *exact tool call sequence with all inputs and outputs*. This allows:
- Reproducing bugs exactly from a user-reported session
- Running regression tests against real-world scenarios
- Building a curated test suite from production edge cases over time

---

## Gap 10 — Ecosystem and Network Effects

### Root Cause

This is the hardest gap. Network effects compound over time and cannot be engineered directly.

### Optimal Fill Strategy

**10.1 — Interoperability as a First-Class Feature**

Position Claw Code as the *most interoperable* AI coding tool. Official Claude Code is tied to Anthropic's ecosystem. Claw Code can be the tool that works with everything:

- Any LLM provider (already in plan)
- Any MCP server (already in plan)
- Import/export conversation history in Claude Code's format (enable migration both ways)
- VS Code extension, JetBrains plugin, Neovim plugin (priority additions to Phase 10)

Interoperability is a network effect multiplier: every integration brings users from that ecosystem.

**10.2 — Telemetry-Free as a Differentiator**

Make "no telemetry, fully local, air-gap capable" a marketing headline, not just a feature. For enterprise and regulated industries, this is not a consolation prize for missing features — it is a hard requirement that the official system cannot match (as it is a cloud service). The security gap (Gap 5) and this positioning become mutually reinforcing.

**10.3 — Plugin/Skill Marketplace with Quality Gates**

When launching the Phase 10 marketplace, implement a lightweight automated quality gate:
- Does the plugin manifest validate?
- Does it pass a basic sandbox escape test?
- Does it have a README?
- Is it pinned to a specific Claw Code schema version?

Plugins that pass get a `verified` badge automatically. This is not a security audit, but it filters out obviously broken plugins and creates a quality floor that improves the perceived ecosystem quality faster than manual curation.

**10.4 — Embed in Developer Workflows (Not Just a CLI)**

The fastest path to ecosystem growth is to meet developers where they already are:

| Integration | Value |
|---|---|
| GitHub Actions | Run Claw Code as a CI step for code review |
| Pre-commit hook | Auto-run security review on staged files |
| VS Code task | Invoke Claw Code from command palette |
| Git post-commit | Auto-generate commit summaries |

Each of these is a thin wrapper around existing Claw Code functionality but puts the tool into a workflow that runs automatically, creating daily active usage without requiring the developer to consciously open a terminal.

---

## Summary: Prioritised Gap Fill Roadmap

Not all gaps are equal. Below is a prioritised order based on **impact / implementation cost ratio**:

| Priority | Gap | Optimal Fill | Phase to Add To |
|---|---|---|---|
| 🔴 P0 | Tool output compression | `ToolOutputCompressor` module | Phase 2 |
| 🔴 P0 | Failure recovery re-framing | `OnToolFailure` hook type | Phase 1 |
| 🔴 P0 | Role-specialised agent personas | Role prompt overlays via `PromptEngine` | Phase 5 |
| 🔴 P0 | Structured agent communication | Message envelope protocol | Phase 5 |
| 🟠 P1 | Prompt engineering layer (`PromptEngine` crate) | Composable prompt pipeline | Phase 0 addition |
| 🟠 P1 | Policy-as-code rule engine | `policy.yaml` DSL + PreToolUse enforcement | Phase 9 |
| 🟠 P1 | Proactive context compaction | Auto-trigger at 60% context usage | Phase 6 |
| 🟠 P1 | Bundled MCP servers (top 10) | First-party lightweight MCP servers | Phase 4 |
| 🟠 P1 | Hierarchical plan trees | Typed `PlanNode` tree + tree executor | Phase 3 |
| 🟡 P2 | WASM plugin sandboxing | `wasmtime`/`wasmer` plugin runner | Phase 1 addition |
| 🟡 P2 | Skill parameterisation | Typed skill parameters + YAML spec | Phase 8 |
| 🟡 P2 | Semantic context pruning | Local embedding relevance scoring | Phase 6 addition |
| 🟡 P2 | Chaos testing framework | `chaos` feature flag in test harness | Phase 0 addition |
| 🟡 P2 | MCP capability negotiation | `CapabilityNegotiator` + prompt injection | Phase 4 |
| 🟢 P3 | Session replay | Full tool call serialisation | Phase 7 |
| 🟢 P3 | HMAC audit log signing | Signed append-only audit entries | Phase 9 |
| 🟢 P3 | MCP server registry + `/mcp search` | Community registry + CLI commands | Phase 10 |
| 🟢 P3 | GitHub Actions / CI integration | Thin wrapper + documentation | Phase 10 |

---

## Architectural Addition: `PromptEngine` Crate

This is the single highest-leverage addition not present in the current IMPL plan. It should be added as `crates/prompt-engine/` and treated as a core crate alongside `runtime` and `tools`. Its impact spans Gaps 1, 2, 6, and 8.

```
crates/prompt-engine/
├── src/
│   ├── lib.rs
│   ├── base.rs              // BaseSystemPrompt builder
│   ├── task_context.rs      // Task type detection + injection
│   ├── tool_primer.rs       // Tool availability + hint injection
│   ├── workspace_primer.rs  // Repo context injection
│   ├── model_adapter.rs     // Per-provider prompt adaptation
│   ├── role_overlay.rs      // Agent role prompt overlays
│   └── guardrail.rs         // Policy-aware constraint injection
└── tests/
    ├── base_test.rs
    └── adapter_test.rs
```

Every API call in the `runtime` crate should pass through `PromptEngine::build()` rather than constructing system prompts inline. This single architectural change enables systematic improvement of model behaviour across every gap without touching the core conversation loop.

---

*Analysis based on: `IMPLEMENTATION_PLAN.md` (2026-04-02) and Residual Gap Analysis document.*
*Strategic objective: Close addressable gaps, differentiate on non-addressable ones.*
