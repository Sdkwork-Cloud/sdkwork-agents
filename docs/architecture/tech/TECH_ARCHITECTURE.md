# SDKWork Agents Technical Architecture

- Version: `5.1.0`
- Status: active
- Owner: `agents-platform`
- Updated: `2026-07-23`
- Specs: [ARCHITECTURE_DECISION_SPEC.md](../../../../sdkwork-specs/ARCHITECTURE_DECISION_SPEC.md), [DOCUMENTATION_SPEC.md](../../../../sdkwork-specs/DOCUMENTATION_SPEC.md)
- API Reference: [TECH-api-reference.md](TECH-api-reference.md)

## 1. Architecture Overview

```text
PC / H5 / Flutter / product integrations / sdkwork-im
                         |
             generated Agents SDK families
                         |
       Open API | App API | Backend API route crates
                         |
          sdkwork-intelligence-agents-service
             |                         |
     PostgreSQL managed store   sdkwork-agents-kernel-bridge
                                       |
                            sdkwork-agents-runtime-facade
                                       |
                                sdkwork-kernel SPI
                                       |
                              provider plugins
```

The durable business aggregate is:

```text
AgentProject
  `- AgentSession
       |- AgentSessionRuntimeBinding
       |- AgentTurn
       |    `- AgentSessionItem
       |- AgentInteraction
       `- AgentSessionCheckpoint
```

No other module owns a durable agent execution Session or transcript.

## 2. Technology Choices

| Layer | Technology | Responsibility |
| --- | --- | --- |
| HTTP | Rust, Axum, `sdkwork-web-framework` | request context, routing, envelopes, problem details |
| Application | Rust service/use-case layer | authorization, orchestration, transactions |
| Persistence | PostgreSQL, `sdkwork-database` | 19-table managed module and lifecycle |
| Runtime | `sdkwork-agents-runtime-facade` | product-safe kernel adapter |
| Provider mechanism | `sdkwork-kernel` | provider SPI, plugins and transient events |
| SDK | canonical `sdkgen` | TypeScript and Flutter generated clients |
| Clients | React PC/H5 and Flutter | services over injected App SDK clients |

## 3. System Boundaries And Modules

| Module | Owns | Does not own |
| --- | --- | --- |
| `sdkwork-intelligence-agents-service` | domain, use cases, repositories, HTTP adapters | provider implementation, UI state |
| `sdkwork-agents-runtime-facade` | runtime host, Turn execution, catalog, interactions | product persistence |
| `sdkwork-agents-kernel-bridge` | typed service-to-runtime adaptation | public product resources |
| `sdkwork-api-agents-assembly` | host-neutral route assembly | business rules |
| `sdkwork-routes-agents-*-api` | surface metadata and route mounting | persistence or provider logic |
| `sdks/sdkwork-agents-*-sdk` | generated consumer contracts | dependency-owned APIs |
| client core packages | client construction and injected ports | raw HTTP or auth header assembly |

The dependency direction is:

```text
product and IM -> sdkwork-agents -> sdkwork-kernel
                              `-> independent capability SDKs
```

## 4. Directory And Package Layout

```text
apis/                   authored API metadata
apps/                   PC, H5, Flutter and other client roots
crates/                 service, facade, bridge, routes and assembly
database/               manifest, contract, DDL, migrations and seeds
sdks/                   Open, App and Backend SDK families
specs/                  repository/application domain contracts
docs/                   product, architecture, guides and runbooks
scripts/                deterministic materialization and verification
```

Each authored module owns `specs/component.spec.json`. Generated SDK output
stays under `generated/server-openapi` and is never hand-edited.

## 5. API, SDK And Data Ownership

### 5.1 API authorities

| Surface | Authority | Prefix | Operations | Auth |
| --- | --- | --- | ---: | --- |
| App | `sdkwork-agents-app-api` | `/app/v3/api` | 68 | dual token |
| Backend | `sdkwork-agents-backend-api` | `/backend/v3/api` | 48 | dual token/operator |
| Open | `sdkwork-agents-open-api` | `/agent/v3/api` | 47 | API key |

Every operation carries `WebRequestContext`, surface metadata, permission,
tenant scope and audit metadata. Full inventory:
[TECH-api-specification.md](./TECH-api-specification.md).

### 5.2 SDK families

| Family | Consumer package | Language |
| --- | --- | --- |
| `sdkwork-agents-app-sdk` | `@sdkwork/agents-app-sdk` | TypeScript |
| `sdkwork-agents-app-sdk` | `sdkwork_agents_app_sdk` | Flutter/Dart |
| `sdkwork-agents-backend-sdk` | `@sdkwork/agents-backend-sdk` | TypeScript |
| `sdkwork-agents-sdk` | `@sdkwork/agents-sdk` | TypeScript |

