# SDKWork Agents AI Composition Database Specification

- Version: `3.0.0`
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Compliance: L2 (core), L3 (audit)
- Related: `database/contract/schema.yaml`, `DATABASE_SPEC.md`, `DATABASE_FRAMEWORK_SPEC.md`

## 1. Scope

`sdkwork-agents` owns the **agent composition plane** only: agent identity, runtime bindings,
composition slot references, and audit facts.

Content and domain persistence are owned by sibling modules:

| Concern | Owner module | Prefix |
| --- | --- | --- |
| Knowledge / RAG | `sdkwork-knowledgebase` | `kb_` |
| Memory | `sdkwork-memory` | `ai_` (memory-owned) |
| Skills | `sdkwork-skills` | `ai_skill_*` |
| Prompts | `sdkwork-prompts` | `ai_prompt_*` |
| Files | `sdkwork-drive` | `dr_` |
| MCP | `sdkwork-mcp` | `ai_mcp_*` |

Agent runtime session/task state remains in `sdkwork-kernel` (`sdkwork-agent-database`).

Agents reference all sibling-module resources exclusively through `ai_agent_composition_slot`.
No MCP, knowledge, memory, skills, prompts, or drive tables exist in this repository.

## 2. Design Principles

1. Every agents-owned table uses the `ai_` module prefix per `DATABASE_SPEC.md`.
2. Internal identifiers are `int64`; API serialization uses string.
3. No plaintext secrets in business tables; references only (`profile.*`, `endpoint.*`).
4. Soft-delete plus audit trace for management mutations.
5. `tenant_id` and `organization_id` are explicit columns on all tenant entities.
6. Snowflake IDs are allocated in application code before insert (no `BIGSERIAL` / `RETURNING id`).
7. Cross-module resources are referenced through `ai_agent_composition_slot`, not duplicated tables.
8. No over-design: agents owns 6 tables — identity, binding, composition, audit, session, message.

## 3. Table Overview

| Table | Profile | Compliance | Responsibility |
| --- | --- | --- | --- |
| `ai_agent` | `tenant_entity` | L2 | Agent identity, manifest snapshot, lifecycle, visibility. |
| `ai_agent_runtime_binding` | `tenant_entity` | L2 | Provider/runtime binding for an agent. |
| `ai_agent_composition_slot` | `tenant_entity` | L2 | Agent → sibling-module resource references. |
| `ai_agent_audit_event` | `audit_log` | L3 | Immutable management audit facts. |

## 4. `ai_agent_composition_slot`

Binds an agent to an external module resource without copying domain data.

| Column | Type | Description |
| --- | --- | --- |
| `slot_id` | `VARCHAR(128)` | Stable id `slot.{kind}.{name}` |
| `slot_kind` | `VARCHAR(64)` | `memory`, `knowledge`, `skill`, `prompt`, `drive`, `tool`, `mcp` |
| `target_module` | `VARCHAR(64)` | `memory`, `knowledgebase`, `skills`, `prompts`, `drive`, `mcp` |
| `target_ref` | `VARCHAR(256)` | External stable reference (e.g. mem space id, kb space id, mcp server id) |
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

## 5. Audit Actions

Only agent-core management actions are recorded. No MCP, memory, or knowledge domain actions.

| Action | Description |
| --- | --- |
| `created` | Agent created |
| `updated` | Agent updated |
| `deleted` | Agent soft-deleted |
| `restored` | Agent restored |
| `status_changed` | Agent status changed |
| `provider_binding_changed` | Provider binding created/updated/activated |
| `composition_slot_created` | Composition slot created |
| `composition_slot_updated` | Composition slot updated |
| `composition_slot_deleted` | Composition slot deleted |

## 6. Migration from legacy `a_agent_*`

| Legacy | New |
| --- | --- |
| `a_agent_business` | `ai_agent` |
| `a_agent_provider_binding` | `ai_agent_runtime_binding` |
| `a_agent_business_audit_event` | `ai_agent_audit_event` |
| `a_agent_deployment` | **removed** (v3 — dead code, no state machine) |
| `agents_app_registry` | **removed** (v3 — dead code, zero business reads) |
| `a_agent_knowledge_*` | **removed** → `sdkwork-knowledgebase` |
| `a_agent_memory_*` | **removed** → `sdkwork-memory` |
| `a_agent_mcp_server` | **removed** → `sdkwork-mcp` (`ai_mcp_server`) |

Migration scripts:

- `database/migrations/postgres/0001_ai_agent_core.up.sql` — legacy table renames and cleanup
- `database/migrations/postgres/0002_ai_agent_refactor.up.sql` — L2 tenant_entity compliance fields (`uuid`/`version`/`organization_id`), JSONB conversion, audit trail columns, soft-delete partial indexes, FK constraints, triggers, and documentation comments
- `database/migrations/postgres/0003_drop_unused_tables.up.sql` — drop `ai_app_registry`, `ai_agent_deployment`, `ai_agent_outbox_event` (dead code / over-design removal)

## 7. Authority

Canonical DDL: `database/ddl/baseline/postgres/0001_ai_agent_baseline.sql`
Contract registry: `database/contract/table-registry.json`
Schema contract: `database/contract/schema.yaml`

## 8. List Query Pagination (DATABASE_SPEC §16)

All agents list/search endpoints **must** push filtering, sorting, `LIMIT`, and `OFFSET` to SQL.
Application and frontend layers must not load full result sets and slice in memory.

| List surface | SQL authority | Notes |
| --- | --- | --- |
| `ai_agent` | `SQL_LIST_AGENT` + `SQL_COUNT_AGENT` | Filters: tenant, org, owner, deleted, `q`, `visibility`. Stable sort: `updated_at DESC, id DESC`. |
| `ai_agent_runtime_binding` | `SQL_LIST_AGENT_PROVIDER_BINDINGS` + `SQL_COUNT_AGENT_PROVIDER_BINDINGS` | Per-agent bindings; sort: active desc, updated_at desc. |
| `ai_agent_composition_slot` | `SQL_LIST_AGENT_COMPOSITION_SLOTS` + `SQL_COUNT_AGENT_COMPOSITION_SLOTS` | Excludes soft-deleted slots. |
| MCP marketplace projection | `SQL_LIST_MCP_MARKETPLACE_SLOTS` + `SQL_COUNT_MCP_MARKETPLACE_SLOTS` | Join `ai_agent` + `slot_kind = mcp`; no N+1 agent scan. |
| `ai_agent_audit_event` | `SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID` + `SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID` | Filters: `action`, `from`, `to`; sort: `created_at DESC, id DESC`. |
| `ai_agent_session` | `SQL_LIST_AGENT_SESSIONS` + `SQL_COUNT_AGENT_SESSIONS` | Offset-paginated; `PageInfo.totalItems` from count query. |
| `ai_agent_message` | `SQL_LIST_AGENT_MESSAGES` / `SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT` + `SQL_COUNT_AGENT_MESSAGES` | API lists use ASC + offset; chat context uses recent DESC window. |
| `ai_agent_interaction` | `SQL_LIST_AGENT_INTERACTIONS` + `SQL_COUNT_AGENT_INTERACTIONS` | Offset-paginated; domain-only until OpenAPI exposes HTTP. |

HTTP handlers pass `page` / `page_size` into repository `PaginationParams` and return `PageInfo.totalItems` from `COUNT(*)` queries.
Marketplace scope (`scope=market|public|published`) maps to `visibility = public` in SQL, not client-side filtering.
