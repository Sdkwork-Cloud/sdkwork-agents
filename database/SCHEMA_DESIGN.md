# Agents Database Schema

Status: active
Owner: agents-platform
Contract: `database/contract/schema.yaml` (v3.1.0)
DDL authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
Migration authority: `database/migrations/postgres/`

## Engine

PostgreSQL is the only supported managed-store engine. Schema lifecycle is
managed by `sdkwork-database-lifecycle` (`LifecycleOrchestrator`) through the
`baseline-plus-migrations` strategy: the baseline DDL is applied once on an
empty database, then versioned migrations in `database/migrations/postgres/`
are applied incrementally. Both runtime bootstrap
(`bootstrap_agents_database`) and `pnpm db:migrate` use the same lifecycle
orchestrator, ensuring migrations are tracked in `ops_schema_migration_history`
with checksum verification.

## Tables

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Agent identity, manifest snapshot, lifecycle |
| `ai_agent_runtime_binding` | Provider/runtime binding |
| `ai_agent_composition_slot` | Cross-module composition references |
| `ai_agent_audit_event` | Immutable management audit log |
| `ai_agent_session` | Hosted chat sessions |
| `ai_agent_message` | Session messages and chat turns |
| `ai_agent_interaction` | Live interaction approval and user-question records |
| `ai_agent_task` | Scheduled tasks projected from kernel `AgentTask` |

Current database contract scope excludes `ai_agent_task_run`. Entry criteria
require a stable kernel `AgentRun` / `AgentStep` projection, approved
`agents.taskRuns.*` API authority, and a versioned migration from the active
contract.

## Cross-Module References

Sibling-owned content for memory, knowledge, skills, prompts, drive, and MCP is
referenced only through `ai_agent_composition_slot`.
