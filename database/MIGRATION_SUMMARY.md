# Database Contract State

Status: active pre-launch baseline

Updated: 2026-07-31

## Current Contract

- Contract version: `7.0.0`
- Managed engine: PostgreSQL
- Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- Lifecycle strategy: `baseline-plus-migrations`
- Development migrations: none
- Active tables: 23

The full current schema is installed from one baseline on an empty database.
The application has not been released, so there is no supported historical
schema or Task schedule to upgrade. Development installations created before
`7.0.0` are rebuilt from the consolidated baseline. There is no dual-write
path, derived read store, legacy Session table, or compatibility migration.

## Operational Checks

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:drift:check
```
