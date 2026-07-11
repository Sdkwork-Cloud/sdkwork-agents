-- sdkwork:migration
-- id: 0002_jsonb_columns_and_fk_constraints
-- engine: postgres
-- module: agents
-- purpose: Revert JSONB columns back to TEXT and remove foreign key constraints
-- reversible: true
-- transactional: true
-- lock: lightweight
-- contract_version: 3.1.0

-- ============================================================================
-- Step 1: Drop foreign key constraints
-- ============================================================================
ALTER TABLE ai_agent_audit_event DROP CONSTRAINT IF EXISTS fk_ai_agent_audit_event_agent;
ALTER TABLE ai_agent_task DROP CONSTRAINT IF EXISTS fk_ai_agent_task_agent;
ALTER TABLE ai_agent_interaction DROP CONSTRAINT IF EXISTS fk_ai_agent_interaction_session;
ALTER TABLE ai_agent_message DROP CONSTRAINT IF EXISTS fk_ai_agent_message_session;
ALTER TABLE ai_agent_session DROP CONSTRAINT IF EXISTS fk_ai_agent_session_agent;
ALTER TABLE ai_agent_composition_slot DROP CONSTRAINT IF EXISTS fk_ai_agent_composition_slot_agent;
ALTER TABLE ai_agent_runtime_binding DROP CONSTRAINT IF EXISTS fk_ai_agent_runtime_binding_agent;

-- ============================================================================
-- Step 2: Drop UNIQUE(tenant_id, id) on ai_agent
-- ============================================================================
ALTER TABLE ai_agent DROP CONSTRAINT IF EXISTS uk_ai_agent_tenant_id;

-- ============================================================================
-- Step 3: Convert JSONB columns back to TEXT
-- ============================================================================

-- ai_agent table
ALTER TABLE ai_agent ALTER COLUMN tags_json DROP DEFAULT;
ALTER TABLE ai_agent ALTER COLUMN tags_json TYPE TEXT USING tags_json::text;
ALTER TABLE ai_agent ALTER COLUMN tags_json SET DEFAULT '[]';
ALTER TABLE ai_agent ALTER COLUMN default_code_task_intent_json TYPE TEXT USING default_code_task_intent_json::text;
ALTER TABLE ai_agent ALTER COLUMN manifest_json TYPE TEXT USING manifest_json::text;

-- ai_agent_runtime_binding table
ALTER TABLE ai_agent_runtime_binding ALTER COLUMN capabilities_json DROP DEFAULT;
ALTER TABLE ai_agent_runtime_binding ALTER COLUMN capabilities_json TYPE TEXT USING capabilities_json::text;
ALTER TABLE ai_agent_runtime_binding ALTER COLUMN capabilities_json SET DEFAULT '[]';

-- ai_agent_audit_event table
ALTER TABLE ai_agent_audit_event ALTER COLUMN payload_json TYPE TEXT USING payload_json::text;

-- ai_agent_session table
ALTER TABLE ai_agent_session ALTER COLUMN metadata_json DROP DEFAULT;
ALTER TABLE ai_agent_session ALTER COLUMN metadata_json TYPE TEXT USING metadata_json::text;
ALTER TABLE ai_agent_session ALTER COLUMN metadata_json SET DEFAULT '{}';

-- ai_agent_message table
ALTER TABLE ai_agent_message ALTER COLUMN artifacts_json DROP DEFAULT;
ALTER TABLE ai_agent_message ALTER COLUMN artifacts_json TYPE TEXT USING artifacts_json::text;
ALTER TABLE ai_agent_message ALTER COLUMN artifacts_json SET DEFAULT '[]';
ALTER TABLE ai_agent_message ALTER COLUMN metadata_json DROP DEFAULT;
ALTER TABLE ai_agent_message ALTER COLUMN metadata_json TYPE TEXT USING metadata_json::text;
ALTER TABLE ai_agent_message ALTER COLUMN metadata_json SET DEFAULT '{}';

-- ============================================================================
-- Step 4: Restore CHECK function to TEXT parameter version
-- ============================================================================
DROP FUNCTION IF EXISTS sdkwork_intelligence_agents_service_capabilities_json_is_standard(JSONB);

CREATE FUNCTION sdkwork_intelligence_agents_service_capabilities_json_is_standard(input TEXT)
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
