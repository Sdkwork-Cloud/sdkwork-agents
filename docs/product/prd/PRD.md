# SDKWork Agents Product Requirements

- Version: `5.1.0`
- Status: draft
- Release Stage: pre-launch release candidate
- Owner: `agents-platform`
- Updated: `2026-07-23`
- Specs: [REQUIREMENTS_SPEC.md](../../../../sdkwork-specs/REQUIREMENTS_SPEC.md), [DOCUMENTATION_SPEC.md](../../../../sdkwork-specs/DOCUMENTATION_SPEC.md)

## 1. Background And Problem

SDKWork products need a common managed-agent service that can execute multiple
provider engines, retain durable execution history, enforce tenant policy and
support user-facing, operator and integration surfaces. Provider mechanisms
belong to `sdkwork-kernel`; managed business state belongs to `sdkwork-agents`.

The product uses one execution vocabulary:

```text
AgentWorkspace -> AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
```

It is distinct from instant messaging. IM may invoke Agents, but continues to
own communication delivery, membership, realtime fanout and visible IM history.

## 2. Target Users

| User | Primary need |
| --- | --- |
| Application user | Run an agent, resume a session, inspect ordered results and resolve approvals/questions |
| Product integrator | Embed Agents through a generated SDK without provider-specific coupling |
| Tenant operator | Govern agents, provider bindings, policies, sharing, audit and lifecycle |
| Platform operator | Deploy, monitor, reconcile and recover the service safely |
| Provider maintainer | Add a conformance-tested kernel provider without changing product contracts |

## 3. Goals And Non-Goals

Goals:

- one durable Session aggregate across assistant, coding, automation and IM
  dispatch use cases;
- idempotent Turn execution with typed ordered Session Items;
- high-cohesion ownership of agent business state and public contracts;
- generated Open, App and Backend SDKs with strict input/output contracts;
- independent provider, skill, prompt, document, memory, knowledge, MCP, LLM and Drive
  capability modules;
- tenant isolation, trusted context, auditability and operational recovery.

Non-goals:

- generic IM conversations, delivery, read state, presence or reactions;
- provider runtime SPI implementation inside the product service;
- skill package content, installation records or marketplace ownership;
- copied model catalogs, prompt bodies, document content, memory records or Drive bytes;
- product-local duplicate Workspace authorities, filesystem paths or UI state persistence.

## 4. Product Scope

| Capability | Product behavior | Authority |
| --- | --- | --- |
| Managed agents | Lifecycle, visibility, provider binding and policy | `ai_agent*` composition tables |
| Workspaces | User-owned Project container with one idempotent default per user | `AgentWorkspace` |
| Projects | Workspace-scoped reusable orchestration context, membership and sharing | `AgentProject` |
| Sessions | Durable execution context and lineage | `AgentSession` |
| Turns | Idempotent execution, retry, cancellation and usage | `AgentTurn` |
| Session items | Ordered input/output/tool/artifact facts | `AgentSessionItem` |
| Human interaction | Claim and resolve approval or user question | `AgentInteraction` |
| Runtime continuity | Runtime bindings and checkpoints | Session aggregate |
| Tasks | Scheduled or deferred agent commands | `AgentTask` |
| User state | Pin, hide and resource-specific preferences | Agents user-state table |
| Operations | Audit, outbox, metrics, health and reconciliation | Agents service |

## 5. User Scenarios

### 5.1 Interactive execution

The app creates or restores a Session, submits an idempotent Turn and renders
ordered Session Items. It may consume a JSON completion or typed SSE delta and
completion events.

### 5.2 Coding product integration

A coding product selects an Agent Workspace, uses its Workspace-scoped Agent
Projects, and calls the same Session and Turn APIs. Product-owned
runtime-location and repository state remain local; only bounded opaque
references are supplied to Agents.

### 5.3 IM dispatch

IM authorizes its participant and communication context, invokes Agents through
a public SDK, and stores opaque Agent Session/Turn correlation. Both services
retain independent write ownership and retry through idempotency.

### 5.4 Human approval

A runtime creates a typed Interaction. One authorized actor claims it and then
approves, rejects or answers it with optimistic concurrency and a one-time
claim token.

### 5.5 Resume and recovery

The user restores a validated checkpoint. The service verifies trusted scope,
runtime binding, lifecycle and expected version before provider invocation.

