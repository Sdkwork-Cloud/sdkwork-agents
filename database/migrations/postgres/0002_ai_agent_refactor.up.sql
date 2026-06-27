-- Migration: Upgrade to refactored schema v3
-- Transforms existing ai_* tables to professional design
-- Safe for production: idempotent, preserves existing data

-- ═══════════════════════════════════════════════════════
-- Step 1: Create helper functions
-- ═══════════════════════════════════════════════════════

CREATE OR REPLACE FUNCTION fnai_validate_capabilities_json(input TEXT)
RETURNS BOOLEAN
LANGUAGE plpgsql
IMMUTABLE
PARALLEL SAFE
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
        FROM jsonb_array_elements(payload) AS elem(value)
        WHERE NOT (
            jsonb_typeof(elem.value) = 'string'
            AND char_length(elem.value #>> '{}') <= 128
            AND (elem.value #>> '{}') ~ '^[a-z0-9_-]+(\.[a-z0-9_-]+)+$'
        )
    )
    AND (
        SELECT COUNT(*)
        FROM jsonb_array_elements(payload) AS elem(value)
    ) = (
        SELECT COUNT(DISTINCT elem.value #>> '{}')
        FROM jsonb_array_elements(payload) AS elem(value)
    );
EXCEPTION WHEN others THEN
    RETURN FALSE;
END;
$$;

CREATE OR REPLACE FUNCTION fnai_is_standard_id(input TEXT, prefix TEXT DEFAULT NULL)
RETURNS BOOLEAN
LANGUAGE plpgsql
IMMUTABLE
PARALLEL SAFE
AS $$
BEGIN
    IF input IS NULL OR char_length(input) = 0 OR char_length(input) > 128 THEN
        RETURN FALSE;
    END IF;
    IF input !~ '^[a-z0-9_-]+(\.[a-z0-9_-]+)*$' THEN
        RETURN FALSE;
    END IF;
    IF prefix IS NOT NULL AND input NOT LIKE prefix || '%' THEN
        RETURN FALSE;
    END IF;
    RETURN TRUE;
END;
$$;

-- ═══════════════════════════════════════════════════════
-- Step 2: Convert TEXT JSON columns to JSONB
-- ═══════════════════════════════════════════════════════

-- ai_agent table
ALTER TABLE ai_agent
    ALTER COLUMN manifest_json TYPE JSONB USING manifest_json::jsonb,
    ALTER COLUMN manifest_json SET DEFAULT '{}'::jsonb;

ALTER TABLE ai_agent
    ALTER COLUMN default_code_task_intent_json TYPE JSONB USING default_code_task_intent_json::jsonb;

ALTER TABLE ai_agent
    ALTER COLUMN tags_json TYPE JSONB USING tags_json::jsonb,
    ALTER COLUMN tags_json SET DEFAULT '[]'::jsonb;

-- ai_agent_runtime_binding table
ALTER TABLE ai_agent_runtime_binding
    ALTER COLUMN capabilities_json TYPE JSONB USING capabilities_json::jsonb,
    ALTER COLUMN capabilities_json SET DEFAULT '[]'::jsonb;

-- ai_agent_deployment table
ALTER TABLE ai_agent_deployment
    ALTER COLUMN capabilities_snapshot_json TYPE JSONB USING capabilities_snapshot_json::jsonb,
    ALTER COLUMN capabilities_snapshot_json SET DEFAULT '[]'::jsonb;

-- ai_agent_audit_event table
ALTER TABLE ai_agent_audit_event
    ALTER COLUMN payload_json TYPE JSONB USING payload_json::jsonb;

-- ═══════════════════════════════════════════════════════
-- Step 3: Add L2 tenant_entity compliance fields
-- Per DATABASE_SPEC.md §5.3 L2 tenant_entity tables require:
--   id, uuid, created_at, updated_at, version, status, tenant_id, organization_id
-- ai_app_registry lacked uuid/version; ai_agent_runtime_binding and
-- ai_agent_deployment lacked organization_id. ai_agent already has all fields.
-- ═══════════════════════════════════════════════════════

-- ai_app_registry: add uuid (NOT NULL via backfill) and version
ALTER TABLE ai_app_registry
    ADD COLUMN IF NOT EXISTS uuid VARCHAR(64),
    ADD COLUMN IF NOT EXISTS version BIGINT NOT NULL DEFAULT 0;

-- Backfill uuid for existing rows that lack one. Stable format keeps
-- the value deterministic across re-runs. Idempotent via WHERE.
UPDATE ai_app_registry
SET uuid = 'app-registry-' || lpad(id::text, 12, '0')
WHERE uuid IS NULL;

ALTER TABLE ai_app_registry
    ALTER COLUMN uuid SET NOT NULL;

-- Add unique constraint on uuid if not present
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'uk_ai_app_registry_uuid'
    ) THEN
        ALTER TABLE ai_app_registry
            ADD CONSTRAINT uk_ai_app_registry_uuid UNIQUE (uuid);
    END IF;
END $$;

-- ai_agent_runtime_binding and ai_agent_deployment: add organization_id
ALTER TABLE ai_agent_runtime_binding
    ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;

ALTER TABLE ai_agent_deployment
    ADD COLUMN IF NOT EXISTS organization_id BIGINT NOT NULL DEFAULT 0;

-- ═══════════════════════════════════════════════════════
-- Step 4: Add audit trail columns
-- ═══════════════════════════════════════════════════════

ALTER TABLE ai_agent
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(128),
    ADD COLUMN IF NOT EXISTS updated_by VARCHAR(128),
    ADD COLUMN IF NOT EXISTS deleted_by VARCHAR(128);

ALTER TABLE ai_agent_runtime_binding
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(128),
    ADD COLUMN IF NOT EXISTS updated_by VARCHAR(128);

ALTER TABLE ai_agent_deployment
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(128);

ALTER TABLE ai_agent_composition_slot
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(128),
    ADD COLUMN IF NOT EXISTS updated_by VARCHAR(128);

ALTER TABLE ai_app_registry
    ADD COLUMN IF NOT EXISTS created_by VARCHAR(128),
    ADD COLUMN IF NOT EXISTS updated_by VARCHAR(128);

-- ═══════════════════════════════════════════════════════
-- Step 5: Fix soft-delete unique constraints
-- ═══════════════════════════════════════════════════════

-- Drop old unique constraint and add partial unique index for soft-deletes
DO $$
BEGIN
    -- ai_agent: code uniqueness should only apply to non-deleted records
    IF EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'uk_ai_agent_tenant_code'
    ) THEN
        ALTER TABLE ai_agent DROP CONSTRAINT uk_ai_agent_tenant_code;
        CREATE UNIQUE INDEX uk_ai_agent_tenant_code
            ON ai_agent (tenant_id, code) WHERE deleted_at IS NULL;
    END IF;

    -- ai_agent_composition_slot: slot uniqueness should only apply to non-deleted records
    IF EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'uk_ai_agent_composition_slot_tenant_agent_slot'
    ) THEN
        ALTER TABLE ai_agent_composition_slot DROP CONSTRAINT uk_ai_agent_composition_slot_tenant_agent_slot;
        CREATE UNIQUE INDEX uk_ai_agent_composition_slot_tenant_agent_slot
            ON ai_agent_composition_slot (tenant_id, agent_id, slot_id) WHERE deleted_at IS NULL;
    END IF;
