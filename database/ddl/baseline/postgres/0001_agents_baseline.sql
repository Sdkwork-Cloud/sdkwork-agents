-- SDKWork Agents AI composition-plane PostgreSQL schema (canonical)
-- Domain: intelligence / agents-platform
-- Contract: database/contract/schema.yaml
-- Knowledge, memory, skills, prompts, drive, and MCP content are owned by
-- sibling modules. Agents reference them exclusively through
-- ai_agent_composition_slot.
-- 8 tables: ai_agent, ai_agent_runtime_binding, ai_agent_composition_slot,
-- ai_agent_audit_event, ai_agent_session, ai_agent_message, ai_agent_interaction,
-- ai_agent_task.

CREATE OR REPLACE FUNCTION sdkwork_intelligence_agents_service_capabilities_json_is_standard(input JSONB)
RETURNS BOOLEAN
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
    payload JSONB;
BEGIN
    payload := input;
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
    manifest_json JSONB NOT NULL,
    manifest_schema_version VARCHAR(32),
    default_code_task_intent_json JSONB,
    implementation_provider_id VARCHAR(128),
    implementation_kind VARCHAR(64),
    implementation_type VARCHAR(64) NOT NULL DEFAULT 'sdkwork-native',
    status SMALLINT NOT NULL,
    visibility SMALLINT NOT NULL,
    tags_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT uk_ai_agent_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_tenant_agent_id UNIQUE (tenant_id, agent_id),
    CONSTRAINT uk_ai_agent_tenant_code UNIQUE (tenant_id, code),
    CONSTRAINT uk_ai_agent_tenant_id UNIQUE (tenant_id, id),
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

-- Index for list query with filters (organization_id, owner_user_id, deleted_at)
CREATE INDEX IF NOT EXISTS idx_ai_agent_list_filters
    ON ai_agent (tenant_id, organization_id, owner_user_id, deleted_at)
    WHERE deleted_at IS NULL;

