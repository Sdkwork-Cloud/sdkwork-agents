# Database Contract State

Status: active pre-launch baseline

Updated: 2026-07-22

## Current Contract

- Contract version: `5.0.0`
- Managed engine: PostgreSQL
- Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- Lifecycle strategy: `baseline-plus-migrations`
- Active migrations: none
- Active tables: 19

The full current schema is installed from one baseline on an empty database.
The migrations directory is intentionally reserved for changes made after the
first production schema release. There is no pre-launch compatibility chain,
dual-write path, projection store, or legacy session table.

## Operational Checks

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:drift:check
```
