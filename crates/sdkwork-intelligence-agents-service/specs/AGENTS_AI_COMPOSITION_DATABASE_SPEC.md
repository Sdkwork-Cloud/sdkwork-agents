# SDKWork Agents AI Composition Database Specification

- Version: `1.0.0`
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Compliance: L2 (core), L3 (audit)
- Related: `database/contract/schema.yaml`, `DATABASE_SPEC.md`, `DATABASE_FRAMEWORK_SPEC.md`

## 1. Scope

`sdkwork-agents` owns the **agent composition plane** only: agent identity, runtime bindings,
deployments, composition slot references, audit facts, outbox events, and application registry.

Content and domain persistence are owned by sibling modules:

| Concern | Owner module | Prefix |
| --- | --- | --- |
| Knowledge / RAG | `sdkwork-knowledgebase` | `kb_` |
| Memory | `sdkwork-memory` | `mem_` |
| Skills | `sdkwork-skills` | `ai_agent_skill_*` |
| Prompts | `sdkwork-prompts` | `ai_prompt_*` |
| Files | `sdkwork-drive` | `dr_` |
| MCP (transitional) | `sdkwork-agents` legacy | `a_agent_mcp_server` |

Agent runtime session/task state remains in `sdkwork-kernel` (`sdkwork-agent-database`).

## 2. Design Principles

1. Every agents-owned table uses the `ai_` module prefix per `DATABASE_SPEC.md`.
2. Internal identifiers are `int64`; API serialization uses string.
3. No plaintext secrets in business tables; references only (`profile.*`, `endpoint.*`).
4. Soft-delete plus audit trace for management mutations.
5. `tenant_id` and `organization_id` are explicit columns on all tenant entities.
6. Snowflake IDs are allocated in application code before insert (no `BIGSERIAL` / `RETURNING id`).
7. Cross-module resources are referenced through `ai_agent_composition_slot`, not duplicated tables.

## 3. Table Overview

| Table | Profile | Compliance | Responsibility |
| --- | --- | --- | --- |
| `ai_app_registry` | `tenant_entity` | L2 | Application deployment registry (tenant, app key, kernel slot). |
| `ai_agent` | `tenant_entity` | L2 | Agent identity, manifest snapshot, lifecycle, visibility. |
| `ai_agent_runtime_binding` | `tenant_entity` | L2 | Provider/runtime binding for an agent. |
| `ai_agent_deployment` | `tenant_entity` | L2 | Deployment history with binding snapshots. |
| `ai_agent_composition_slot` | `tenant_entity` | L2 | Agent → sibling-module resource references. |
| `ai_agent_audit_event` | `audit_log` | L3 | Immutable management audit facts. |
| `ai_agent_outbox_event` | `ledger_event` | L2 | Cross-module async propagation. |
| `a_agent_mcp_server` | `tenant_entity` | L2 | **Legacy** MCP marketplace until `sdkwork-mcp` lands. |

## 4. `ai_agent_composition_slot`

Binds an agent to an external module resource without copying domain data.

| Column | Type | Description |
| --- | --- | --- |
| `slot_id` | `VARCHAR(128)` | Stable id `slot.{kind}.{name}` |
| `slot_kind` | `VARCHAR(64)` | `memory`, `knowledge`, `skill`, `prompt`, `drive`, `tool` |
| `target_module` | `VARCHAR(64)` | `memory`, `knowledgebase`, `skills`, `prompts`, `drive` |
| `target_ref` | `VARCHAR(256)` | External stable reference (e.g. mem space id, kb space id) |
| `target_version_ref` | `VARCHAR(128)` | Optional pinned version |
| `policy_json` | `JSONB` | Slot-level overrides (non-secret) |
| `priority`, `enabled` | | Orchestration order |

Indexes:

- `uk_ai_agent_composition_slot_tenant_agent_slot`
- `idx_ai_agent_composition_slot_lookup`

Composition slot management audit actions recorded in `ai_agent_audit_event`:

- `composition_slot_created`
- `composition_slot_updated`
- `composition_slot_deleted`

## 6. Migration from legacy `a_agent_*`

| Legacy | New |
| --- | --- |
| `a_agent_business` | `ai_agent` |
| `a_agent_provider_binding` | `ai_agent_runtime_binding` |
| `a_agent_deployment` | `ai_agent_deployment` |
| `a_agent_business_audit_event` | `ai_agent_audit_event` |
| `agents_app_registry` | `ai_app_registry` |
| `a_agent_knowledge_*` | **removed** → `sdkwork-knowledgebase` |
| `a_agent_memory_*` | **removed** → `sdkwork-memory` |

Migration script: `database/migrations/postgres/0001_ai_agent_core.up.sql`

## 7. Authority

Canonical DDL: `specs/sql/agents_managed_store_postgres.sql`  
Contract registry: `database/contract/table-registry.json`
