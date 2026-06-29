# SDKWork Agents — Complete API Specification

Status: active
Owner: agents-platform
Updated: 2026-06-28
Specs: API_SPEC.md, SDK_SPEC.md, NAMING_SPEC.md

## Overview

SDKWork Agents exposes three HTTP API surfaces following the SDKWork `API_SPEC.md`
canonical prefix lock. Each surface has a distinct audience and security model.

| Surface | Prefix | Audience | Auth | SDK Family |
| --- | --- | --- | --- | --- |
| Open API | `/agent/v3/api` | Third-party integrators, public/domain API | API key | `sdkwork-agents-sdk` |
| App API | `/app/v3/api` | PC/H5/Flutter/Mini Program clients | Dual-token (IAM session) | `sdkwork-agents-app-sdk` |
| Backend API | `/backend/v3/api` | Admin console, internal operators | Dual-token + `agent.business.manage` | `sdkwork-agents-backend-sdk` |

All operations are metadata-driven via `ApiOperation` constants and materialized
into OpenAPI 3.1.2 documents for SDK generation.

---

## 1. Open API (`/agent/v3/api`)

Public integration surface for external API consumers.

### 1.1 Agent Management

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 1 | GET | `/agent/v3/api/ai/agents` | `agents.list` | List agents for the authenticated tenant |
| 2 | POST | `/agent/v3/api/ai/agents` | `agents.create` | Create a new agent |
| 3 | GET | `/agent/v3/api/ai/agents/{agent_id}` | `agents.retrieve` | Retrieve a single agent by ID |
| 4 | PATCH | `/agent/v3/api/ai/agents/{agent_id}` | `agents.update` | Update agent configuration |
| 5 | DELETE | `/agent/v3/api/ai/agents/{agent_id}` | `agents.delete` | Soft-delete an agent |

### 1.2 Composition Slots

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 6 | GET | `/agent/v3/api/ai/agents/{agent_id}/composition_slots` | `agents.compositionSlots.list` | List composition slots for an agent |
| 7 | POST | `/agent/v3/api/ai/agents/{agent_id}/composition_slots` | `agents.compositionSlots.create` | Create a composition slot binding |
| 8 | GET | `/agent/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.retrieve` | Retrieve a single composition slot |
| 9 | PATCH | `/agent/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.update` | Update a composition slot |
| 10 | DELETE | `/agent/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.delete` | Delete a composition slot |

### 1.3 Provider Bindings

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 11 | GET | `/agent/v3/api/ai/agents/{agent_id}/provider_bindings` | `agents.providerBindings.list` | List provider bindings for an agent |
| 12 | POST | `/agent/v3/api/ai/agents/{agent_id}/provider_bindings` | `agents.providerBindings.create` | Create a provider binding |
| 13 | POST | `/agent/v3/api/ai/agents/{agent_id}/provider_bindings/{binding_id}/activate` | `agents.providerBindings.activate` | Activate a provider binding |

### 1.4 Runtime Execution

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 14 | POST | `/agent/v3/api/ai/agents/{agent_id}/preview_responses` | `agents.previewResponses.create` | Preview an agent response without creating a session |
| 15 | POST | `/agent/v3/api/ai/agents/{agent_id}/prompt_optimizations` | `agents.promptOptimizations.create` | Optimize a prompt for an agent |

### 1.5 Sessions

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 16 | GET | `/agent/v3/api/ai/agents/{agent_id}/sessions` | `agents.sessions.list` | List chat sessions for an agent |
| 17 | POST | `/agent/v3/api/ai/agents/{agent_id}/sessions` | `agents.sessions.create` | Create a new chat session |
| 18 | GET | `/agent/v3/api/ai/agents/{agent_id}/sessions/{session_id}` | `agents.sessions.retrieve` | Retrieve a single session |

### 1.6 Messages

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 19 | GET | `/agent/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages` | `agents.messages.list` | List messages in a session |
| 20 | POST | `/agent/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages` | `agents.messages.create` | Send user content; returns chat completion (user + assistant messages) |
| 21 | GET | `/agent/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages/{message_id}` | `agents.messages.retrieve` | Retrieve a single message |
| 22 | POST | `/agent/v3/api/ai/agents/{agent_id}/sessions/{session_id}/close` | `agents.sessions.close` | Close a chat session |

