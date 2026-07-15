# Agents Database Module

Application-level metadata for the SDKWork Agents product. Agent runtime
persistence for sessions, tasks, and run state remains owned by
`sdkwork-kernel` through `sdkwork-agent-database`.

Managed per `DATABASE_FRAMEWORK_SPEC.md` and
`database/database.manifest.json`.

## Initialization State

This module is in initialization state for greenfield deployments:

1. **Baseline** - `database/ddl/baseline/{engine}/0001_agents_baseline.sql` contains the complete `3.1.0` schema snapshot.
2. **Migrations** - `database/migrations/{engine}/` accepts only changes introduced after the current baseline is released. A structure already present in the baseline must not be repeated in a migration.
3. **Drift** - run `pnpm db:drift:check` before release.

## Commands

```bash
pnpm run db:validate
pnpm run db:materialize:contract
pnpm run db:plan
pnpm run db:init
pnpm run db:migrate
pnpm run db:seed
pnpm run db:status
pnpm run db:drift:check
```
