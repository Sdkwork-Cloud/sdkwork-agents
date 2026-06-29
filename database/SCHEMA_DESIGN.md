# Agents Database Schema Design (v3)

## Architecture Overview

### Composition-Plane Pattern

The agents database follows a **composition-plane architecture** that achieves high cohesion and low coupling through clear module boundaries:

```
┌─────────────────────────────────────────────────────────┐
│                    sdkwork-agents                        │
│  ┌──────────────────────────────────────────────────┐   │
│  │             Core Agent Lifecycle                  │   │
│  │  • ai_agent (identity, manifest, lifecycle)       │   │
│  │  • ai_agent_runtime_binding (provider config)     │   │
│  │  • ai_agent_composition_slot (cross-module refs)  │   │
│  │  • ai_agent_audit_event (audit trail)             │   │
│  └──────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────┘
                         │
          ai_agent_composition_slot (target_module + target_ref)
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ sdkwork-     │ │ sdkwork-     │ │ sdkwork-     │
│ memory       │ │ knowledge-   │ │ skills       │
│              │ │ base         │ │              │
└──────────────┘ └──────────────┘ └──────────────┘
        │                │                │
        ▼                ▼                ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ sdkwork-     │ │ sdkwork-     │ │ sdkwork-     │
│ prompts      │ │ drive        │ │ mcp          │
└──────────────┘ └──────────────┘ └──────────────┘
```

### Design Principles

1. **Single Responsibility**: Each table owns exactly one domain concept
2. **Explicit Boundaries**: Cross-module references only through `ai_agent_composition_slot`
3. **Multi-Tenant Isolation**: All queries scoped by `tenant_id` with optional RLS
4. **Soft-Delete Safety**: Unique constraints use partial indexes excluding soft-deleted records
5. **Immutable Audit**: Append-only audit log with no UPDATE/DELETE capability
6. **No Over-Design**: Only 6 tables — identity, binding, composition, audit, session, message

### Removed Tables (v3 simplification)

The following tables were removed because they were dead code or over-designed:

| Removed Table | Reason |
| --- | --- |
| `ai_app_registry` | Dead code: schema + seed existed but zero business reads in Rust source |
| `ai_agent_deployment` | Incomplete: only INSERT/LIST implemented, no state machine transitions |
| `ai_agent_outbox_event` | Dead code: no publisher, no consumer, Outbox Pattern never implemented |

## Table Details

### ai_agent

| Column | Type | Description |
|--------|------|-------------|
| id | BIGINT | Primary key |
| uuid | VARCHAR(64) | Global unique identifier |
| tenant_id | BIGINT | Multi-tenant isolation key |
| organization_id | BIGINT | Organization within tenant |
| owner_user_id | BIGINT | Creating user |
| agent_id | VARCHAR(128) | Business identifier |
| code | VARCHAR(128) | Human-readable code |
| display_name | VARCHAR(255) | Display name |
| description | TEXT | Optional description |
| manifest_json | TEXT | Full agent manifest |
| implementation_kind | VARCHAR(64) | Provider type |
| implementation_type | VARCHAR(64) | Framework type |
| status | SMALLINT | 0=draft, 1=active, 2=disabled, 3=archived, 4=deleted |
| visibility | SMALLINT | 0=private, 1=internal, 2=public, 3=marketplace |
| tags_json | TEXT | Tag array |
| version | BIGINT | Optimistic locking |
| deleted_at | TIMESTAMPTZ | Soft-delete timestamp |

**Indexes:**
- `uk_ai_agent_uuid` - Global unique
- `uk_ai_agent_tenant_agent_id` - Unique business key
- `uk_ai_agent_tenant_code` - Unique code within tenant
- `idx_ai_agent_tenant_org_status_updated` - List query optimization
- `idx_ai_agent_tenant_owner_status` - Owner-owned agents query

### ai_agent_runtime_binding

| Column | Type | Description |
|--------|------|-------------|
| binding_id | VARCHAR(128) | Business identifier |
| provider_id | VARCHAR(128) | Provider reference |
| implementation_kind | VARCHAR(64) | Provider type |
| configuration_profile_id | VARCHAR(128) | Config profile |
| capabilities_json | TEXT | Declared capabilities |
| active | BOOLEAN | Single active binding per agent |
| version | BIGINT | Optimistic locking |

**Partial Unique Index:**
```sql
uk_ai_agent_runtime_binding_active_default
    ON ai_agent_runtime_binding (tenant_id, agent_id)
    WHERE active = TRUE;
```
*Ensures only ONE active binding per agent*

### ai_agent_composition_slot

