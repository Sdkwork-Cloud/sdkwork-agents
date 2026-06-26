-- SDKWork Agents AI composition-plane PostgreSQL schema (canonical)
-- Domain: intelligence / agents-platform
-- Knowledge and memory data planes live in sibling modules (sdkwork-knowledgebase, sdkwork-memory).
-- SDKWork Agents AI composition-plane PostgreSQL baseline
-- Domain: intelligence / agents-platform
-- Contract: database/contract/schema.yaml
-- Knowledge, memory, skills, prompts, and drive content are owned by sibling modules.

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

CREATE TABLE IF NOT EXISTS ai_app_registry (
    id BIGINT NOT NULL PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    application_key TEXT NOT NULL,
    kernel_slot_id TEXT NOT NULL,
    default_agent_id VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_ai_app_registry_tenant_app UNIQUE (tenant_id, application_key)
);

CREATE INDEX IF NOT EXISTS idx_ai_app_registry_tenant_updated
    ON ai_app_registry (tenant_id, updated_at DESC);

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

CREATE TABLE IF NOT EXISTS ai_agent_deployment (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    deployment_id VARCHAR(128) NOT NULL,
    binding_id VARCHAR(128) NOT NULL,
    provider_id_snapshot VARCHAR(128) NOT NULL,
    implementation_kind_snapshot VARCHAR(64) NOT NULL,
    configuration_profile_id_snapshot VARCHAR(128) NOT NULL,
    capabilities_snapshot_json TEXT NOT NULL DEFAULT '[]',
    status SMALLINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_deployment_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_deployment_tenant_agent_deployment UNIQUE (
        tenant_id,
        agent_id,
        deployment_id
    ),
    CONSTRAINT ck_ai_agent_deployment_implementation_kind CHECK (
        implementation_kind_snapshot IN (
            'manifest-only',
            'typed-local-provider',
            'process-adapter',
            'protocol-adapter'
        )
    ),
    CONSTRAINT ck_ai_agent_deployment_deployment_id_standard CHECK (
        deployment_id ~ '^deployment\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_deployment_binding_id_standard CHECK (
        binding_id ~ '^binding\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_deployment_provider_id_snapshot_standard CHECK (
        provider_id_snapshot ~ '^provider\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_deployment_configuration_profile_id_snapshot_standard CHECK (
        configuration_profile_id_snapshot ~ '^profile\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_deployment_capabilities_snapshot_standard CHECK (
        sdkwork_intelligence_agents_service_capabilities_json_is_standard(capabilities_snapshot_json)
    ),
    CONSTRAINT ck_ai_agent_deployment_status CHECK (status IN (0, 1, 2, 3))
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_deployment_tenant_agent_created
    ON ai_agent_deployment (tenant_id, agent_id, created_at DESC, deployment_id ASC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_deployment_tenant_provider_status
    ON ai_agent_deployment (tenant_id, provider_id_snapshot, status);

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
        slot_kind IN ('memory', 'knowledge', 'skill', 'prompt', 'drive', 'tool')
    ),
    CONSTRAINT ck_ai_agent_composition_slot_module CHECK (
        target_module IN ('memory', 'knowledgebase', 'skills', 'prompts', 'drive')
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
            'deployment_created',
            'composition_slot_created',
            'composition_slot_updated',
            'composition_slot_deleted',
            'mcp_created',
            'mcp_updated',
            'mcp_deleted',
            'mcp_restored'
        )
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_agent_created
    ON ai_agent_audit_event (tenant_id, agent_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_action_created
    ON ai_agent_audit_event (tenant_id, action, created_at DESC);

CREATE TABLE IF NOT EXISTS ai_agent_outbox_event (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    aggregate_type VARCHAR(64) NOT NULL,
    aggregate_id BIGINT NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    payload_json JSONB NOT NULL,
    status SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT uk_ai_agent_outbox_event_uuid UNIQUE (tenant_id, uuid)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_outbox_event_status_created
    ON ai_agent_outbox_event (tenant_id, status, created_at);

-- Legacy MCP marketplace table (deferred migration to sdkwork-mcp).
-- Retained for runtime compatibility until the MCP module database lands.

CREATE TABLE IF NOT EXISTS a_agent_mcp_server (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL,
    mcp_server_id VARCHAR(128) NOT NULL,
    code VARCHAR(128) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    protocol_version VARCHAR(32) NOT NULL,
    transport_kind VARCHAR(32) NOT NULL,
    endpoint_ref VARCHAR(128),
    command_ref VARCHAR(128),
    auth_kind VARCHAR(32) NOT NULL,
    auth_profile_id VARCHAR(128),
    capability_ids_json TEXT NOT NULL DEFAULT '[]',
    tool_count BIGINT NOT NULL DEFAULT 0,
    resource_count BIGINT NOT NULL DEFAULT 0,
    prompt_count BIGINT NOT NULL DEFAULT 0,
    capabilities_json TEXT NOT NULL DEFAULT '{}',
    categories_json TEXT NOT NULL DEFAULT '[]',
    tags_json TEXT NOT NULL DEFAULT '[]',
    security_profile_id VARCHAR(128),
    status SMALLINT NOT NULL,
    visibility SMALLINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP NULL,
    CONSTRAINT uk_a_agent_mcp_server_uuid UNIQUE (uuid),
    CONSTRAINT uk_a_agent_mcp_server_tenant_server UNIQUE (tenant_id, mcp_server_id),
    CONSTRAINT uk_a_agent_mcp_server_tenant_code UNIQUE (tenant_id, code),
    CONSTRAINT ck_a_agent_mcp_server_id_standard CHECK (
        mcp_server_id ~ '^mcp\.server\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_transport_kind CHECK (
        transport_kind IN ('stdio', 'http', 'sse', 'websocket')
    ),
    CONSTRAINT ck_a_agent_mcp_server_transport_refs CHECK (
        (transport_kind = 'stdio' AND command_ref IS NOT NULL)
        OR (transport_kind IN ('http', 'sse', 'websocket') AND endpoint_ref IS NOT NULL)
    ),
    CONSTRAINT ck_a_agent_mcp_server_endpoint_ref_standard CHECK (
        endpoint_ref IS NULL
        OR endpoint_ref ~ '^endpoint\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_command_ref_standard CHECK (
        command_ref IS NULL
        OR command_ref ~ '^command\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_auth_kind CHECK (
        auth_kind IN ('none', 'oauth2', 'api-key-ref', 'host-secret-ref')
    ),
    CONSTRAINT ck_a_agent_mcp_server_auth_profile_standard CHECK (
        auth_profile_id IS NULL
        OR auth_profile_id ~ '^profile\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_security_profile_standard CHECK (
        security_profile_id IS NULL
        OR security_profile_id ~ '^profile\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_capability_ids_standard CHECK (
        sdkwork_intelligence_agents_service_capabilities_json_is_standard(capability_ids_json)
    ),
    CONSTRAINT ck_a_agent_mcp_server_counts_non_negative CHECK (
        tool_count >= 0 AND resource_count >= 0 AND prompt_count >= 0
    ),
    CONSTRAINT ck_a_agent_mcp_server_capabilities_json CHECK (capabilities_json::jsonb IS NOT NULL),
    CONSTRAINT ck_a_agent_mcp_server_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_a_agent_mcp_server_visibility CHECK (visibility IN (0, 1, 2, 3))
);

CREATE INDEX IF NOT EXISTS idx_a_agent_mcp_server_tenant_org_status_updated
    ON a_agent_mcp_server (tenant_id, organization_id, status, updated_at DESC, code ASC);

CREATE INDEX IF NOT EXISTS idx_a_agent_mcp_server_tenant_transport_auth
    ON a_agent_mcp_server (tenant_id, transport_kind, auth_kind);

