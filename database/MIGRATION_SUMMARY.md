# Database Contract State

Status: active pre-launch baseline

Updated: 2026-07-22

## Current Contract

- Contract version: `6.0.0`
- Managed engine: PostgreSQL
- Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
- Lifecycle strategy: `baseline-plus-migrations`
- Active migrations: none (pre-launch baseline)
- Active tables: 20

The full current schema is installed from one baseline on an empty database.
The application has not been released, so there is no supported historical
installation to upgrade and no compatibility migration is retained. There is
no dual-write path, derived read store, or legacy session table. The first
post-release schema change will add an ordered forward migration without
rewriting this baseline.

## Operational Checks

```powershell
pnpm db:validate
pnpm db:plan
pnpm db:drift:check
```
