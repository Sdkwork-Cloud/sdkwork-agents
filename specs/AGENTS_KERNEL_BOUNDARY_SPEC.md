# SDKWork Agents — Kernel Boundary Specification

- Version: 1.0.0
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: agents-platform
- Related:
  - [`../sdkwork-kernel/specs/AGENT_KERNEL_SPEC.md`](../../sdkwork-kernel/specs/AGENT_KERNEL_SPEC.md)
  - [`../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md`](../../sdkwork-kernel/specs/AGENT_PROVIDER_INTEGRATION_SPEC.md)
  - [`AGENTS_PROVIDER_TAXONOMY_SPEC.md`](./AGENTS_PROVIDER_TAXONOMY_SPEC.md)
  - [`docs/architecture/AGENTS_LAYERING.md`](../docs/architecture/AGENTS_LAYERING.md)

## 1. Purpose

This spec freezes the responsibility boundary between **sdkwork-kernel** (agent runtime
mechanism) and **sdkwork-agents** (managed-agent business application). Product
applications such as **sdkwork-birdcoder** MUST consume agent capabilities only through
`sdkwork-agents` HTTP/SDK surfaces or `sdkwork-agents-runtime-facade`, never by depending
on `sdkwork-agent-provider-*` crates directly.

## 2. Layering Model

```text
product applications (BirdCoder PC, IM, custom hosts)
        │
        ▼
sdkwork-agents  ── HTTP: /agent|app|backend/v3/api ── SDK: @sdkwork/agents-*-sdk
        │              runtime-facade (code-engine host / turn / catalog / live interaction)
        ▼
sdkwork-kernel  ── AgentRuntime SPI, provider plugins, internal /internal/v3/api
        │
        ▼
external agent frameworks (Codex, Claude Code, OpenCode, OpenClaw, Hermes, Rig, …)
```

### 2.1 sdkwork-kernel owns (mechanism only)

| Concern | Authority |
| --- | --- |
| Agent object model | `AgentSession`, `AgentTask`, `AgentRun`, `AgentStep`, `AgentMessage`, `AgentPart` |
| Provider SPI | model, tool, MCP, skill, memory context, knowledge, planning, host, policy, telemetry |
| Provider plugins | `sdkwork-agent-provider-{codex,claude-code,opencode,openclaw,hermes,rig,gemini-cli}` |
| Protocol adapters | external wire → kernel object mapping |
| Kernel operational HTTP | `/health`, `/metrics`, `/internal/v3/api/...` |
| Kernel runtime DB (optional) | `sdkwork-agent-database` for in-process runtime state when host does not persist |

Kernel MUST NOT own:

- Managed-agent CRUD, marketplace, tenant policy, or composition-slot business rules
- Application `ai_*` tables or migrations
- Open/App/Backend business HTTP surfaces (`/agent/v3/api`, `/app/v3/api`, `/backend/v3/api`)
- Product UI, workbench projection, or coding-session dialect

### 2.2 sdkwork-agents owns (business application)

| Concern | Authority |
| --- | --- |
| Managed-agent composition plane | `ai_agent`, `ai_agent_runtime_binding`, `ai_agent_composition_slot`, `ai_agent_audit_event` |
| Hosted chat persistence | `ai_agent_session`, `ai_agent_message` |
| Live interaction persistence | `ai_agent_interaction` (approval / user-question flows) |
| Task persistence | `ai_agent_task`; `ai_agent_task_run` is non-GA scope until kernel `AgentRun` projection is stable |
| HTTP surfaces | Open API (27 ops), App API (35 ops), Backend API (33 ops) |
| SDK families | `@sdkwork/agents-sdk`, `@sdkwork/agents-app-sdk`, `@sdkwork/agents-backend-sdk` |
| Runtime facade | `sdkwork-agents-runtime-facade` — sole agents→kernel bridge for products |
| Independent module integration | composition slots → memory, knowledgebase, skills, prompts, drive, MCP; runtime binding/profile → LLM |

Agents MUST NOT:

- Redefine kernel SPI types in application DTOs when kernel types are the semantic source
- Embed provider-specific fields in `ai_agent` without namespacing (`manifest_json`, `policy_json`)
- Bypass kernel provider SPI with raw HTTP to Codex/Claude/OpenCode when an official SDK binding exists
- Make `sdkwork-memory`, `sdkwork-knowledgebase`, `sdkwork-skills`, `sdkwork-prompts`, `sdkwork-mcp`, `sdkwork-llm`, or `sdkwork-drive` depend on `sdkwork-agents`
- Duplicate independent-module tables or business APIs inside the agents managed store

