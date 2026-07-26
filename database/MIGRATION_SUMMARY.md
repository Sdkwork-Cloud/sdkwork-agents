# Database Contract State

Status: active pre-launch baseline

Updated: 2026-07-22

## Current Contract

- Contract version: `6.0.0`
- Managed engine: PostgreSQL
- Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- Lifecycle strategy: `baseline-plus-migrations`
- Active migrations: `0001_add_agent_workspaces.up.sql`
- Active tables: 20

The full current schema is installed from one baseline on an empty database.
Existing `5.0.0` installations create one default Workspace per historical
Project owner and backfill every Project before the Workspace foreign key is
enabled. There is no dual-write path, derived read store, or legacy session
table.

## Operational Checks

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:drift:check
```
