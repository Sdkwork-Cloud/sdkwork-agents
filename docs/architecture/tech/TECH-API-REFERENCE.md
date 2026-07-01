# SDKWork Agents API Reference

Status: active
Owner: agents-platform
Updated: 2026-06-28
Specs: API_SPEC.md, SDK_SPEC.md
Canonical list: [TECH-api-specification.md](TECH-api-specification.md)

## 1. Overview

sdkwork-agents exposes three HTTP API surfaces over OpenAPI 3.1.2. All operations
are metadata-driven through `api.rs` `ApiOperation` entries and materialized into
per-surface OpenAPI specifications.

| Surface | Prefix | Audience | OpenAPI authority | Operations |
| --- | --- | --- | --- | --- |
| Open API | `/agent/v3/api` | Third-party integrators | `sdkwork-agents-open-api` | 22 |
| App API | `/app/v3/api` | PC/H5/Flutter/Mini Program | `sdkwork-agents-app-api` | 25 |
| Backend API | `/backend/v3/api` | Admin backend | `sdkwork-agents-backend-api` | 23 |

**Grand total: 70 HTTP operations** across all surfaces. The authoritative
operation matrix lives in [TECH-api-specification.md](TECH-api-specification.md).

### 1.1 App-only runtime catalog APIs

| Method | Path | operationId | Purpose |
| --- | --- | --- | --- |
| GET | `/app/v3/api/ai/code_engines` | `agents.codeEngines.list` | Canonical code-engine catalog (`sdkwork-agents-runtime-facade`) |
| GET | `/app/v3/api/ai/mcp_servers` | `agents.mcpServers.list` | MCP composition-slot marketplace projection (references `sdkwork-mcp`) |

## 2. SDK Families

| SDK family | TypeScript package | API authority | Prefix |
| --- | --- | --- | --- |
| `sdkwork-agents-sdk` | `@sdkwork/agents-sdk` | `sdkwork-agents-open-api` | `/agent/v3/api` |
| `sdkwork-agents-app-sdk` | `@sdkwork/agents-app-sdk` | `sdkwork-agents-app-api` | `/app/v3/api` |
| `sdkwork-agents-backend-sdk` | `@sdkwork/agents-backend-sdk` | `sdkwork-agents-backend-api` | `/backend/v3/api` |

All SDK families are generated from the authority OpenAPI via `sdks/workspace-agent-sdkgen.mjs`.
Source OpenAPI files are authored under
`crates/sdkwork-intelligence-agents-service/specs/openapi/` and synced to
`sdks/sdkwork-agents-*/openapi/`.

## 3. Chat API Design (Production Contract)

Chat is a first-class agents-owned capability. Session and message persistence
live in `ai_agent_session` and `ai_agent_message`. Runtime turns are orchestrated
by `AgentsService::send_chat_message` through the pluggable `ChatCompleter` port
(default: `ContractChatCompleter`; production: mount `KernelModelChatCompleter` via
`AgentHttpState::with_chat_completer` at gateway bootstrap).

### 3.1 Session lifecycle

| Method | Path suffix | operationId | Notes |
| --- | --- | --- | --- |
| GET | `/ai/agents/{agentId}/sessions` | `agents.sessions.list` | Paginated list |
| POST | `/ai/agents/{agentId}/sessions` | `agents.sessions.create` | Server may generate `session.{id}` |
| GET | `/ai/agents/{agentId}/sessions/{sessionId}` | `agents.sessions.retrieve` | |
| POST | `/ai/agents/{agentId}/sessions/{sessionId}/close` | `agents.sessions.close` | |
| POST | `/ai/agents/{agentId}/sessions/{sessionId}/archive` | `agents.sessions.archive` | Backend only |

### 3.2 Chat completion (send message)

| Method | Path suffix | operationId | Semantics |
| --- | --- | --- | --- |
| POST | `.../sessions/{sessionId}/messages` | `agents.messages.create` | User sends `content`; service persists user + assistant messages and returns `AgentChatCompletionResponse` |
| GET | `.../sessions/{sessionId}/messages` | `agents.messages.list` | Ordered transcript |
| GET | `.../messages/{messageId}` | `agents.messages.retrieve` | Single message |

**App surface request body** (`AppSendAgentChatMessageRequest`): only `content`,
`requestedAt`, and optional `modelId` / `metadataJson`. IDs and roles are assigned
server-side.

**Open/Backend surface request body** (`SendAgentChatMessageRequest`): includes
`tenantId` for trusted tenant reconciliation.

### 3.3 Response shape

Chat completion returns a single-resource payload wrapped in the
`SdkWorkApiResponse` envelope (`ResourceData<AgentChatCompletionResponse>`):

```json
{
  "code": 0,
  "data": {
    "item": {
      "session": { "sessionId": "session.…", "messageCount": "2", … },
      "userMessage": { "role": "user", "content": "…" },
      "assistantMessage": { "role": "assistant", "content": "…" }
    }
  },
  "traceId": "<server-uuid>"
}
```

SSE streaming responses carry one `SdkWorkApiResponse` envelope per event,
keeping `code`/`traceId` consistent with the JSON path. Errors render as
`application/problem+json` with numeric `code` per `API_SPEC.md` §15.3.

### 3.4 Permissions

