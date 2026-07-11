-- sdkwork:migration
-- id: 0002_jsonb_columns_and_fk_constraints
-- engine: postgres
-- module: agents
-- purpose: Migrate TEXT JSON columns to JSONB and add foreign key constraints
-- reversible: true
-- transactional: true
-- lock: lightweight
-- contract_version: 3.1.0

-- ============================================================================
-- Step 1: Update CHECK function signature from TEXT to JSONB
-- ============================================================================
-- CREATE OR REPLACE FUNCTION with a different argument type creates a new
-- function rather than replacing the old one, so DROP first then CREATE.

DROP FUNCTION IF EXISTS sdkwork_intelligence_agents_service_capabilities_json_is_standard(TEXT);

CREATE FUNCTION sdkwork_intelligence_agents_service_capabilities_json_is_standard(input JSONB)
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

-- ============================================================================
-- Step 2: Migrate TEXT columns to JSONB
-- ============================================================================

-- ai_agent table
ALTER TABLE ai_agent ALTER COLUMN manifest_json TYPE JSONB USING manifest_json::jsonb;
ALTER TABLE ai_agent ALTER COLUMN default_code_task_intent_json TYPE JSONB USING default_code_task_intent_json::jsonb;
ALTER TABLE ai_agent ALTER COLUMN tags_json TYPE JSONB USING tags_json::jsonb;
ALTER TABLE ai_agent ALTER COLUMN tags_json SET DEFAULT '[]'::jsonb;

-- ai_agent_runtime_binding table
ALTER TABLE ai_agent_runtime_binding ALTER COLUMN capabilities_json TYPE JSONB USING capabilities_json::jsonb;
ALTER TABLE ai_agent_runtime_binding ALTER COLUMN capabilities_json SET DEFAULT '[]'::jsonb;

-- ai_agent_audit_event table
ALTER TABLE ai_agent_audit_event ALTER COLUMN payload_json TYPE JSONB USING payload_json::jsonb;

-- ai_agent_session table
ALTER TABLE ai_agent_session ALTER COLUMN metadata_json TYPE JSONB USING metadata_json::jsonb;
ALTER TABLE ai_agent_session ALTER COLUMN metadata_json SET DEFAULT '{}'::jsonb;

-- ai_agent_message table
ALTER TABLE ai_agent_message ALTER COLUMN artifacts_json TYPE JSONB USING artifacts_json::jsonb;
ALTER TABLE ai_agent_message ALTER COLUMN artifacts_json SET DEFAULT '[]'::jsonb;
ALTER TABLE ai_agent_message ALTER COLUMN metadata_json TYPE JSONB USING metadata_json::jsonb;
ALTER TABLE ai_agent_message ALTER COLUMN metadata_json SET DEFAULT '{}'::jsonb;

-- ============================================================================
-- Step 3: Add UNIQUE(tenant_id, id) on ai_agent for audit_event FK
-- ============================================================================
-- id is the PRIMARY KEY, but tenant_id + id combination requires an explicit
-- UNIQUE constraint to be referenceable as a foreign key target.
ALTER TABLE ai_agent ADD CONSTRAINT uk_ai_agent_tenant_id UNIQUE (tenant_id, id);

-- ============================================================================
-- Step 4: Add foreign key constraints
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
