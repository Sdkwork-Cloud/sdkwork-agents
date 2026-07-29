# Agents Database Module

Owner: `agents-platform`

Canonical contract: `database/contract/schema.yaml` (`6.0.2`)

Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`

## Scope

This module is the single system of record for managed agents and durable agent
execution:

```text
AgentWorkspace -> AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
```

The Session aggregate also owns one current runtime binding, resumable
checkpoint references, typed Drive relations, retry/lease/fencing state, audit
facts, and reliable outbox events. IM conversations, IM delivery state,
runtime-location details, provider catalogs, capability content, and Drive bytes
remain owned by their respective modules.

## Engine And Lifecycle

PostgreSQL is the only managed-store engine. The `6.0.2` greenfield baseline
contains the complete Workspace-scoped Project model and canonical provider
Session lineage names. It is the only database state supported before the first
release. `baseline-plus-migrations` remains the lifecycle strategy so ordered
forward migrations can be added after the schema is released. The migration
directory remains empty while the application is pre-launch; local development
installations are rebuilt from the current baseline.

Lifecycle `init` atomically materializes the consolidated baseline only when the
completion anchor is absent. Automatic pending-migration execution defaults to
disabled (`lifecycle.autoMigrate=false`); release and operator workflows run
`pnpm db:migrate` explicitly before service readiness. An existing partial
schema that already contains the completion anchor is never treated as an empty
database: startup drift validation fails closed instead of replaying the
greenfield baseline.

All business `id` columns are application-allocated signed 64-bit values.
PostgreSQL sequences and identity columns are not used for business IDs.

## Commands

```powershell
pnpm db:validate
pnpm db:materialize:contract
pnpm db:plan
pnpm db:init
pnpm db:migrate
pnpm db:seed
pnpm db:status
pnpm db:drift:check
pnpm test:database:postgres-live
```

The live PostgreSQL test requires `SDKWORK_AGENTS_TEST_POSTGRES_URL` with
permission to create and drop an isolated test schema. Credentials belong in
operator or CI secret storage and must not be committed.

Related authorities:

- `../sdkwork-specs/DATABASE_SPEC.md`
- `../sdkwork-specs/DATABASE_FRAMEWORK_SPEC.md`
- `../sdkwork-specs/MIGRATION_SPEC.md`
- `specs/AGENTS_DOMAIN_SPEC.md`
- `specs/AGENTS_SESSION_MODEL_SPEC.md`