Consumers import package roots. The TypeScript App facade accepts a canonical
App surface URL ending in `/app/v3/api`. The Flutter mobile core adapter applies
the same contract before constructing the generated client.

### 5.3 PostgreSQL module

| Group | Tables |
| --- | --- |
| Agent composition | `ai_agent`, `ai_agent_runtime_binding`, `ai_agent_composition_slot`, `ai_agent_audit_event` |
| Project | `ai_agent_project`, `ai_agent_project_composition_slot`, `ai_agent_project_member`, `ai_agent_share_link` |
| Session execution | `ai_agent_session`, `ai_agent_session_runtime_binding`, `ai_agent_turn`, `ai_agent_session_item`, `ai_agent_item_drive_ref`, `ai_agent_item_feedback`, `ai_agent_interaction`, `ai_agent_session_checkpoint` |
| Orchestration and delivery | `ai_agent_task`, `ai_agent_resource_user_state`, `ai_agent_outbox_event` |

All rows are tenant scoped. Session idempotency, Turn idempotency, ordered item
sequence, current runtime binding, Interaction claim, checkpoint lifecycle and
outbox delivery have explicit constraints/indexes. The detailed authority is
[AGENTS_AI_COMPOSITION_DATABASE_SPEC.md](../../../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md).

### 5.4 Independent capabilities

Skills, prompts, documents, memory, knowledge, MCP, model/provider profiles and
Drive own their package entities, tables and APIs. Agents stores stable references only.
This keeps module replacement and extension open without changing the Session
aggregate.

Agent and Project composition use the canonical mapping in
`specs/AGENTS_DOMAIN_SPEC.md` section 3. `document/documents` is a reference to
the Documents module, not a copied document table, Drive alias or read projection.

## 6. Execution And Consistency

1. The HTTP adapter derives tenant, organization, user, credential mode and
   trace from trusted request context.
2. The application service authorizes the agent and Session scope.
3. Session or Turn idempotency key and payload hash are checked atomically.
4. The repository writes request state and obtains sequence/fencing values.
5. The runtime bridge invokes the selected provider through the facade.
6. The service commits Turn outcome, ordered Session Items, usage, audit and
   outbox facts in the owning transaction boundary.
7. JSON returns typed resource data; SSE emits typed delta events followed by
   one terminal completion response.

Timeout reconciliation reads persisted state using the fully scoped identity
and never assumes failure from a network timeout.

## 7. Security, Privacy And Observability

- App/Backend credentials use the global session token manager; Open API uses
  an isolated API-key provider.
- Tenant, organization and user selectors are not accepted from request bodies
  or query strings.
- Authorization is enforced server-side for every resource and command.
- Provider errors are sanitized; credentials, signed URLs and raw dependency
  payloads are not persisted.
- Input sizes, pagination and metadata are bounded.
- Audit and outbox facts are append-oriented; redaction and retention are
  explicit lifecycle commands.
- Health, metrics, traces and structured logs expose no secrets or item content.

## 8. Deployment And Runtime Topology

Standalone and cloud profiles expose the same OpenAPI behavior. Route assembly
is host neutral. The selected profile supplies typed source configuration,
PostgreSQL connection pooling, dependency service endpoints and credential
providers. Production startup fails closed for missing database, auth or
required dependency configuration.

Kernel/provider runtime state is not a replacement for the Agents PostgreSQL
authority. Optional capability services may be mounted or remote without
changing public resource semantics.

## 9. Architecture Decision Index

- [ADR-20260722-agent-session-domain-unification.md](../decisions/ADR-20260722-agent-session-domain-unification.md)
- [AGENTS_KERNEL_BOUNDARY_SPEC.md](../../../specs/AGENTS_KERNEL_BOUNDARY_SPEC.md)
- [AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md](../../../specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md)

## 10. Verification

```powershell
cargo check -p sdkwork-intelligence-agents-service --features http-axum
cargo test -p sdkwork-intelligence-agents-service --features http-axum --lib
cargo test -p sdkwork-intelligence-agents-service --features http-axum --test http_axum_contracts
cargo test -p sdkwork-agents-runtime-facade
cargo check -p sdkwork-agents-kernel-bridge
cargo check -p sdkwork-api-agents-assembly
node sdks/workspace-agent-sdkgen.mjs --mode dry-run
node scripts/check-agent-sdk-workspace.mjs
pnpm check:agents-im-boundary
pnpm db:validate
pnpm check:docs
```
