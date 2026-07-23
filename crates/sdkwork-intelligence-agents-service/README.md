# SDKWork Agent Business

`sdkwork-intelligence-agents-service` is the backend business module for agent management.
It depends on:

- `sdkwork-agent-kernel` for core agent runtime contracts, policy requests,
  events, and error models.
- `sdkwork-code-kernel` for code-task intent contracts used by managed agents.

This module is backend-focused and does not include frontend UI or app shell
integration. It defines:

- agents managed store domain model and lifecycle
- CRUD service orchestration with policy checks
- audit event contracts
- repository/audit ports and deterministic in-memory adapters for tests
- postgres-oriented row mapping and SQL contract constants for persistence
  adapters
- optional `postgres-sync` feature with executable PostgreSQL adapter for
  repository/audit writes
- optional `http-axum` feature with app-api/backend-api router composition and
  RFC 9457 problem detail responses
- app-api/backend-api route and operation contract declarations
- app-api/backend-api DTO mapping contracts
- database contract and DDL baseline for deployment-specific adapters

## Architecture

**Key Design Principles (2025 Update):**
- **Stateless Service Layer**: `AgentsService` uses `&self` methods with no interior mutability, enabling true concurrent request processing without global locks.
- **Thread-Safe Ports**: `AgentRepository` and `AgentAuditSink` traits use `&self` for all operations; adapters must provide interior mutability (e.g., `RwLock` for in-memory, connection pool for PostgreSQL).
- **SQL-Level Filtering**: Search queries are pushed to PostgreSQL WHERE clause with trigram GIN indexes, eliminating in-memory filter chains.
- **Connection Pool Monitoring**: `BlockingPostgresPool` exposes `PoolMetrics` (total, idle, active, utilization) for observability.

```text
sdkwork-intelligence-agents-service/
|-- src/
|   |-- lib.rs
|   |-- api.rs               # app-api/backend-api operation contract declarations
|   |-- domain.rs            # entities, status machine, business enums
|   |-- dto.rs               # API DTO <-> command/entity mapping
|   |-- application.rs       # command models + stateless business service (&self)
|   |-- ports.rs             # repository/audit interfaces (all &self methods)
|   |-- infrastructure.rs    # in-memory adapters, IAM-gated/dev-only policy providers, metrics
|   |-- persistence.rs       # postgres row mapping, SQL with WHERE filtering, adapter wrappers
|   |-- postgres_sync_pool.rs # blocking pool facade with PoolMetrics monitoring
|   `-- http.rs              # optional axum app-api/backend-api route entrypoints
|-- tests/
|   |-- agent_business_service_contracts.rs
|   `-- http_axum_contracts.rs
|-- scripts/
|   |-- verify-sdkgen.ps1
|   `-- verify-ci.ps1
`-- specs/
    |-- README.md
    |-- component.spec.json
    |-- AGENTS_MANAGED_STORE_DATABASE_SPEC.md
    |-- sdkgen/commands.md
    |-- sql/agents_managed_store_postgres.sql  # includes trigram indexes for LIKE search
    `-- openapi/
        |-- agents-app-api.openapi.yaml
        `-- agents-backend-api.openapi.yaml
```

## API Surfaces

- App API prefix: `/app/v3/api`
- Backend API prefix: `/backend/v3/api`
- Canonical resources:
  - `/app/v3/api/ai/agents`
  - `/app/v3/api/ai/projects/{projectId}/composition_slots`
  - `/backend/v3/api/ai/agents`
- Restore endpoints:
  - `/app/v3/api/ai/agents/{agentId}/restore`
  - `/backend/v3/api/ai/agents/{agentId}/restore`
- Backend audit endpoint `/backend/v3/api/ai/agents/{agentId}/audit_events`
  returns recorded audit events with `page/page_size` pagination and optional
  `action/from/to` filters; `from`/`to` must be RFC3339 and `from <= to`.
- Agent list endpoint supports optional `q` fuzzy search over `agentId`, `code`,
  `displayName`, and `description`.
- All mutation requests validate `requestedAt` strictly as RFC3339
  date-time (`create/update/delete/status/restore`).
- Operation IDs follow dotted resource style, for example `agents.create`,
  `agents.status.create`, and `agents.auditEvents.list`.

## SDK Generation Contract

Use the repository root `sdks/` workspace as the canonical SDK generation
boundary. The current application domain is `agent`.

```powershell
node .\sdks\materialize-agent-v3-openapi-boundaries.mjs
```

```powershell
node .\sdks\workspace-agent-sdkgen.mjs --mode dry-run
node .\sdks\workspace-agent-sdkgen.mjs --mode apply
```

SDK families:

- `sdkwork-agents-sdk`: `sdkwork-agents-open-api`, `/agent/v3/api`, `@sdkwork/agents-sdk`
- `sdkwork-agents-app-sdk`: `sdkwork-agents-app-api`, `/app/v3/api`, `@sdkwork/agents-app-sdk`
- `sdkwork-agents-backend-sdk`: `sdkwork-agents-backend-api`, `/backend/v3/api`, `@sdkwork/agents-backend-sdk`

All SDK generator commands use `--standard-profile sdkwork-v3`.

Composition slots use the canonical domain mapping. Document references are
created with `slotKind=document` and `targetModule=documents`; the generated
SDK exposes the existing Project composition CRUD surface and does not copy or
re-export Documents API operations.

## Verification

```bash
cargo test --manifest-path sdkwork-intelligence-agents-service/Cargo.toml
```

```bash
cargo test --features http-axum --manifest-path sdkwork-intelligence-agents-service/Cargo.toml
```

```bash
cargo test --features postgres-sync --manifest-path sdkwork-intelligence-agents-service/Cargo.toml
```

```powershell
node .\scripts\check-agent-sdk-workspace.mjs
```

```powershell
node .\sdks\workspace-agent-sdkgen.mjs --mode dry-run
```

```powershell
powershell -ExecutionPolicy Bypass -File .\sdkwork-intelligence-agents-service\scripts\verify-ci.ps1
```

## SDKWork Documentation Contract

Domain: intelligence
Capability: agents
Package type: rust-crate
Status: standardizing

### Public API

Public exports are declared in `specs/component.spec.json` under `contracts.publicExports`.

### Required SDK Surface

- `sdkwork-agents-sdk: sdkwork-agents-open-api -> @sdkwork/agents-sdk`
- `sdkwork-agents-app-sdk: sdkwork-agents-app-api -> @sdkwork/agents-app-sdk`
- `sdkwork-agents-backend-sdk: sdkwork-agents-backend-api -> @sdkwork/agents-backend-sdk`

### Configuration

Configuration keys and runtime entrypoints are declared in `specs/component.spec.json`.

### SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`, including deployment and runtime configuration rules where applicable.

### Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

### Extension Points

Extension points are limited to declared public exports, runtime entrypoints, SDK clients, events, and config keys.

### Verification

- `cargo test --manifest-path sdkwork-intelligence-agents-service/Cargo.toml`
- `cargo test --features http-axum --manifest-path sdkwork-intelligence-agents-service/Cargo.toml`
- `cargo test --features postgres-sync --manifest-path sdkwork-intelligence-agents-service/Cargo.toml`
- `node sdks/materialize-agent-v3-openapi-boundaries.mjs`
- `node sdks/workspace-agent-sdkgen.mjs --mode dry-run`
- `node scripts/check-agent-sdk-workspace.mjs`
- `powershell -ExecutionPolicy Bypass -File sdkwork-intelligence-agents-service/scripts/verify-ci.ps1`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