| operationId | Permission |
| --- | --- |
| `agents.sessions.list` | `agent.business.session.list` |
| `agents.sessions.create` | `agent.business.session.create` |
| `agents.sessions.retrieve` | `agent.business.session.retrieve` |
| `agents.sessions.close` | `agent.business.session.close` |
| `agents.sessions.archive` | `agent.business.session.archive` |
| `agents.messages.list` | `agent.business.message.list` |
| `agents.messages.create` | `agent.business.message.create` |
| `agents.messages.retrieve` | `agent.business.message.retrieve` |

## 4. Surface Comparison Matrix

| Operation | Open | App | Backend |
| --- | --- | --- | --- |
| `agents.list` | ✓ | ✓ | ✓ |
| `agents.create` | ✓ | ✓ | ✓ |
| `agents.retrieve` | ✓ | ✓ | ✓ |
| `agents.update` | ✓ | ✓ | ✓ |
| `agents.delete` | ✓ | ✓ | — |
| `agents.restore` | — | ✓ | ✓ |
| `agents.status.update` | — | — | ✓ |
| `agents.auditEvents.list` | — | — | ✓ |
| `agents.providerBindings.*` | ✓ | ✓ | ✓ |
| `agents.compositionSlots.*` | ✓ | ✓ | ✓ |
| `agents.previewResponses.create` | ✓ | ✓ | — |
| `agents.promptOptimizations.create` | ✓ | ✓ | — |
| `agents.sessions.*` | ✓ | ✓ | ✓ |
| `agents.sessions.archive` | — | — | ✓ |
| `agents.messages.*` | ✓ | ✓ | ✓ |
| `agents.codeEngines.list` | — | ✓ | — |
| `agents.mcpServers.list` | — | ✓ | — |

### 4.1 Catalog permissions

| operationId | Permission |
| --- | --- |
| `agents.codeEngines.list` | `agent.business.code_engine.list` |
| `agents.mcpServers.list` | `agent.business.mcp_server.list` |

## 5. Runtime Facade API (Rust crate, not HTTP)

`sdkwork-agents-runtime-facade` is the product-neutral Rust facade that product
repositories (BirdCoder, IM PC, etc.) must depend on instead of importing
`sdkwork-agent-provider-*` or `sdkwork-agent-kernel` directly.

Code-engine bootstrap, catalog, turn execution, and live interaction are documented
in [TECH_ARCHITECTURE.md](TECH_ARCHITECTURE.md). HTTP chat completion and code-engine
turns are complementary: HTTP for managed session persistence and client SDKs; the
facade for in-process multi-engine orchestration.

## 6. Database Tables (6 tables, agents-owned chat + composition)

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Agent identity, manifest, lifecycle |
| `ai_agent_runtime_binding` | Provider/runtime binding |
| `ai_agent_composition_slot` | Cross-module resource references |
| `ai_agent_audit_event` | Immutable audit log |
| `ai_agent_session` | Chat session lifecycle and counters |
| `ai_agent_message` | Ordered session transcript |

## 7. Integration Boundary for Product Applications

```text
Product Application (e.g., sdkwork-birdcoder)
    │
    ├── Frontend ──→ @sdkwork/agents-app-sdk
    │                POST .../sessions
    │                POST .../sessions/{id}/messages  (chat completion)
    │
    ├── Rust server ──→ sdkwork-agents-runtime-facade (code engines only)
    │
    └── MUST NOT directly depend on:
        ├── sdkwork-agent-kernel
        ├── sdkwork-agent-provider-*
        └── sdkwork-code-kernel
```

### Verification

```powershell
# sdkwork-agents
pnpm verify
pnpm check
cargo test -p sdkwork-intelligence-agents-service --features http-axum

# OpenAPI surface sync (after editing service specs)
node crates/sdkwork-intelligence-agents-service/scripts/sync-chat-openapi-surfaces.mjs
node sdks/workspace-agent-sdkgen.mjs --mode apply

# sdkwork-birdcoder (consumer)
cargo test -p sdkwork-birdcoder-kernel-bridge
cargo test -p sdkwork-agents-runtime-facade
```

`sync-chat-openapi-surfaces.mjs` projects session/message paths and schemas from `agents-open-api.openapi.yaml`
onto app/backend surfaces. It uses anchor insertion (not `String.replace` substitution strings) so regex
patterns ending in `$` are not corrupted. Backend paths map `TenantIdQuery` → `TenantId`; app paths strip
tenant query parameters.

## 8. OpenAPI Source Files

| Surface | Source OpenAPI | SDK OpenAPI |
| --- | --- | --- |
| Open API | `crates/sdkwork-intelligence-agents-service/specs/openapi/agents-open-api.openapi.yaml` | `sdks/sdkwork-agents-sdk/openapi/sdkwork-agents-open-api.openapi.yaml` |
| App API | `crates/sdkwork-intelligence-agents-service/specs/openapi/agents-app-api.openapi.yaml` | `sdks/sdkwork-agents-app-sdk/openapi/sdkwork-agents-app-api.openapi.yaml` |
| Backend API | `crates/sdkwork-intelligence-agents-service/specs/openapi/agents-backend-api.openapi.yaml` | `sdks/sdkwork-agents-backend-sdk/openapi/sdkwork-agents-backend-api.openapi.yaml` |
