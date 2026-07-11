# Database Contract Migration Summary

Status: active
Updated: 2026-07-12

## Current State

- **Contract version:** 3.1.0 (`database/contract/schema.yaml`)
- **Engine:** PostgreSQL only
- **DDL authority:** `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- **Migration authority:** `database/migrations/postgres/`
- **Runtime bootstrap:** `sdkwork_agents_database_host::bootstrap_agents_database()` uses `LifecycleOrchestrator` (init + migrate)
- **Strategy:** `baseline-plus-migrations` — baseline applied once on empty database, then versioned migrations applied incrementally

## v3.1 Migration (2026-07)

1. Migrated 8 TEXT JSON columns to JSONB for native JSON indexing and validation.
2. Added 7 foreign key constraints (all tenant-scoped, ON DELETE CASCADE).
3. Added `UNIQUE(tenant_id, id)` on `ai_agent` for audit event FK reference.
4. Updated `capabilities_json_is_standard` CHECK function to accept JSONB parameter.
5. Replaced `SyncPostgresAdapter::apply_managed_store_schema()` (direct baseline SQL execution) with `LifecycleOrchestrator` for proper migration tracking, checksum verification, and incremental migration support.
6. Migration file: `0002_jsonb_columns_and_fk_constraints`

## v3 Consolidation (2026-07)

1. Merged runtime embedded DDL into lifecycle baseline as the single source.
2. Removed SQLite from contract and retired `database/ddl/baseline/sqlite/0001_agents_baseline.sql`.
3. Kept `ai_agent_task_run` outside contract v3.0.0; entry requires stable kernel `AgentRun` / `AgentStep` projection, approved task-run API authority, and a versioned migration.
4. Aligned `database.manifest.json#contractVersion` with schema contract.

## Operations

```powershell
pnpm db:migrate
pnpm db:materialize:contract
```
