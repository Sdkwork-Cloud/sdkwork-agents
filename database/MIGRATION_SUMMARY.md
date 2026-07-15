# Database Contract State

Status: active
Updated: 2026-07-14

## Current State

- **Contract version:** 3.1.0 (`database/contract/schema.yaml`)
- **Implemented engine:** PostgreSQL
- **DDL authority:** `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- **Migration authority:** `database/migrations/postgres/`
- **Runtime bootstrap:** `sdkwork_agents_database_host::bootstrap_agents_database()` uses `LifecycleOrchestrator` (init + migrate)
- **Strategy:** `baseline-plus-migrations` — baseline applied once on empty database, then versioned migrations applied incrementally

SQLite now has a native eight-table baseline and a validated service pool facade. Managed-store
engine support remains gated on the complete repository/audit adapters, transaction semantics,
lifecycle integration, server-side pagination, and PostgreSQL parity tests. Runtime or kernel
SQLite databases remain outside this module and must not be reported as agents store parity.

The `3.1.0` baseline is the complete schema authority for new installations. It includes native
JSON columns, tenant-scoped foreign keys, uniqueness constraints, and the capability validation
function. Incremental migrations begin only after this baseline is released; no migration may
repeat a structure already owned by the baseline.

`ai_agent_task_run` remains outside the current product contract until the kernel `AgentRun` and
`AgentStep` projection authority is approved. It must enter through a reviewed API contract and a
new versioned migration rather than an undocumented table addition.

## Operations

```powershell
pnpm db:migrate
pnpm db:materialize:contract
```
