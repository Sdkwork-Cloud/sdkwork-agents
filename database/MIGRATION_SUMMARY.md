# Database Contract State

Status: active pre-launch baseline with shared-development reconciliation

Updated: 2026-08-01

## Current Contract

- Contract version: `7.2.0`
- Managed engine: PostgreSQL
- Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- Lifecycle strategy: `baseline-plus-migrations`
- Development migrations: `0001_complete_agents_7_0_0_schema`, `0002_add_provider_session_directory`, `0003_add_typed_agent_interaction_envelope`
- Active tables: 23

The full current schema is installed from one baseline on an empty database.
Existing shared development schemas are reconciled through ordered forward
migrations. `0001` adds the canonical Turn input queue and Task Run/Attempt
tables, `0002` adds provider Session directory metadata, and `0003` adds the
bounded typed Interaction request envelope and expanded Interaction categories.
None deletes dependency-owned data. There is no dual-write path, derived read
store, legacy Session table, or runtime compatibility branch.

## Operational Checks

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:drift:check
```
