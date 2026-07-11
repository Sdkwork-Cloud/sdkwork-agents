# Database Contract Migration Summary

Status: active
Updated: 2026-07-06

## Current State

- **Contract version:** 3.0.0 (`database/contract/schema.yaml`)
- **Engine:** PostgreSQL only
- **DDL authority:** `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- **Runtime bootstrap:** `SyncPostgresAdapter::apply_managed_store_schema()` includes the same baseline file

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