**Open API total: 22 operations**

---

## 2. App API (`/app/v3/api`)

Application client surface for PC, H5, Flutter, and Mini Program apps.

### 2.1 Agent Management

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 1 | GET | `/app/v3/api/ai/agents` | `agents.list` | List agents for the current user |
| 2 | POST | `/app/v3/api/ai/agents` | `agents.create` | Create a new agent |
| 3 | GET | `/app/v3/api/ai/agents/{agent_id}` | `agents.retrieve` | Retrieve a single agent |
| 4 | PATCH | `/app/v3/api/ai/agents/{agent_id}` | `agents.update` | Update agent configuration |
| 5 | DELETE | `/app/v3/api/ai/agents/{agent_id}` | `agents.delete` | Soft-delete an agent |
| 6 | POST | `/app/v3/api/ai/agents/{agent_id}/restore` | `agents.restore` | Restore a soft-deleted agent |

### 2.2 Composition Slots

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 7 | GET | `/app/v3/api/ai/agents/{agent_id}/composition_slots` | `agents.compositionSlots.list` | List composition slots |
| 8 | POST | `/app/v3/api/ai/agents/{agent_id}/composition_slots` | `agents.compositionSlots.create` | Create a composition slot |
| 9 | GET | `/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.retrieve` | Retrieve a composition slot |
| 10 | PATCH | `/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.update` | Update a composition slot |
| 11 | DELETE | `/app/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.delete` | Delete a composition slot |

### 2.3 Provider Bindings

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 12 | GET | `/app/v3/api/ai/agents/{agent_id}/provider_bindings` | `agents.providerBindings.list` | List provider bindings |
| 13 | POST | `/app/v3/api/ai/agents/{agent_id}/provider_bindings` | `agents.providerBindings.create` | Create a provider binding |
| 14 | POST | `/app/v3/api/ai/agents/{agent_id}/provider_bindings/{binding_id}/activate` | `agents.providerBindings.activate` | Activate a provider binding |

### 2.4 Runtime Execution

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 15 | POST | `/app/v3/api/ai/agents/{agent_id}/preview_responses` | `agents.previewResponses.create` | Preview an agent response |
| 16 | POST | `/app/v3/api/ai/agents/{agent_id}/prompt_optimizations` | `agents.promptOptimizations.create` | Optimize a prompt |

### 2.5 Sessions

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 17 | GET | `/app/v3/api/ai/agents/{agent_id}/sessions` | `agents.sessions.list` | List sessions for an agent |
| 18 | POST | `/app/v3/api/ai/agents/{agent_id}/sessions` | `agents.sessions.create` | Create a new session |
| 19 | GET | `/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}` | `agents.sessions.retrieve` | Retrieve a session |
| 20 | POST | `/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/close` | `agents.sessions.close` | Close a session |

### 2.6 Messages

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 21 | GET | `/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages` | `agents.messages.list` | List messages in a session |
| 22 | POST | `/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages` | `agents.messages.create` | Send a message to an agent |
| 23 | GET | `/app/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages/{message_id}` | `agents.messages.retrieve` | Retrieve a single message |

**App API total: 25 operations**

### 2.7 Code Engine Catalog (App-only)

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 24 | GET | `/app/v3/api/ai/code_engines` | `agents.codeEngines.list` | List canonical code-engine catalog (runtime facade projection) |
| 25 | GET | `/app/v3/api/ai/mcp_servers` | `agents.mcpServers.list` | List MCP marketplace entries from agent composition slots |

---

## 3. Backend API (`/backend/v3/api`)

Admin/operations surface for management, auditing, and control-plane operations.

### 3.1 Agent Management

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 1 | GET | `/backend/v3/api/ai/agents` | `agents.list` | List all agents (admin) |
| 2 | POST | `/backend/v3/api/ai/agents` | `agents.create` | Create an agent (admin) |
| 3 | GET | `/backend/v3/api/ai/agents/{agent_id}` | `agents.retrieve` | Retrieve an agent (admin) |
| 4 | PATCH | `/backend/v3/api/ai/agents/{agent_id}` | `agents.update` | Update an agent (admin) |
| 5 | POST | `/backend/v3/api/ai/agents/{agent_id}/restore` | `agents.restore` | Restore a deleted agent |
| 6 | POST | `/backend/v3/api/ai/agents/{agent_id}/status` | `agents.status.update` | Change agent status (admin) |

