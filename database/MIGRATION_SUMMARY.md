# Database Contract State

Status: active
Updated: 2026-07-14

## Current State

- **Contract version:** 4.0.0 active (`database/contract/schema.yaml`)
- **Implemented engine:** PostgreSQL
- **DDL authority:** `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- **Migration authority:** `database/migrations/postgres/`
- **Runtime bootstrap:** `sdkwork_agents_database_host::bootstrap_agents_database()` uses `LifecycleOrchestrator` (init + migrate)
- **Strategy:** `baseline-plus-migrations` — baseline applied once on empty database, then versioned migrations applied incrementally

SQLite has an eight-table compatibility baseline and a validated service pool
facade, but it is not an active managed-store engine. The commercial contract is
PostgreSQL-only. Runtime or kernel SQLite databases remain outside this module
and must not be reported as Agents store parity.

The `3.1.0` baseline remains the immutable baseline authority. It includes native
JSON columns, tenant-scoped foreign keys, uniqueness constraints, and the capability validation
function. `0002_chat_project_commercial_expand`,
`0003_scope_agents_outbox_dedupe`, `0004_audit_action_runtime_compatibility`,
and `0005_generalize_agents_audit_aggregate` provide the active commercial
contract without rewriting the baseline or its checksum history.

`ai_agent_task_run` remains outside the current product contract until the kernel `AgentRun` and
`AgentStep` projection authority is approved. It must enter through a reviewed API contract and a
new versioned migration rather than an undocumented table addition.

## Operations

```powershell
pnpm db:migrate
pnpm db:materialize:contract
```
