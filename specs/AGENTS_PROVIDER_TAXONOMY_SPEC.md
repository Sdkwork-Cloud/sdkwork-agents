# SDKWork Agents — Provider Taxonomy Specification

- Version: 1.0.0
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: agents-platform
- Related:
  - [`../../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md`](../../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md)
  - [`../../sdkwork-kernel/specs/kernel-local-conventions.md`](../../sdkwork-kernel/specs/kernel-local-conventions.md)
  - [`AGENTS_KERNEL_BOUNDARY_SPEC.md`](./AGENTS_KERNEL_BOUNDARY_SPEC.md)

## 1. Purpose

Define a stable, industry-aligned taxonomy for agent provider families so product,
API, and documentation use the same vocabulary. Kernel implements providers; agents
exposes catalog and composition; BirdCoder consumes through agents.

## 2. Agent Family Tiers

| Tier | Family | Description | Examples |
| --- | --- | --- | --- |
| T1 | **Code agents** | IDE/CLI coding assistants with official SDK or typed transport | Codex, Claude Code, Gemini CLI, OpenCode |
| T2 | **Autonomous agents** | Self-directed execution agents with tool/MCP loops | OpenClaw, Hermes Agent |
| T3 | **Framework agents** | In-process agent frameworks wrapped as kernel plugins | Rig (`rig-rust`) |
| T4 | **Orchestration frameworks** | Declared via `implementation_type` on `ai_agent`; future provider bindings | LangGraph, CrewAI, AutoGen, Semantic Kernel, OpenAI Agents |

## 3. Kernel Provider Matrix

| engine_key / provider | Kernel crate | binding_id | SDK integration | Transport | Agents catalog tier | BirdCoder default |
| --- | --- | --- | --- | --- | --- | --- |
| `codex` | `sdkwork-agent-provider-codex` | `binding.agent-provider.codex` | Official TypeScript SDK + Rust native | `typescript_node`, `rust_native`, `ipc_protocol` | T1 canonical | Yes |
| `claude-code` | `sdkwork-agent-provider-claude-code` | `binding.agent-provider.claude-code` | Official SDK | `typescript_node` | T1 canonical | Yes |
| `gemini` | `sdkwork-agent-provider-gemini-cli` | `binding.agent-provider.gemini-cli` | Official CLI/SDK | `typescript_node` | T1 canonical | Yes |
| `opencode` | `sdkwork-agent-provider-opencode` | `binding.agent-provider.opencode` | Official SDK + OpenAPI | `typescript_node`, `http_openapi` | T1 canonical | Yes |
| `openclaw` | `sdkwork-agent-provider-openclaw` | `binding.agent-provider.openclaw` | Plugin binding (SDK when available) | plugin transport | T2 extended | Opt-in |
| `hermes` | `sdkwork-agent-provider-hermes` | `binding.agent-provider.hermes` | Plugin binding | plugin transport | T2 extended | Opt-in |
| `rig` | `sdkwork-agent-provider-rig` | `binding.agent-provider.rig` | `rust_crate` + `source_tree` | in-process | T3 framework | Via `implementation_type` |

**Canonical catalog (T1):** `codex`, `claude-code`, `gemini`, `opencode` — bootstrapped by
default in `sdkwork-agents-runtime-facade` and exposed via `GET /app/v3/api/ai/code_engines`.

**Extended catalog (T2):** `openclaw`, `hermes` — bootstrap supported on demand; not in
default host bootstrap until sibling SDK/conformance gates pass in CI.

**Framework (T3):** Rig — selected via `ai_agent.implementation_type = rig-rust` and kernel
plugin registration; not listed in code-engine catalog.

## 4. SPI Coverage By Capability

| Capability | Kernel SPI spec | Codex | Claude Code | OpenCode | OpenClaw | Hermes | Rig |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Model invoke | `AGENT_MODEL_PROVIDER_SPI_SPEC.md` | SDK | SDK | SDK | plugin | plugin | crate |
| Tool call | `AGENT_TOOL_PROVIDER_SPI_SPEC.md` | Yes | Yes | Yes | Yes | Yes | Yes |
| MCP | `AGENT_MCP_PROVIDER_SPI_SPEC.md` | Via tools | Via tools | Native | Native | Native | Adapter |
| Memory context | `AGENT_CONTEXT_MEMORY_SPEC.md` | Via agents slot | Via agents slot | Via agents slot | Via agents slot | Via agents slot | Via agents slot |
| Knowledge | `AGENT_KNOWLEDGE_PROVIDER_SPI_SPEC.md` | Via agents slot | Via agents slot | Via agents slot | Via agents slot | Via agents slot | Plugin |
| Skills | `AGENT_SKILL_PROVIDER_SPI_SPEC.md` | Via agents slot | Via agents slot | Via agents slot | Via agents slot | Via agents slot | Plugin |
| Planning / tasks | `AGENT_PLANNING_EXECUTION_SPEC.md` | Partial | Partial | Partial | Full | Full | Full |
| Live interaction | facade → kernel (transition) | Approval | Approval | Permission + Q&A | TBD | TBD | Fail-closed default |
| Streaming | `ModelProvider::stream` | Planned | Planned | Planned | Planned | Planned | Planned |

Memory types (permanent, user-scoped, growth) are owned by `sdkwork-memory` and referenced
through `slot_kind=memory` composition slots — not kernel tables.

## 5. Domain Model Mapping (Product Language)

| Product term | Kernel object | Agents persistence |
| --- | --- | --- |
| Agent definition | `AgentManifest` | `ai_agent.manifest_json` |
| Agent configuration | `AgentConfiguration` | `ai_agent` + `ai_agent_runtime_binding` |
| Chat session | `AgentSession` (runtime) | `ai_agent_session` (hosted) |
| Message / turn | `AgentMessage` + `AgentPart` | `ai_agent_message` |
| Task | `AgentTask` | `ai_agent_task` |
| Run / step | `AgentRun` / `AgentStep` | Kernel runtime; future `ai_agent_task_run` projection |
| Tool approval | live interaction | `ai_agent_interaction` |
| External memory | `MemoryRecord` via provider | composition slot → `sdkwork-memory` |
| Code workspace | code-kernel extension | BirdCoder-owned (`coding_session*`) |

## 6. `implementation_type` Enum (Agents-owned)

Stored on `ai_agent.implementation_type`:

| Value | Maps to |
| --- | --- |
| `sdkwork-native` | Kernel native runtime (default) |
| `rig-rust` | `sdkwork-agent-provider-rig` |
| `openai-agents` | Future provider binding |
| `langchain` / `langgraph` / `crewai` / `autogen` / `semantic-kernel` | Declared; binding TBD |
| `custom` | Tenant-defined manifest + custom provider binding |

Code-engine `engine_key` is orthogonal: it selects the T1/T2 provider for turn execution
when `implementation_kind` is `process-adapter` or `protocol-adapter`.

## 7. Integration Policy

1. **Official SDK exists** → kernel provider MUST use SDK integration (`official_sdk` mode).
2. **No official SDK** → defer integration; document in gap analysis; do not add raw HTTP bypass.
3. **Product apps** → MUST use `@sdkwork/agents-app-sdk` for sessions/messages/catalog.
4. **BirdCoder** → coding-session dialect stays BirdCoder-owned; turn execution via agents facade.

## 8. Verification

```powershell
cargo test -p sdkwork-agents-runtime-facade
cargo test -p sdkwork-agent-provider-codex -p sdkwork-agent-provider-claude-code -p sdkwork-agent-provider-opencode
# from sdkwork-kernel workspace root
```
