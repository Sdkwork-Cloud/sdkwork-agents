# Agents Database Module

Owner: `agents-platform`

Canonical contract: `database/contract/schema.yaml` (`7.2.0`)

Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`

## Scope

This module is the single system of record for managed agents and durable agent
execution:

```text
AgentWorkspace -> AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
                                      ^
AgentTask -> AgentTaskRun -> AgentTaskRunAttempt
```

The Session aggregate also owns one current runtime binding, resumable
checkpoint references, typed Drive relations, durable Turn input queues,
Task/Run/Attempt scheduling state, retry/lease/fencing state, audit facts, and
transactional outbox facts. External outbox delivery remains a release gate
until the platform provides an approved generic publisher SPI; this module does
not implement a local Kafka producer, raw HTTP relay, or downstream table
writer. IM conversations, IM delivery state,
runtime-location details, provider catalogs, capability content, and Drive bytes
remain owned by their respective modules.

## Engine And Lifecycle

PostgreSQL is the only managed-store engine. The `7.2.0` greenfield baseline
contains the complete 23-table Session execution and Task scheduling model. It
is the only database state supported before the first release.
`baseline-plus-migrations` remains the lifecycle strategy. New installations
use the complete `7.2.0` baseline. Existing shared development schemas use the
ordered forward-only migrations in `database/migrations/postgres/` to reach the
same contract without replaying the baseline or deleting dependency-owned data.

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

The live PostgreSQL suite requires `SDKWORK_DATABASE_URL` and these
administrative provisioning values:

- `SDKWORK_DATABASE_ADMIN_HOST`
- `SDKWORK_DATABASE_ADMIN_DATABASE`
- `SDKWORK_DATABASE_ADMIN_USERNAME`
- `SDKWORK_DATABASE_ADMIN_PASSWORD`

`SDKWORK_DATABASE_ADMIN_PORT` and `SDKWORK_DATABASE_ADMIN_SSL_MODE` are
optional. The suite creates and removes an isolated `sdkwork_ai_test_*`
database and schema, and verifies Task occurrence materialization, concurrent
Run claiming, Attempt creation, expired-lease recovery, and stale-fence
rejection. Credentials belong in operator or CI secret storage and must not be
committed.

Related authorities:

- `../sdkwork-specs/DATABASE_SPEC.md`
- `../sdkwork-specs/DATABASE_FRAMEWORK_SPEC.md`
- `../sdkwork-specs/MIGRATION_SPEC.md`
- `specs/AGENTS_DOMAIN_SPEC.md`
- `specs/AGENTS_SESSION_MODEL_SPEC.md`
- `specs/AGENTS_TASK_SCHEDULING_SPEC.md`