### 3.2 Audit Events

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 7 | GET | `/backend/v3/api/ai/agents/{agent_id}/audit_events` | `agents.auditEvents.list` | List audit events for an agent |

### 3.3 Composition Slots

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 8 | GET | `/backend/v3/api/ai/agents/{agent_id}/composition_slots` | `agents.compositionSlots.list` | List composition slots (admin) |
| 9 | POST | `/backend/v3/api/ai/agents/{agent_id}/composition_slots` | `agents.compositionSlots.create` | Create a composition slot (admin) |
| 10 | GET | `/backend/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.retrieve` | Retrieve a composition slot (admin) |
| 11 | PATCH | `/backend/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.update` | Update a composition slot (admin) |
| 12 | DELETE | `/backend/v3/api/ai/agents/{agent_id}/composition_slots/{slot_id}` | `agents.compositionSlots.delete` | Delete a composition slot (admin) |

### 3.4 Provider Bindings

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 13 | GET | `/backend/v3/api/ai/agents/{agent_id}/provider_bindings` | `agents.providerBindings.list` | List provider bindings (admin) |
| 14 | POST | `/backend/v3/api/ai/agents/{agent_id}/provider_bindings` | `agents.providerBindings.create` | Create a provider binding (admin) |
| 15 | POST | `/backend/v3/api/ai/agents/{agent_id}/provider_bindings/{binding_id}/activate` | `agents.providerBindings.activate` | Activate a provider binding (admin) |

### 3.5 Sessions

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 16 | GET | `/backend/v3/api/ai/agents/{agent_id}/sessions` | `agents.sessions.list` | List sessions (admin) |
| 17 | POST | `/backend/v3/api/ai/agents/{agent_id}/sessions` | `agents.sessions.create` | Create a session (admin) |
| 18 | GET | `/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}` | `agents.sessions.retrieve` | Retrieve a session (admin) |
| 19 | POST | `/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}/close` | `agents.sessions.close` | Close a session (admin) |
| 20 | POST | `/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}/archive` | `agents.sessions.archive` | Archive a session (admin only) |

### 3.6 Messages

| # | Method | Path | operationId | Description |
| --- | --- | --- | --- | --- |
| 21 | GET | `/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages` | `agents.messages.list` | List messages (admin) |
| 22 | POST | `/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages` | `agents.messages.create` | Create a message (admin) |
| 23 | GET | `/backend/v3/api/ai/agents/{agent_id}/sessions/{session_id}/messages/{message_id}` | `agents.messages.retrieve` | Retrieve a message (admin) |

**Backend API total: 23 operations**

---

## 4. Summary

| Surface | Operations | Unique to Surface |
| --- | --- | --- |
| Open API | 22 | `previewResponses`, `promptOptimizations` |
| App API | 25 | `restore`, `previewResponses`, `promptOptimizations`, `codeEngines.list`, `mcpServers.list` |
| Backend API | 23 | `restore`, `status.update`, `auditEvents.list`, `sessions.archive` |
| **Grand Total** | **70** | |

### 4.1 Cross-Surface Operation Availability

| Operation | Open | App | Backend |
| --- | --- | --- | --- |
| agents.list | ✓ | ✓ | ✓ |
| agents.create | ✓ | ✓ | ✓ |
| agents.retrieve | ✓ | ✓ | ✓ |
| agents.update | ✓ | ✓ | ✓ |
| agents.delete | ✓ | ✓ | — |
| agents.restore | — | ✓ | ✓ |
| agents.status.update | — | — | ✓ |
| agents.auditEvents.list | — | — | ✓ |
| agents.compositionSlots.* | ✓ | ✓ | ✓ |
| agents.providerBindings.* | ✓ | ✓ | ✓ |
| agents.previewResponses.create | ✓ | ✓ | — |
| agents.promptOptimizations.create | ✓ | ✓ | — |
| agents.sessions.list | ✓ | ✓ | ✓ |
| agents.sessions.create | ✓ | ✓ | ✓ |
| agents.sessions.retrieve | ✓ | ✓ | ✓ |
| agents.sessions.close | ✓ | ✓ | ✓ |
| agents.sessions.archive | — | — | ✓ |
| agents.messages.list | ✓ | ✓ | ✓ |
| agents.messages.create | ✓ | ✓ | ✓ |
| agents.messages.retrieve | ✓ | ✓ | ✓ |
| agents.codeEngines.list | — | ✓ | — |
| agents.mcpServers.list | — | ✓ | — |