-- GIN trigram indexes for ILIKE/LIKE text search (requires pg_trgm extension)
-- These enable fast case-insensitive substring search on agent_id, code, display_name, description
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_ai_agent_agent_id_trgm ON ai_agent USING gin (agent_id gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_ai_agent_code_trgm ON ai_agent USING gin (code gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_ai_agent_display_name_trgm ON ai_agent USING gin (display_name gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_ai_agent_description_trgm ON ai_agent USING gin (COALESCE(description, '') gin_trgm_ops);

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
    capabilities_json JSONB NOT NULL DEFAULT '[]'::jsonb,
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
    payload_json JSONB NOT NULL,
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
            'composition_slot_deleted',
            'session_created',
            'session_closed',
            'session_archived',
            'message_created',
            'message_failed'
        )
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_agent_created
    ON ai_agent_audit_event (tenant_id, agent_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_action_created
    ON ai_agent_audit_event (tenant_id, action, created_at DESC);

-- ============================================================================
-- Agent Session Management — conversation lifecycle persistence
-- Aligns with kernel lifecycle SPI (AgentSession, AgentRun, AgentStep)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_agent_session (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    title VARCHAR(512),
    status SMALLINT NOT NULL DEFAULT 0,
    provider_binding_id VARCHAR(128),
    model_id VARCHAR(128),
    message_count BIGINT NOT NULL DEFAULT 0,
    total_input_tokens BIGINT NOT NULL DEFAULT 0,
    total_output_tokens BIGINT NOT NULL DEFAULT 0,
    metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_message_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_session_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_session_tenant_session_id UNIQUE (tenant_id, session_id),
    CONSTRAINT ck_ai_agent_session_status CHECK (status IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_session_id_standard CHECK (
        session_id ~ '^session\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_session_provider_binding_id_standard CHECK (
        provider_binding_id IS NULL
        OR provider_binding_id ~ '^binding\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_session_tenant_agent_status_updated
    ON ai_agent_session (tenant_id, agent_id, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_session_tenant_owner_status
    ON ai_agent_session (tenant_id, owner_user_id, status);

CREATE INDEX IF NOT EXISTS idx_ai_agent_session_tenant_org_status
    ON ai_agent_session (tenant_id, organization_id, status, updated_at DESC);

-- ============================================================================
-- Agent Message Management — message persistence for query and display
-- Aligns with kernel message SPI (AgentMessage, AgentMessageRole, AgentPart)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_agent_message (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    message_id VARCHAR(128) NOT NULL,
    role SMALLINT NOT NULL,
    content TEXT NOT NULL,
    content_type VARCHAR(64) NOT NULL DEFAULT 'text/plain',
    status SMALLINT NOT NULL DEFAULT 0,
    sequence BIGINT NOT NULL,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    model_id VARCHAR(128),
    provider_id VARCHAR(128),
    artifacts_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    parent_message_id VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_message_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_message_tenant_session_message UNIQUE (tenant_id, session_id, message_id),
    CONSTRAINT uk_ai_agent_message_tenant_session_sequence UNIQUE (tenant_id, session_id, sequence),
    CONSTRAINT ck_ai_agent_message_role CHECK (role IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_message_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_message_id_standard CHECK (
        message_id ~ '^msg\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_message_provider_id_standard CHECK (
        provider_id IS NULL
        OR provider_id ~ '^provider\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_ai_agent_message_parent_message_id_standard CHECK (
        parent_message_id IS NULL
        OR parent_message_id ~ '^msg\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_message_tenant_session_sequence
    ON ai_agent_message (tenant_id, session_id, sequence ASC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_message_tenant_session_role_created
    ON ai_agent_message (tenant_id, session_id, role, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_message_tenant_agent_created
    ON ai_agent_message (tenant_id, agent_id, created_at DESC);

-- ============================================================================
-- Agent Interaction Management — live interaction persistence (approval / Q&A)
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_agent_interaction (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    engine_key VARCHAR(64) NOT NULL,
    interaction_id VARCHAR(128) NOT NULL,
    kind SMALLINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    prompt TEXT NOT NULL,
    options_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    resolution_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_interaction_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_interaction_tenant_session_interaction UNIQUE (tenant_id, session_id, interaction_id),
    CONSTRAINT ck_ai_agent_interaction_kind CHECK (kind IN (0, 1)),
    CONSTRAINT ck_ai_agent_interaction_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_interaction_id_standard CHECK (
        interaction_id ~ '^interaction\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_tenant_session_status
    ON ai_agent_interaction (tenant_id, session_id, status, created_at DESC);

-- ============================================================================
-- Agent Task Management — kernel AgentTask projection for scheduling
-- ============================================================================

CREATE TABLE IF NOT EXISTS ai_agent_task (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    task_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    title VARCHAR(512),
    prompt TEXT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    external_ref VARCHAR(256),
    metadata_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_task_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_task_tenant_task_id UNIQUE (tenant_id, task_id),
    CONSTRAINT ck_ai_agent_task_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_task_id_standard CHECK (
        task_id ~ '^task\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    )
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_task_tenant_agent_status_updated
    ON ai_agent_task (tenant_id, agent_id, status, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_task_tenant_owner_status
    ON ai_agent_task (tenant_id, owner_user_id, status);

-- ============================================================================
-- Foreign Key Constraints
-- All FKs include tenant_id to preserve multi-tenant isolation.
-- ============================================================================
ALTER TABLE ai_agent_runtime_binding
    ADD CONSTRAINT fk_ai_agent_runtime_binding_agent
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent(tenant_id, agent_id) ON DELETE CASCADE;

ALTER TABLE ai_agent_composition_slot
    ADD CONSTRAINT fk_ai_agent_composition_slot_agent
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent(tenant_id, agent_id) ON DELETE CASCADE;

ALTER TABLE ai_agent_session
    ADD CONSTRAINT fk_ai_agent_session_agent
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent(tenant_id, agent_id) ON DELETE CASCADE;

ALTER TABLE ai_agent_message
    ADD CONSTRAINT fk_ai_agent_message_session
    FOREIGN KEY (tenant_id, session_id) REFERENCES ai_agent_session(tenant_id, session_id) ON DELETE CASCADE;

ALTER TABLE ai_agent_interaction
    ADD CONSTRAINT fk_ai_agent_interaction_session
    FOREIGN KEY (tenant_id, session_id) REFERENCES ai_agent_session(tenant_id, session_id) ON DELETE CASCADE;

ALTER TABLE ai_agent_task
    ADD CONSTRAINT fk_ai_agent_task_agent
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent(tenant_id, agent_id) ON DELETE CASCADE;

ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT fk_ai_agent_audit_event_agent
    FOREIGN KEY (tenant_id, agent_internal_id) REFERENCES ai_agent(tenant_id, id) ON DELETE CASCADE;