END $$;

-- ═══════════════════════════════════════════════════════
-- Step 6: Add missing indexes for query performance
-- ═══════════════════════════════════════════════════════

CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_visibility
    ON ai_agent (tenant_id, visibility, status)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_ai_agent_composition_slot_target
    ON ai_agent_composition_slot (tenant_id, target_module, target_ref)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_org_created
    ON ai_agent_audit_event (tenant_id, organization_id, created_at DESC);

-- ═══════════════════════════════════════════════════════
-- Step 7: Update CHECK constraints to use helper functions
-- ═══════════════════════════════════════════════════════

-- Add new CHECK constraints (existing ones remain for backward compatibility)
ALTER TABLE ai_agent
    ADD CONSTRAINT ck_ai_agent_implementation_provider_id_v2 CHECK (
        implementation_provider_id IS NULL
        OR fnai_is_standard_id(implementation_provider_id, 'provider.')
    );

ALTER TABLE ai_agent_runtime_binding
    ADD CONSTRAINT ck_ai_agent_runtime_binding_capabilities_v2 CHECK (
        fnai_validate_capabilities_json(capabilities_json::text)
    );

-- ═══════════════════════════════════════════════════════
-- Step 8: Add foreign key constraints (if not exists)
-- ═══════════════════════════════════════════════════════

DO $$
BEGIN
    -- Add FK for runtime binding → agent
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_ai_agent_runtime_binding_agent'
    ) THEN
        ALTER TABLE ai_agent_runtime_binding
            ADD CONSTRAINT fk_ai_agent_runtime_binding_agent
            FOREIGN KEY (tenant_id, agent_id)
            REFERENCES ai_agent(tenant_id, agent_id)
            ON DELETE CASCADE;
    END IF;

    -- Add FK for deployment → agent
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_ai_agent_deployment_agent'
    ) THEN
        ALTER TABLE ai_agent_deployment
            ADD CONSTRAINT fk_ai_agent_deployment_agent
            FOREIGN KEY (tenant_id, agent_id)
            REFERENCES ai_agent(tenant_id, agent_id)
            ON DELETE CASCADE;
    END IF;

    -- Add FK for composition slot → agent
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_ai_agent_composition_slot_agent'
    ) THEN
        ALTER TABLE ai_agent_composition_slot
            ADD CONSTRAINT fk_ai_agent_composition_slot_agent
            FOREIGN KEY (tenant_id, agent_id)
            REFERENCES ai_agent(tenant_id, agent_id)
            ON DELETE CASCADE;
    END IF;
