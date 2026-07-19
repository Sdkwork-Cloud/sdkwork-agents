# Agents Database Module

Application-level metadata for the SDKWork Agents product. Agent runtime
persistence for sessions, tasks, and run state remains owned by
`sdkwork-kernel` through `sdkwork-agent-database`.

Managed per `DATABASE_FRAMEWORK_SPEC.md` and
`database/database.manifest.json`.

## Initialization State

This module uses an immutable baseline plus versioned migrations:

1. **Baseline** - `database/ddl/baseline/{engine}/0001_agents_baseline.sql` contains the immutable `3.1.0` schema snapshot.
2. **Migrations** - PostgreSQL `0002` adds the commercial Chat/Project schema, `0003` scopes outbox deduplication, and `0004`/`0005` align audit persistence with runtime actions and project aggregates. The manifest exposes active contract `4.0.0`.
3. **Drift** - run `pnpm db:drift:check` before release.

Contract `4.0.0` is active after repository, API, generated SDK, frontend, IM
consumer, migration, isolated PostgreSQL, and release verification passed. IM
contract `2.0.0` owns dispatch and visible reply correlation; Agents retains no
IM table or identifier.

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
pnpm run test:database:postgres-live
```

The PostgreSQL live test requires `SDKWORK_AGENTS_TEST_POSTGRES_URL` with
permission to create and drop schemas. It creates a unique schema, applies the
canonical baseline and every migration through `sdkwork-agents-database-host`,
executes commercial Chat/Project persistence and transaction scenarios, and
removes the schema even when the test unwinds after a failure. Credentials stay
in the operator/CI secret environment and are never written to tracked config.
