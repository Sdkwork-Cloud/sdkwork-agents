# Agents Database Module

Application-level metadata for the SDKWork Agents product. Agent runtime
persistence for sessions, tasks, and run state remains owned by
`sdkwork-kernel` through `sdkwork-agent-database`.

Managed per `DATABASE_FRAMEWORK_SPEC.md` and
`database/database.manifest.json`.

## Initialization State

This module is in initialization state for greenfield deployments:

1. **Baseline** - `database/ddl/baseline/{engine}/0001_agents_baseline.sql` contains the full DDL snapshot.
2. **Migrations** - `database/migrations/{engine}/` is reserved for approved incremental schema changes after the baseline contract changes. It is intentionally empty while contract version `3.0.0` is served entirely from the baseline.
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