| Column | Type | Description |
|--------|------|-------------|
| slot_id | VARCHAR(128) | Business identifier |
| slot_kind | VARCHAR(64) | memory, knowledge, skill, prompt, drive, tool, mcp |
| target_module | VARCHAR(64) | memory, knowledgebase, skills, prompts, drive, mcp |
| target_ref | VARCHAR(256) | Reference ID in target module |
| target_version_ref | VARCHAR(128) | Optional version pin |
| priority | INTEGER | Execution priority |
| enabled | BOOLEAN | Slot active flag |
| policy_json | JSONB | Access control policy |
| status | SMALLINT | 0=disabled, 1=active, 2=error, 3=deprecated, 4=deleted |

**Cross-Module Reference Pattern:**
```sql
-- Example: Bind agent to memory store
INSERT INTO ai_agent_composition_slot (
    agent_id, slot_id, slot_kind, target_module, target_ref
) VALUES (
    'agent.user-assistant',
    'slot.memory.short-term',
    'memory',
    'memory',
    'mem_store.user_123_short_term'
);
```

### ai_agent_audit_event

**Immutable append-only audit log.** No UPDATE or DELETE operations allowed.

| Column | Type | Description |
|--------|------|-------------|
| agent_internal_id | BIGINT | FK to ai_agent.id |
| agent_id | VARCHAR(128) | Business identifier (denormalized) |
| action | VARCHAR(64) | Audit action type |
| subject_id | VARCHAR(128) | Acting user/service |
| subject_tenant_id | VARCHAR(128) | Subject's tenant |
| request_id | VARCHAR(128) | Request correlation ID |
| trace_id | VARCHAR(128) | Distributed trace ID |
| payload_json | TEXT | Action-specific details |

**Audit Actions:**
- `created`, `updated`, `deleted`, `restored`, `status_changed`
- `provider_binding_changed`
- `composition_slot_created`, `composition_slot_updated`, `composition_slot_deleted`

## Module Boundaries

### Owned by sdkwork-agents

| Table | Responsibility |
|-------|----------------|
| ai_agent | Agent identity, manifest, lifecycle |
| ai_agent_runtime_binding | Provider runtime configuration |
| ai_agent_composition_slot | Cross-module resource binding |
| ai_agent_audit_event | Agent management audit log |

### Owned by Sibling Modules

| Module | Tables (examples) | Reference Method |
|--------|-------------------|------------------|
| sdkwork-memory | memory_store, memory_record, memory_binding | `target_module='memory'`, `target_ref='<memory_id>'` |
| sdkwork-knowledgebase | knowledge_base, knowledge_document, knowledge_index | `target_module='knowledgebase'`, `target_ref='<kb_id>'` |
| sdkwork-skills | skill, skill_version, skill_execution | `target_module='skills'`, `target_ref='<skill_id>'` |
| sdkwork-prompts | prompt_template, prompt_version | `target_module='prompts'`, `target_ref='<prompt_id>'` |
| sdkwork-drive | drive_file, drive_folder | `target_module='drive'`, `target_ref='<file_id>'` |
| sdkwork-mcp | mcp_server, mcp_tool, mcp_resource | `target_module='mcp'`, `target_ref='<server_id>'` |

**CRITICAL RULE**: The agents module **MUST NOT** create tables for sibling module domains. All cross-module references go through `ai_agent_composition_slot`.

## Industry Alignment

This schema design aligns with:

1. **OpenAI Assistants**: Agent identity + tools binding pattern (composition_slot = tools array)
2. **Dify**: Agent + model_config separation (runtime_binding = app_models)
3. **Coze**: Bot + plugin binding (composition_slot = plugin_bindings)
4. **SOC 2 Type II**: Immutable audit trail, access controls
5. **GDPR**: Soft-delete, data subject accountability

## Migration History

| Version | Migration | Description |
| --- | --- | --- |
| v1 → v2 | `0001_ai_agent_core.up.sql` | Legacy `a_agent_*` table renames |
| v2 → v3 | `0002_ai_agent_refactor.up.sql` | L2 compliance fields, JSONB, audit trail, FKs, triggers |
| v3 simplify | `0003_drop_unused_tables.up.sql` | Drop dead code: `ai_app_registry`, `ai_agent_deployment`, `ai_agent_outbox_event` |

## Compliance Checklist

- [x] Multi-tenant data isolation (tenant_id + organization_id)
- [x] Immutable audit trail (append-only, no UPDATE/DELETE)
- [x] Soft-delete support (deleted_at column)
- [x] Data validation (CHECK constraints, validated JSON)
- [x] Cross-module boundary enforcement (composition_slot only)
- [x] Optimistic locking (version columns)
- [x] Index optimization (tenant-scoped, composite)
- [x] No dead code (all 6 tables have active business logic)
