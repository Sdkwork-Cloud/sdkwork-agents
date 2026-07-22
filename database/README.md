# Agents Database Module

Owner: `agents-platform`

Canonical contract: `database/contract/schema.yaml` (`5.0.0`)

Physical authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`

## Scope

This module is the single system of record for managed agents and durable agent
execution:

```text
AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
```

The Session aggregate also owns one current runtime binding, resumable
checkpoint references, typed Drive relations, retry/lease/fencing state, audit
facts, and reliable outbox events. IM conversations, IM delivery state,
runtime-location details, provider catalogs, capability content, and Drive bytes
remain owned by their respective modules.

## Engine And Lifecycle

PostgreSQL is the only managed-store engine. The pre-launch contract is one
greenfield baseline; `database/migrations/postgres/` is empty and reserved for
future released-schema changes. `baseline-plus-migrations` remains the lifecycle
strategy so the first post-launch change can be added without changing bootstrap
semantics.

The SQLite DDL is an explicitly non-authoritative four-table control-plane
development subset. It does not implement the Session aggregate and is not
listed in `database.manifest.json#engines`.

All business `id` columns are application-allocated signed 64-bit values.
Neither PostgreSQL nor SQLite uses sequences, identity columns, rowid aliases,
or any other database-side business ID allocation.

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