END $$;

-- ═══════════════════════════════════════════════════════
-- Step 9: Create automatic updated_at trigger
-- ═══════════════════════════════════════════════════════

CREATE OR REPLACE FUNCTION fnai_update_updated_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;

-- Drop and recreate triggers if they exist
DROP TRIGGER IF EXISTS trg_ai_agent_updated_at ON ai_agent;
CREATE TRIGGER trg_ai_agent_updated_at
    BEFORE UPDATE ON ai_agent
    FOR EACH ROW
    EXECUTE FUNCTION fnai_update_updated_at();

DROP TRIGGER IF EXISTS trg_ai_agent_runtime_binding_updated_at ON ai_agent_runtime_binding;
CREATE TRIGGER trg_ai_agent_runtime_binding_updated_at
    BEFORE UPDATE ON ai_agent_runtime_binding
    FOR EACH ROW
    EXECUTE FUNCTION fnai_update_updated_at();

DROP TRIGGER IF EXISTS trg_ai_agent_composition_slot_updated_at ON ai_agent_composition_slot;
CREATE TRIGGER trg_ai_agent_composition_slot_updated_at
    BEFORE UPDATE ON ai_agent_composition_slot
    FOR EACH ROW
    EXECUTE FUNCTION fnai_update_updated_at();

-- ═══════════════════════════════════════════════════════
-- Step 10: Create view for active agents with bindings
-- ═══════════════════════════════════════════════════════

CREATE OR REPLACE VIEW v_ai_agent_active_binding AS
SELECT
    a.id AS agent_id,
    a.agent_id AS agent_code,
    a.tenant_id,
    a.organization_id,
    a.owner_user_id,
    a.code,
    a.display_name,
    a.status,
    a.visibility,
    rb.binding_id,
    rb.provider_id,
    rb.implementation_kind,
    rb.capabilities_json,
    a.updated_at
FROM ai_agent a
LEFT JOIN ai_agent_runtime_binding rb
    ON a.tenant_id = rb.tenant_id
    AND a.agent_id = rb.agent_id
    AND rb.active = TRUE
WHERE a.deleted_at IS NULL;

-- ═══════════════════════════════════════════════════════
-- Step 11: Enable Row-Level Security (optional, uncomment to enable)
-- ═══════════════════════════════════════════════════════

-- NOTE: RLS should be enabled after testing in staging environment
-- Uncomment the following lines to enable tenant isolation at database level:

-- ALTER TABLE ai_agent ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE ai_agent_runtime_binding ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE ai_agent_deployment ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE ai_agent_composition_slot ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE ai_agent_audit_event ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE ai_agent_outbox_event ENABLE ROW LEVEL SECURITY;

-- CREATE POLICY tenant_isolation_ai_agent ON ai_agent
--     USING (tenant_id = current_setting('app.current_tenant_id', true)::BIGINT);

-- ═══════════════════════════════════════════════════════
-- Step 12: Add table and column comments for documentation
-- ═══════════════════════════════════════════════════════

COMMENT ON TABLE ai_agent IS 'Agent business entity with identity, manifest snapshot, and lifecycle state';
COMMENT ON COLUMN ai_agent.manifest_json IS 'Full agent manifest as JSONB for structured queries';
COMMENT ON COLUMN ai_agent.status IS '0=draft, 1=active, 2=disabled, 3=archived, 4=deleted';
COMMENT ON COLUMN ai_agent.visibility IS '0=private, 1=internal, 2=public, 3=marketplace';

COMMENT ON TABLE ai_agent_runtime_binding IS 'Maps agents to provider runtime configurations with capability declarations';
COMMENT ON COLUMN ai_agent_runtime_binding.active IS 'Only one binding can be active per agent';

COMMENT ON TABLE ai_agent_deployment IS 'Immutable deployment history with provider configuration snapshots';
COMMENT ON COLUMN ai_agent_deployment.status IS '0=pending, 1=deployed, 2=rollback, 3=failed';

COMMENT ON TABLE ai_agent_composition_slot IS 'Binds agents to sibling module resources (memory, knowledge, skills, prompts, drive, mcp)';
COMMENT ON COLUMN ai_agent_composition_slot.slot_kind IS 'Resource type: memory, knowledge, skill, prompt, drive, tool, mcp';
COMMENT ON COLUMN ai_agent_composition_slot.target_module IS 'Sibling module owning the resource';
COMMENT ON COLUMN ai_agent_composition_slot.target_ref IS 'Reference ID to the resource in the target module';

COMMENT ON TABLE ai_agent_audit_event IS 'Immutable audit log for agent management operations';
COMMENT ON TABLE ai_agent_outbox_event IS 'Outbox pattern table for reliable cross-module event propagation';
COMMENT ON COLUMN ai_agent_outbox_event.status IS '0=pending, 1=published, 2=failed';

COMMENT ON VIEW v_ai_agent_active_binding IS 'Active agents with their current provider binding';
