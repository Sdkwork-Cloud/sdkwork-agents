# Database Contract State

Status: active pre-launch baseline consolidation

Updated: 2026-08-04

## Current Contract

- Contract version: `7.2.0`
- Managed engine: PostgreSQL
- Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- Lifecycle strategy: `baseline-plus-migrations` (empty post-baseline migration set)
- Development migrations: none (pre-launch consolidation on the baseline)
- Active tables: 23

The full current schema is installed from one baseline on an empty schema and
tracked in `ops_database_installation_state`. The pre-launch forward
development migrations (`0001`..`0007`) were removed when the baseline was
folded to the complete `7.2.0` schema; no pending or applied migration rows
are expected for the `agents` module in shared development schemas. There is
no dual-write path, derived read store, legacy Session table, or runtime
compatibility branch.

After first production release, add ordered expand/contract migrations without
rewriting the released baseline.

## Operational Checks

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:drift:check
```
