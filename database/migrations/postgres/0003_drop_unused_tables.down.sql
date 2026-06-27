-- ═══════════════════════════════════════════════════════
-- Migration 0003 DOWN: Recreate dropped tables
-- ═══════════════════════════════════════════════════════
-- Recreates the 3 tables that were dropped in the up migration.
-- Data is NOT recovered — only table structures are restored.
-- ═══════════════════════════════════════════════════════

-- Step 1: Recreate ai_app_registry
CREATE TABLE IF NOT EXISTS ai_app_registry (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(64) NOT NULL,
    tenant_id BIGINT NOT NULL,
    application_key TEXT NOT NULL,
    kernel_slot_id TEXT NOT NULL,
    default_agent_id VARCHAR(128),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uk_ai_app_registry_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_app_registry_tenant_app UNIQUE (tenant_id, application_key)
);

CREATE INDEX IF NOT EXISTS idx_ai_app_registry_tenant_updated
    ON ai_app_registry (tenant_id, updated_at DESC);

-- Step 2: Recreate ai_agent_deployment
CREATE TABLE IF NOT EXISTS ai_agent_deployment (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
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
    CONSTRAINT ck_ai_agent_deployment_status CHECK (status IN (0, 1, 2, 3))
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_deployment_tenant_agent_created
    ON ai_agent_deployment (tenant_id, agent_id, created_at DESC, deployment_id ASC);

-- Step 3: Restore 'deployment_created' to audit CHECK constraint
ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT IF EXISTS ck_ai_agent_audit_action;

ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT ck_ai_agent_audit_action CHECK (
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
            'composition_slot_deleted'
        )
    );

-- Step 4: Recreate ai_agent_outbox_event
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
