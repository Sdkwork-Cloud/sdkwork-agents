# Agents Database Schema

Status: active
Owner: agents-platform
Contract: `database/contract/schema.yaml` (v3.0.0)
DDL authority: `database/ddl/baseline/postgres/0001_agents_baseline.sql`

## Engine

PostgreSQL is the **only** supported managed-store engine. Runtime bootstrap (`apply_managed_store_schema`) and `pnpm db:migrate` both apply the same baseline DDL file.

## Tables (8)

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Agent identity, manifest snapshot, lifecycle |
| `ai_agent_runtime_binding` | Provider/runtime binding |
| `ai_agent_composition_slot` | Cross-module composition references |
| `ai_agent_audit_event` | Immutable management audit log |
| `ai_agent_session` | Hosted chat sessions |
| `ai_agent_message` | Session messages and chat turns |
| `ai_agent_interaction` | Live interaction (approval / user question) |
| `ai_agent_task` | Scheduled tasks projected from kernel `AgentTask` |

`ai_agent_task_run` is deferred until kernel `AgentRun` projection ships (PRD Phase 5).

## Cross-module references

Sibling-owned content (memory, knowledge, skills, prompts, drive, MCP) is referenced only through `ai_agent_composition_slot`.
