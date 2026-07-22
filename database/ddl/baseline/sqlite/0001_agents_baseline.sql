-- SDKWork Agents SQLite development subset.
-- PostgreSQL is the only managed-store authority for contract 5.0.0.
-- This subset supports agent control-plane development only; it intentionally
-- excludes the durable Session aggregate and must not be treated as parity DDL.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS ai_agent (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL,
    agent_id TEXT NOT NULL,
    code TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    manifest_json TEXT NOT NULL CHECK (json_valid(manifest_json) AND json_type(manifest_json) = 'object'),
    manifest_schema_version TEXT,
    default_code_task_intent_json TEXT CHECK (
        default_code_task_intent_json IS NULL
        OR (json_valid(default_code_task_intent_json) AND json_type(default_code_task_intent_json) = 'object')
    ),
    implementation_provider_id TEXT,
    implementation_kind TEXT CHECK (
        implementation_kind IS NULL OR implementation_kind IN (
            'manifest-only', 'typed-local-provider', 'process-adapter', 'protocol-adapter'
        )
    ),
    implementation_type TEXT NOT NULL DEFAULT 'sdkwork-native' CHECK (
        implementation_type IN (
            'sdkwork-native', 'rig-rust', 'openai-agents', 'langchain', 'langgraph',
            'crewai', 'autogen', 'semantic-kernel', 'custom'
        )
    ),
    status INTEGER NOT NULL CHECK (status IN (0, 1, 2, 3, 4)),
    visibility INTEGER NOT NULL CHECK (visibility IN (0, 1, 2, 3)),
    tags_json TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(tags_json) AND json_type(tags_json) = 'array'
    ),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    version BIGINT NOT NULL DEFAULT 0 CHECK (version >= 0),
    UNIQUE (tenant_id, agent_id),
    UNIQUE (tenant_id, code),
    UNIQUE (tenant_id, id)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_org_status_updated
    ON ai_agent (tenant_id, organization_id, status, updated_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_owner_status
    ON ai_agent (tenant_id, owner_user_id, status, updated_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_list_filters
    ON ai_agent (tenant_id, organization_id, owner_user_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_runtime_binding (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id TEXT NOT NULL,
    binding_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    implementation_kind TEXT NOT NULL CHECK (
        implementation_kind IN (
            'manifest-only', 'typed-local-provider', 'process-adapter', 'protocol-adapter'
        )
    ),
    configuration_profile_id TEXT NOT NULL,
    capabilities_json TEXT NOT NULL DEFAULT '[]' CHECK (
        json_valid(capabilities_json) AND json_type(capabilities_json) = 'array'
    ),
    active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1)),
    version BIGINT NOT NULL DEFAULT 1 CHECK (version >= 1),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, agent_id, binding_id),
    FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_runtime_binding_active
    ON ai_agent_runtime_binding (tenant_id, agent_id) WHERE active = 1;
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_agent_updated
    ON ai_agent_runtime_binding (
        tenant_id, agent_id, active DESC, updated_at DESC, binding_id ASC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_provider
    ON ai_agent_runtime_binding (tenant_id, provider_id, updated_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS ai_agent_composition_slot (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id TEXT NOT NULL,
    slot_id TEXT NOT NULL,
    slot_kind TEXT NOT NULL CHECK (
        slot_kind IN ('memory', 'knowledge', 'skill', 'prompt', 'drive', 'tool', 'mcp')
    ),
    target_module TEXT NOT NULL CHECK (
        target_module IN ('memory', 'knowledgebase', 'skills', 'prompts', 'drive', 'mcp')
    ),
    target_ref TEXT NOT NULL,
    target_version_ref TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    policy_json TEXT NOT NULL DEFAULT '{}' CHECK (
        json_valid(policy_json) AND json_type(policy_json) = 'object'
    ),
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (0, 1, 2, 3, 4)),
    version BIGINT NOT NULL DEFAULT 0 CHECK (version >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    UNIQUE (tenant_id, agent_id, slot_id),
    FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_composition_slot_lookup
    ON ai_agent_composition_slot (
        tenant_id, agent_id, slot_kind, enabled, priority, slot_id
    ) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_audit_event (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    agent_internal_id BIGINT,
    agent_id TEXT,
    action TEXT NOT NULL,
    actor_type INTEGER NOT NULL CHECK (actor_type IN (0, 1, 2)),
    actor_id BIGINT NOT NULL,
    request_id TEXT,
    trace_id TEXT,
    payload_json TEXT NOT NULL CHECK (
        json_valid(payload_json) AND json_type(payload_json) = 'object'
    ),
    created_at TEXT NOT NULL,
    retention_until TEXT,
    CHECK (
        (aggregate_type = 'agent' AND agent_internal_id IS NOT NULL AND agent_id IS NOT NULL)
        OR (aggregate_type <> 'agent' AND agent_internal_id IS NULL AND agent_id IS NULL)
    ),
    FOREIGN KEY (tenant_id, agent_internal_id)
        REFERENCES ai_agent (tenant_id, id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_aggregate_created
    ON ai_agent_audit_event (
        tenant_id, organization_id, aggregate_type, aggregate_id, created_at DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_action_created
    ON ai_agent_audit_event (
        tenant_id, organization_id, action, created_at DESC, id DESC
    );
