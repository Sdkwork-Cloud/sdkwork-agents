-- Rollback: Refactored schema v3
-- Reverts changes from 0002_ai_agent_refactor.up.sql

-- ═══════════════════════════════════════════════════════
-- Step 1: Drop view
-- ═══════════════════════════════════════════════════════
DROP VIEW IF EXISTS v_ai_agent_active_binding;

-- ═══════════════════════════════════════════════════════
-- Step 2: Drop triggers
-- ═══════════════════════════════════════════════════════
DROP TRIGGER IF EXISTS trg_ai_agent_updated_at ON ai_agent;
DROP TRIGGER IF EXISTS trg_ai_agent_runtime_binding_updated_at ON ai_agent_runtime_binding;
DROP TRIGGER IF EXISTS trg_ai_agent_composition_slot_updated_at ON ai_agent_composition_slot;

-- ═══════════════════════════════════════════════════════
-- Step 3: Drop helper functions
-- ═══════════════════════════════════════════════════════
DROP FUNCTION IF EXISTS fnai_update_updated_at();
DROP FUNCTION IF EXISTS fnai_is_standard_id(TEXT, TEXT);
DROP FUNCTION IF EXISTS fnai_validate_capabilities_json(TEXT);

-- ═══════════════════════════════════════════════════════
-- Step 4: Remove audit trail and L2 compliance columns
-- ═══════════════════════════════════════════════════════

-- Drop L2 unique constraint first (depends on uuid column)
ALTER TABLE ai_app_registry
    DROP CONSTRAINT IF EXISTS uk_ai_app_registry_uuid;

ALTER TABLE ai_agent
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS updated_by,
    DROP COLUMN IF EXISTS deleted_by;

ALTER TABLE ai_agent_runtime_binding
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS updated_by,
    DROP COLUMN IF EXISTS organization_id;

ALTER TABLE ai_agent_deployment
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS organization_id;

ALTER TABLE ai_agent_composition_slot
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS updated_by;

ALTER TABLE ai_app_registry
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS updated_by,
    DROP COLUMN IF EXISTS uuid,
    DROP COLUMN IF EXISTS version;

-- ═══════════════════════════════════════════════════════
-- Step 5: Revert JSONB columns back to TEXT
-- ═══════════════════════════════════════════════════════
ALTER TABLE ai_agent
    ALTER COLUMN manifest_json TYPE TEXT USING manifest_json::text,
    ALTER COLUMN manifest_json SET NOT NULL;

ALTER TABLE ai_agent
    ALTER COLUMN default_code_task_intent_json TYPE TEXT USING default_code_task_intent_json::text;

ALTER TABLE ai_agent
    ALTER COLUMN tags_json TYPE TEXT USING tags_json::text,
    ALTER COLUMN tags_json SET DEFAULT '[]';

ALTER TABLE ai_agent_runtime_binding
    ALTER COLUMN capabilities_json TYPE TEXT USING capabilities_json::text,
    ALTER COLUMN capabilities_json SET DEFAULT '[]';

ALTER TABLE ai_agent_deployment
    ALTER COLUMN capabilities_snapshot_json TYPE TEXT USING capabilities_snapshot_json::text,
    ALTER COLUMN capabilities_snapshot_json SET DEFAULT '[]';

ALTER TABLE ai_agent_audit_event
    ALTER COLUMN payload_json TYPE TEXT USING payload_json::text,
    ALTER COLUMN payload_json SET NOT NULL;

-- ═══════════════════════════════════════════════════════
-- Step 6: Drop partial unique indexes for soft-delete
-- ═══════════════════════════════════════════════════════
DROP INDEX IF EXISTS uk_ai_agent_tenant_code;
DROP INDEX IF EXISTS uk_ai_agent_composition_slot_tenant_agent_slot;

-- Recreate original full unique constraints
ALTER TABLE ai_agent
    ADD CONSTRAINT uk_ai_agent_tenant_code UNIQUE (tenant_id, code);

ALTER TABLE ai_agent_composition_slot
    ADD CONSTRAINT uk_ai_agent_composition_slot_tenant_agent_slot
        UNIQUE (tenant_id, agent_id, slot_id);

-- ═══════════════════════════════════════════════════════
-- Step 7: Drop new indexes
-- ═══════════════════════════════════════════════════════
DROP INDEX IF EXISTS idx_ai_agent_tenant_visibility;
DROP INDEX IF EXISTS idx_ai_agent_composition_slot_target;
DROP INDEX IF EXISTS idx_ai_agent_audit_tenant_org_created;

-- ═══════════════════════════════════════════════════════
-- Step 8: Drop new CHECK constraints
-- ═══════════════════════════════════════════════════════
ALTER TABLE ai_agent
    DROP CONSTRAINT IF EXISTS ck_ai_agent_implementation_provider_id_v2;

ALTER TABLE ai_agent_runtime_binding
    DROP CONSTRAINT IF EXISTS ck_ai_agent_runtime_binding_capabilities_v2;

-- ═══════════════════════════════════════════════════════
-- Step 9: Drop foreign key constraints
-- ═══════════════════════════════════════════════════════
ALTER TABLE ai_agent_runtime_binding
    DROP CONSTRAINT IF EXISTS fk_ai_agent_runtime_binding_agent;

ALTER TABLE ai_agent_deployment
    DROP CONSTRAINT IF EXISTS fk_ai_agent_deployment_agent;

ALTER TABLE ai_agent_composition_slot
    DROP CONSTRAINT IF EXISTS fk_ai_agent_composition_slot_agent;

-- ═══════════════════════════════════════════════════════
-- Step 10: Remove comments
-- ═══════════════════════════════════════════════════════
COMMENT ON TABLE ai_agent IS NULL;
COMMENT ON COLUMN ai_agent.manifest_json IS NULL;
COMMENT ON COLUMN ai_agent.status IS NULL;
COMMENT ON COLUMN ai_agent.visibility IS NULL;

COMMENT ON TABLE ai_agent_runtime_binding IS NULL;
COMMENT ON COLUMN ai_agent_runtime_binding.active IS NULL;

COMMENT ON TABLE ai_agent_deployment IS NULL;
COMMENT ON COLUMN ai_agent_deployment.status IS NULL;

COMMENT ON TABLE ai_agent_composition_slot IS NULL;
COMMENT ON COLUMN ai_agent_composition_slot.slot_kind IS NULL;
COMMENT ON COLUMN ai_agent_composition_slot.target_module IS NULL;
COMMENT ON COLUMN ai_agent_composition_slot.target_ref IS NULL;

COMMENT ON TABLE ai_agent_audit_event IS NULL;
COMMENT ON TABLE ai_agent_outbox_event IS NULL;
COMMENT ON COLUMN ai_agent_outbox_event.status IS NULL;
