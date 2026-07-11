# SDKWork Agents API Reference

Status: active
Owner: agents-platform
Updated: 2026-07-06
Specs: API_SPEC.md, SDK_SPEC.md
Canonical list: [TECH-api-specification.md](TECH-api-specification.md)

## 1. Overview

sdkwork-agents exposes three HTTP API surfaces over OpenAPI 3.1.2. All operations
are metadata-driven through `api.rs` `ApiOperation` entries and materialized into
per-surface OpenAPI specifications.

| Surface | Prefix | Audience | OpenAPI authority | Operations |
| --- | --- | --- | --- | --- |
| Open API | `/agent/v3/api` | Third-party integrators | `sdkwork-agents-open-api` | 27 |
| App API | `/app/v3/api` | PC/H5/Flutter/Mini Program | `sdkwork-agents-app-api` | 35 |
| Backend API | `/backend/v3/api` | Admin backend | `sdkwork-agents-backend-api` | 33 |

**Grand total: 95 HTTP operations** across all surfaces. The authoritative
operation matrix lives in [TECH-api-specification.md](TECH-api-specification.md).

### 1.1 App-only runtime catalog APIs

| Method | Path | operationId | Purpose |
| --- | --- | --- | --- |
| GET | `/app/v3/api/ai/code_engines` | `agents.codeEngines.list` | Canonical code-engine catalog (`sdkwork-agents-runtime-facade`) |
| GET | `/app/v3/api/ai/mcp_servers` | `agents.mcpServers.list` | MCP composition-slot marketplace projection (paginated; supports `q`) |

### 1.2 Scheduled tasks

| Method | Path suffix | operationId | Notes |
| --- | --- | --- | --- |
| GET | `/ai/agents/{agentId}/tasks` | `agents.tasks.list` | Paginated (`page`, `page_size`) |
| POST | `/ai/agents/{agentId}/tasks` | `agents.tasks.create` | |
| GET | `/ai/agents/{agentId}/tasks/{taskId}` | `agents.tasks.retrieve` | |
| POST | `/ai/agents/{agentId}/tasks/{taskId}/cancel` | `agents.tasks.cancel` | Optimistic concurrency via `expectedVersion` |
| POST | `/ai/agents/{agentId}/tasks/{taskId}/execute` | `agents.tasks.execute` | Run deferred `pending` task |

### 1.3 Live interactions (App + Backend)

Code-engine pause points (approval / user question) under a chat session. Not
exposed on Open API.

| Method | Path suffix | operationId | Notes |
| --- | --- | --- | --- |
| GET | `.../sessions/{sessionId}/interactions` | `agents.interactions.list` | Paginated (`page`, `page_size`) |
| POST | `.../sessions/{sessionId}/interactions` | `agents.interactions.create` | `kind`: `approval` \| `user_question` |
| GET | `.../interactions/{interactionId}` | `agents.interactions.retrieve` | |
| POST | `.../interactions/{interactionId}/approve` | `agents.interactions.approve` | Approval kind only; `expectedVersion` required |
| POST | `.../interactions/{interactionId}/answer` | `agents.interactions.answer` | User-question kind only; `answer` required when not rejected |

**Authorization (App API):** nested session/task/message/interaction handlers resolve
`owner_scope` from the authenticated app session and reject cross-owner access.
`path_agent_id` from the URL must match the persisted resource `agent_id` (404 when
mismatched). Interaction create/list/approve/answer follow the same owner-scope rules
as messages.

### 1.4 List filter validation

Invalid `status` or `role` query values on list endpoints return HTTP `400` with
`application/problem+json` (`code` `40001`) instead of silently ignoring the filter.
Applies to sessions, messages, interactions, and tasks list APIs.

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
(default: `ContractChatCompleter`; production: mount `RuntimeFacadeChatCompleter` via
`AgentHttpState::with_chat_completer` at gateway bootstrap so code-engine turns flow
through `sdkwork-agents-runtime-facade`).

### 3.1 Session lifecycle

| Method | Path suffix | operationId | Notes |
| --- | --- | --- | --- |
| GET | `/ai/agents/{agentId}/sessions` | `agents.sessions.list` | Paginated list |
| POST | `/ai/agents/{agentId}/sessions` | `agents.sessions.create` | Server may generate `session.{id}` |
| GET | `/ai/agents/{agentId}/sessions/{sessionId}` | `agents.sessions.retrieve` | |
| POST | `/ai/agents/{agentId}/sessions/{sessionId}/close` | `agents.sessions.close` | |
| POST | `/ai/agents/{agentId}/sessions/{sessionId}/archive` | `agents.sessions.archive` | Backend only; session must be `closed` first (400 if still active); idempotent when already archived |