---

## 5. SDK Families

| SDK Family | Package Name | Surface | Language |
| --- | --- | --- | --- |
| `sdkwork-agents-sdk` | `@sdkwork/agents-sdk` | Open API | TypeScript |
| `sdkwork-agents-app-sdk` | `@sdkwork/agents-app-sdk` | App API | TypeScript |
| `sdkwork-agents-backend-sdk` | `@sdkwork/agents-backend-sdk` | Backend API | TypeScript |

SDK generation follows `SDK_SPEC.md` and `SDK_WORKSPACE_GENERATION_SPEC.md`:
- OpenAPI authority documents live under `sdks/sdkwork-agents-*/openapi/`
- Generated TypeScript output lives under `sdks/sdkwork-agents-*-typescript/generated/server-openapi/`
- SDK families declare `.sdkwork-assembly.json` with `sdkOwner` and `apiAuthority`

---

## 6. Response Envelope

All L2+ API success JSON bodies follow the `SdkWorkApiResponse` envelope from
`API_SPEC.md` §15:

```json
{
  "code": 0,
  "data": { ... },
  "traceId": "<server-uuid>"
}
```

- `code` is numeric `int32`; success is always `0`.
- `traceId` is the W3C trace identifier propagated from the `traceparent` header
  or minted at the gateway boundary.
- Single-resource payloads use `data.item: <ResourceDto>` (`SdkWorkResourceData<T>`).
- List payloads use `data.items: [<RecordDto>…]` plus `data.pageInfo`
  (`SdkWorkPageData<T>`), where `pageInfo.mode` is `offset` or `cursor`.
- Command acknowledgements use `data.accepted` plus optional `resourceId` /
  `status` (`SdkWorkCommandData`); HTTP `202` async accepts return
  `data.operationId`, `data.status`, and optional `data.pollUrl`.

Wire types are owned by the service crate (`sdkwork-intelligence-agents-service`):
`response.rs` exposes `ApiProblem`, `ResourceData<T>`, `PageData<T>`, and the
`finish_api_json` / `created_json` helpers that serialize through
`sdkwork-web-framework`'s `WebRequestContext`.

---

## 7. Error Mapping

All API errors use HTTP 4xx/5xx with `application/problem+json` (`ProblemDetail`,
RFC 9457). The body carries required numeric `code` (int32, non-zero) and
`traceId`:

```json
{
  "type": "about:blank",
  "title": "Not Found",
  "status": 404,
  "code": 40401,
  "traceId": "<server-uuid>",
  "detail": "agent not found: agent.demo"
}
```

Platform numeric error codes follow `API_SPEC.md` §15.3:

| Numeric code | HTTP status | When |
| --- | --- | --- |
| `40001` | 400 | Validation failure, malformed input, ID mismatch |
| `40101` | 401 | Missing or invalid credentials |
| `40301` | 403 | IAM policy denied the operation |
| `40401` | 404 | Agent/session/message/slot/binding not found or deleted |
| `40901` | 409 | Duplicate code, duplicate binding, version conflict |
| `42201` | 422 | Semantically invalid request payload |
| `50001` | 500 | Persistence failure, kernel error, runtime error |
| `50101` | 501 | Operation not supported on this surface |
| `50301` | 503 | Upstream dependency unavailable |

Business failures MUST NOT use HTTP 2xx with non-zero `code`, string wire codes,
`success`, or human `message`. The forbidden legacy envelopes (`PlusApiResult`,
`AppbaseApiResult`, `StoreApiResult`, `SdkWorkResponse`, per-domain `*ApiResult`,
top-level `requestId`) are absent from all surfaces.
