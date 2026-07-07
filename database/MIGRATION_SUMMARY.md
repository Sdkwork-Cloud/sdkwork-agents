# Database Contract Migration Summary

Status: active
Updated: 2026-07-06

## Current state

- **Contract version:** 3.0.0 (`database/contract/schema.yaml`)
- **Engine:** PostgreSQL only
- **DDL authority:** `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- **Runtime bootstrap:** `SyncPostgresAdapter::apply_managed_store_schema()` includes the same baseline file

## v3 consolidation (2026-07)

1. Merged runtime embedded DDL into lifecycle baseline (single source).
2. Removed SQLite from contract and retired `database/ddl/baseline/sqlite/0001_agents_baseline.sql`.
3. Removed unimplemented `ai_agent_task_run` table until kernel run projection is implemented.
4. Aligned `database.manifest.json#contractVersion` with schema contract.

## Operations

```powershell
pnpm db:migrate
pnpm db:materialize:contract
```