Nested `GET` handlers for sessions, tasks, messages, and interactions validate that
the path `{agentId}` matches the stored resource `agent_id`.

### 3.2 Chat completion (send message)

| Method | Path suffix | operationId | Semantics |
| --- | --- | --- | --- |
| POST | `.../sessions/{sessionId}/messages` | `agents.messages.create` | User sends `content`; service persists user + assistant messages and returns `AgentChatCompletionResponse` |
| GET | `.../sessions/{sessionId}/messages` | `agents.messages.list` | Paginated transcript (`page`, `page_size`); default sort ascending by sequence; clients load the last page for the newest window and page backward for history |
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
| `agents.status.create` | — | — | ✓ |
| `agents.auditEvents.list` | — | — | ✓ |
| `agents.providerBindings.*` | ✓ | ✓ | ✓ |
| `agents.compositionSlots.*` | ✓ | ✓ | ✓ |
| `agents.previewResponses.create` | ✓ | ✓ | — |
| `agents.promptOptimizations.create` | ✓ | ✓ | — |
| `agents.sessions.*` | ✓ | ✓ | ✓ |
| `agents.sessions.archive` | — | — | ✓ |
| `agents.messages.*` | ✓ | ✓ | ✓ |
| `agents.interactions.*` | — | ✓ | ✓ |
| `agents.tasks.*` | ✓ | ✓ | ✓ |
| `agents.codeEngines.list` | — | ✓ | — |
| `agents.mcpServers.list` | — | ✓ | — |

### 4.1 Catalog permissions

| operationId | Permission |
| --- | --- |
| `agents.codeEngines.list` | `agent.business.code_engine.list` |
| `agents.mcpServers.list` | `agent.business.mcp_server.list` |
| `agents.interactions.list` | `agent.business.interaction.list` |
| `agents.interactions.create` | `agent.business.interaction.create` |
| `agents.interactions.retrieve` | `agent.business.interaction.retrieve` |
| `agents.interactions.approve` | `agent.business.interaction.approve` |
| `agents.interactions.answer` | `agent.business.interaction.answer` |
| `agents.tasks.list` | `agent.business.task.list` |

## 5. Runtime Facade API (Rust crate, not HTTP)

`sdkwork-agents-runtime-facade` is the product-neutral Rust facade that product
repositories (BirdCoder, IM PC, etc.) must depend on instead of importing
`sdkwork-agent-provider-*` or `sdkwork-agent-kernel` directly.

Code-engine bootstrap, catalog, turn execution, and live interaction are documented
in [TECH_ARCHITECTURE.md](TECH_ARCHITECTURE.md). HTTP chat completion and code-engine
turns are complementary: HTTP for managed session persistence and client SDKs; the
facade for in-process multi-engine orchestration.

## 6. Database Tables (8 tables, agents-owned chat + tasks + interactions)

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Agent identity, manifest, lifecycle |
| `ai_agent_runtime_binding` | Provider/runtime binding |
| `ai_agent_composition_slot` | Cross-module resource references |
| `ai_agent_audit_event` | Immutable audit log |
| `ai_agent_session` | Chat session lifecycle and counters |
| `ai_agent_message` | Ordered session transcript |
| `ai_agent_interaction` | Live interaction pause points (approval / user question) |
| `ai_agent_task` | Scheduled/async agent tasks |

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
pnpm check:api-envelope
pnpm check:api-operation-patterns
pnpm check:route-path-collisions
pnpm check:pagination
pnpm check:app-sdk-consumer-imports
cargo test -p sdkwork-intelligence-agents-service --features http-axum

# OpenAPI surface sync (after editing service specs)
node crates/sdkwork-intelligence-agents-service/scripts/sync-chat-openapi-surfaces.mjs
node sdks/workspace-agent-sdkgen.mjs --mode apply

# sdkwork-birdcoder (consumer)
cargo test --manifest-path ../sdkwork-birdcoder/Cargo.toml -p sdkwork-birdcoder-kernel-bridge
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