## 3. Database Boundary

| Store | Owner | Env prefix | Table prefix | Purpose |
| --- | --- | --- | --- | --- |
| Agents managed store | `sdkwork-agents` | `SDKWORK_AGENTS_STORE_DATABASE_*` | `ai_*` | Composition plane + hosted sessions/messages/interactions/tasks |
| Agents app metadata | `sdkwork-agents` | `SDKWORK_AGENTS_DATABASE_*` | app metadata | Application host DB |
| Kernel runtime store | `sdkwork-kernel` | `SDKWORK_AGENT_SERVER_DATABASE_*` | kernel-owned | In-process runtime when host delegates persistence to kernel |

**Rule:** All product-visible session/message query APIs are backed by `ai_agent_session` /
`ai_agent_message` in the agents managed store. Kernel runtime DB is an implementation
detail for provider execution, not the canonical product read model.

## 4. Runtime Facade Contract

`sdkwork-agents-runtime-facade` is the only supported integration path for:

| Capability | Facade module | Kernel dependency |
| --- | --- | --- |
| Code-engine bootstrap | `code_engines.rs` | `sdkwork-agent-provider-*` via SPI |
| Turn execution | `turn.rs` | `ModelProvider::invoke` |
| Engine catalog | `engine_catalog.rs` | bootstrapped provider slots |
| Live interaction | `live_interaction.rs` | approval / user-question registry (transitioning to kernel SPI) |
| Runtime host | `runtime_host.rs` | aggregates slots + live registry |

Products (BirdCoder) depend on `sdkwork-agents-runtime-facade`, not `sdkwork-agent-kernel`.

## 5. Independent Module Integration

Agents references independent capability modules through composition slots or runtime
bindings. The dependency direction is always `sdkwork-agents -> independent module`.

| Capability | Integration in agents | Owner module | Contract |
| --- | --- | --- | --- |
| Memory | `slot_kind=memory`, `target_module=memory` | `sdkwork-memory` | `@sdkwork/memory-app-sdk` (when mounted) |
| Knowledgebase / RAG | `slot_kind=knowledge`, `target_module=knowledgebase` | `sdkwork-knowledgebase` | `@sdkwork/knowledgebase-app-sdk` |
| Skills | `slot_kind=skill`, `target_module=skills` | `sdkwork-skills` | `@sdkwork/skills-app-sdk` |
| Prompts | `slot_kind=prompt`, `target_module=prompts` | `sdkwork-prompts` | `@sdkwork/prompts-app-sdk` |
| MCP | `slot_kind=mcp`, `target_module=mcp` | `sdkwork-mcp` | marketplace projection + future federation |
| LLM | `ai_agent_runtime_binding`, `configuration_profile_id`, model provider profile | `sdkwork-llm` + kernel model provider | model catalog / provider profile / credential references |
| Drive | `slot_kind=drive`, `target_module=drive` | `sdkwork-drive` | `@sdkwork/drive-app-sdk` (Drive Uploader only) |

Agents does not duplicate independent-module tables or HTTP authorities.

## 6. Forbidden Dependencies

| Consumer | Forbidden direct dependency | Required path |
| --- | --- | --- |
| `sdkwork-birdcoder` | `sdkwork-agent-kernel`, `sdkwork-agent-provider-*` | `sdkwork-agents-runtime-facade` + `@sdkwork/agents-app-sdk` |
| `sdkwork-agents` PC/H5/MP clients | generator transport package names | `@sdkwork/agents-app-sdk` composed facade |
| Any product | raw HTTP to `/internal/v3/api` | agents open/app/backend SDK |

Enforced by: `specs/agents-birdcoder-alignment.spec.json`, BirdCoder `birdcoder-agents-integration-contract.test.mjs`.

## 7. Verification

```powershell
cargo test -p sdkwork-agents-runtime-facade
cargo test -p sdkwork-intelligence-agents-service --features http-axum
node ../sdkwork-birdcoder/scripts/birdcoder-agents-integration-contract.test.mjs
pnpm check:architecture-alignment
```