## 6. Functional Requirements

| ID | Requirement | Acceptance evidence |
| --- | --- | --- |
| FR-1 | Session is the sole durable agent execution context | No second product session/transcript persistence |
| FR-2 | Session creation is idempotent | Required kind, surface, key, hash and request time |
| FR-3 | Turn execution is idempotent and cancellable | Conflict detection, lifecycle tests and fencing state |
| FR-4 | Session Items are ordered and typed | Store-level pagination and immutable completed facts |
| FR-5 | Human pause points are race-safe | Claim token plus optimistic version tests |
| FR-6 | Runtime selection and resume are bounded | Runtime binding and checkpoint authorization tests |
| FR-7 | Product integration uses generated SDKs | No raw HTTP, manual auth header or local DTO fork |
| FR-8 | IM semantics remain independent | Mandatory `sdkwork-im -> sdkwork-agents` boundary check |
| FR-9 | Skills remain independently owned | Agents stores stable skill/version references only |
| FR-10 | Files use Drive references | No raw bytes, credentials or signed URLs in Agents rows |
| FR-11 | Documents use canonical composition references | Only `document/documents` is accepted; content remains in `sdkwork-documents` |

## 7. API And SDK Product Surface

| Surface | Prefix | Operations | Credential mode | SDK |
| --- | --- | ---: | --- | --- |
| App API | `/app/v3/api` | 79 | dual token | `@sdkwork/agents-app-sdk`, `sdkwork_agents_app_sdk` |
| Backend API | `/backend/v3/api` | 48 | dual token/operator context | `@sdkwork/agents-backend-sdk` |
| Open API | `/agent/v3/api` | 47 | `X-API-Key` | `@sdkwork/agents-sdk` |

The complete generated inventory is
[TECH-api-specification.md](../../architecture/tech/TECH-api-specification.md).

## 8. Data Ownership

Agents owns 20 PostgreSQL tables under prefix `ai_`. The canonical design is
[AGENTS_AI_COMPOSITION_DATABASE_SPEC.md](../../../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md).

All cross-domain links are identifiers validated through public contracts.
There are no cross-module foreign keys, shared write owners, alternate session
stores, or copied dependency entities.

## 9. Quality Requirements

- Security: trusted `AgentRequestContext`, least privilege, fail-closed provider
  selection, bounded input, sanitized errors and hashed share secrets.
- Correctness: transactionally consistent Turn and Session Item writes,
  idempotency conflict detection, optimistic concurrency and fencing.
- Performance: store-level pagination, bounded page size, indexed tenant scope,
  connection pooling and no in-process full-list slicing.
- Reliability: outbox publication, retry-safe commands, timeout reconciliation,
  checkpoints, health and metrics.
- Extensibility: provider manifests and composition slots extend the product
  without changing the Session aggregate.
- Maintainability: generated SDKs, machine-readable component specs and
  deterministic API documentation.

## 10. Success Metrics

| Metric | Target |
| --- | ---: |
| Cross-tenant access contract failures | 0 |
| Duplicate provider execution for one idempotency key | 0 |
| API/SDK/documentation inventory drift | 0 |
| Raw SDKWork HTTP calls in product consumers | 0 |
| Agents dependencies on IM | 0 |
| Database contract drift at release | 0 |
| Unhandled terminal Turn or Interaction states | 0 |

## 11. Release Gates

Release requires API, SDK, database, Rust, security, documentation, deployment
and supply-chain checks to pass, including `pnpm check:production-security`.
PostgreSQL is the production persistence authority. Open API credentials are
isolated from app/backend session tokens. The exact commands are maintained in
[pre-launch-verification.md](../../runbooks/pre-launch-verification.md).

## 12. Linked Requirements And Decisions

- [REQ-2026-0722-agent-session-execution.md](../requirements/REQ-2026-0722-agent-session-execution.md)
- [ADR-20260722-agent-session-domain-unification.md](../../architecture/decisions/ADR-20260722-agent-session-domain-unification.md)
- [AGENTS_SESSION_MODEL_SPEC.md](../../../specs/AGENTS_SESSION_MODEL_SPEC.md)
- [AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md](../../../specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md)

## 13. Open Questions

None for the canonical domain, persistence or public SDK boundary. New provider
families and optional independent capabilities follow their existing extension
contracts and conformance gates.
