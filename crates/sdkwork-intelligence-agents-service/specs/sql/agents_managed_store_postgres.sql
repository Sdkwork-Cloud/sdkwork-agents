-- SDKWork Agents AI composition-plane PostgreSQL schema (canonical)
-- Domain: intelligence / agents-platform
-- Contract: database/contract/schema.yaml
-- Knowledge, memory, skills, prompts, drive, and MCP content are owned by
-- sibling modules. Agents reference them exclusively through
-- ai_agent_composition_slot.
-- 4 core tables: ai_agent, ai_agent_runtime_binding,
-- ai_agent_composition_slot, ai_agent_audit_event.

CREATE OR REPLACE FUNCTION sdkwork_intelligence_agents_service_capabilities_json_is_standard(input TEXT)
RETURNS BOOLEAN
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
    payload JSONB;
BEGIN
    payload := input::jsonb;
    IF jsonb_typeof(payload) <> 'array' THEN
        RETURN FALSE;
    END IF;

    RETURN NOT EXISTS (
        SELECT 1
        FROM jsonb_array_elements(payload) AS capability_values(value)
        WHERE NOT (
            jsonb_typeof(capability_values.value) = 'string'
            AND char_length(capability_values.value #>> '{}') <= 128
            AND (capability_values.value #>> '{}') ~ '^[a-z0-9_-]+(\.[a-z0-9_-]+)+$'
        )
    )
    AND (
        SELECT COUNT(*)
        FROM jsonb_array_elements(payload) AS capability_values(value)
    ) = (
        SELECT COUNT(DISTINCT capability_values.value #>> '{}')
        FROM jsonb_array_elements(payload) AS capability_values(value)
    );
EXCEPTION WHEN others THEN
    RETURN FALSE;
END;
$$;

CREATE TABLE IF NOT EXISTS ai_agent (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    code VARCHAR(128) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    manifest_json TEXT NOT NULL,
    manifest_schema_version VARCHAR(32),
    default_code_task_intent_json TEXT,
    implementation_provider_id VARCHAR(128),
    implementation_kind VARCHAR(64),
    implementation_type VARCHAR(64) NOT NULL DEFAULT 'sdkwork-native',
    status SMALLINT NOT NULL,
    visibility SMALLINT NOT NULL,
    tags_json TEXT NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT uk_ai_agent_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_tenant_agent_id UNIQUE (tenant_id, agent_id),
    CONSTRAINT uk_ai_agent_tenant_code UNIQUE (tenant_id, code),
    CONSTRAINT ck_ai_agent_implementation_kind CHECK (
        implementation_kind IS NULL OR implementation_kind IN (
            'manifest-only',
            'typed-local-provider',
            'process-adapter',
            'protocol-adapter'
        )
    ),
    CONSTRAINT ck_ai_agent_implementation_type CHECK (
        implementation_type IN (
            'sdkwork-native',
            'rig-rust',
            'openai-agents',
            'langchain',
            'langgraph',
            'crewai',
            'autogen',
            'semantic-kernel',
            'custom'
        )
    ),
    CONSTRAINT ck_ai_agent_implementation_provider_id_standard CHECK (
        implementation_provider_id IS NULL
        OR implementation_provider_id ~ '^provider\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_visibility CHECK (visibility IN (0, 1, 2, 3))
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_org_status_updated
    ON ai_agent (tenant_id, organization_id, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_owner_status
    ON ai_agent (tenant_id, owner_user_id, status);

CREATE TABLE IF NOT EXISTS ai_agent_runtime_binding (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    binding_id VARCHAR(128) NOT NULL,
    provider_id VARCHAR(128) NOT NULL,
    implementation_kind VARCHAR(64) NOT NULL,
    configuration_profile_id VARCHAR(128) NOT NULL,
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    active BOOLEAN NOT NULL DEFAULT FALSE,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_runtime_binding_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_runtime_binding_tenant_agent_binding UNIQUE (
        tenant_id,
        agent_id,
        binding_id
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_implementation_kind CHECK (
        implementation_kind IN (
            'manifest-only',
            'typed-local-provider',
            'process-adapter',
            'protocol-adapter'
        )
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_binding_id_standard CHECK (
        binding_id ~ '^binding\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_provider_id_standard CHECK (
        provider_id ~ '^provider\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_configuration_profile_id_standard CHECK (
        configuration_profile_id ~ '^profile\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_runtime_binding_capabilities_standard CHECK (
        sdkwork_intelligence_agents_service_capabilities_json_is_standard(capabilities_json)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_runtime_binding_active_default
    ON ai_agent_runtime_binding (tenant_id, agent_id)
    WHERE active = TRUE;

CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_tenant_agent_updated
    ON ai_agent_runtime_binding (tenant_id, agent_id, active DESC, updated_at DESC, binding_id ASC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_tenant_provider
    ON ai_agent_runtime_binding (tenant_id, provider_id);

CREATE TABLE IF NOT EXISTS ai_agent_composition_slot (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    slot_id VARCHAR(128) NOT NULL,
    slot_kind VARCHAR(64) NOT NULL,
    target_module VARCHAR(64) NOT NULL,
    target_ref VARCHAR(256) NOT NULL,
    target_version_ref VARCHAR(128),
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    policy_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    status SMALLINT NOT NULL DEFAULT 1,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_composition_slot_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_composition_slot_tenant_agent_slot UNIQUE (tenant_id, agent_id, slot_id),
    CONSTRAINT ck_ai_agent_composition_slot_kind CHECK (
        slot_kind IN ('memory', 'knowledge', 'skill', 'prompt', 'drive', 'tool', 'mcp')
    ),
    CONSTRAINT ck_ai_agent_composition_slot_module CHECK (
        target_module IN ('memory', 'knowledgebase', 'skills', 'prompts', 'drive', 'mcp')
    ),
    CONSTRAINT ck_ai_agent_composition_slot_id_standard CHECK (
        slot_id ~ '^slot\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_composition_slot_status CHECK (status IN (0, 1, 2, 3, 4))
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_composition_slot_lookup
    ON ai_agent_composition_slot (tenant_id, agent_id, slot_kind, enabled, priority, slot_id);

CREATE TABLE IF NOT EXISTS ai_agent_audit_event (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_internal_id BIGINT NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    action VARCHAR(64) NOT NULL,
    subject_id VARCHAR(128) NOT NULL,
    subject_tenant_id VARCHAR(128) NOT NULL,
    request_id VARCHAR(128),
    trace_id VARCHAR(128),
    payload_json TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_audit_event_uuid UNIQUE (uuid),
    CONSTRAINT ck_ai_agent_audit_action CHECK (
        action IN (
            'created',
            'updated',
            'deleted',
            'restored',
            'status_changed',
            'started',
            'completed',
            'failed',
            'cancelled',
            'provider_binding_changed',
            'composition_slot_created',
            'composition_slot_updated',
            'composition_slot_deleted'
        )
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_agent_created
    ON ai_agent_audit_event (tenant_id, agent_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_action_created
    ON ai_agent_audit_event (tenant_id, action, created_at DESC);
