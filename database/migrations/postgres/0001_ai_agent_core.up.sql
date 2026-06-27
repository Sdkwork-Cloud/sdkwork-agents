-- Upgrade legacy agents managed-store tables to ai_* composition plane.
-- MCP, knowledge, and memory tables are dropped; their domains are owned by
-- sibling modules (sdkwork-mcp, sdkwork-knowledgebase, sdkwork-memory) and
-- referenced through ai_agent_composition_slot.

-- Rename legacy core tables to ai_* names only if the old table exists
-- and the new table does not (handles databases created before the ai_* baseline).
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'a_agent_business')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ai_agent') THEN
        ALTER TABLE a_agent_business RENAME TO ai_agent;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'a_agent_provider_binding')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ai_agent_runtime_binding') THEN
        ALTER TABLE a_agent_provider_binding RENAME TO ai_agent_runtime_binding;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'a_agent_deployment')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ai_agent_deployment') THEN
        ALTER TABLE a_agent_deployment RENAME TO ai_agent_deployment;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'a_agent_business_audit_event')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ai_agent_audit_event') THEN
        ALTER TABLE a_agent_business_audit_event RENAME TO ai_agent_audit_event;
        ALTER TABLE ai_agent_audit_event RENAME COLUMN agent_business_id TO agent_internal_id;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'agents_app_registry')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ai_app_registry') THEN
        ALTER TABLE agents_app_registry RENAME TO ai_app_registry;
    END IF;
END $$;

-- Add columns introduced in the composition plane (idempotent).
ALTER TABLE IF EXISTS ai_agent
    ADD COLUMN IF NOT EXISTS manifest_schema_version VARCHAR(32);

ALTER TABLE IF EXISTS ai_app_registry
    ADD COLUMN IF NOT EXISTS default_agent_id VARCHAR(128);

-- Rename audit column if not already renamed (idempotent).
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'ai_agent_audit_event' AND column_name = 'agent_business_id'
    ) THEN
        ALTER TABLE ai_agent_audit_event RENAME COLUMN agent_business_id TO agent_internal_id;
    END IF;
END $$;

-- Drop legacy knowledge tables (domain moved to sdkwork-knowledgebase).
DROP TABLE IF EXISTS a_agent_knowledge_sync_job;
DROP TABLE IF EXISTS a_agent_knowledge_binding;
DROP TABLE IF EXISTS a_agent_knowledge_index;
DROP TABLE IF EXISTS a_agent_knowledge_chunk;
DROP TABLE IF EXISTS a_agent_knowledge_document;
DROP TABLE IF EXISTS a_agent_knowledge_source;
DROP TABLE IF EXISTS a_agent_knowledge_base;

-- Drop legacy memory tables (domain moved to sdkwork-memory).
DROP TABLE IF EXISTS a_agent_memory_compaction_job;
DROP TABLE IF EXISTS a_agent_memory_access_event;
DROP TABLE IF EXISTS a_agent_memory_retrieval_index;
DROP TABLE IF EXISTS a_agent_memory_relation;
DROP TABLE IF EXISTS a_agent_memory_source;
DROP TABLE IF EXISTS a_agent_memory_record;
DROP TABLE IF EXISTS a_agent_memory_namespace;
DROP TABLE IF EXISTS a_agent_memory_binding;
DROP TABLE IF EXISTS a_agent_memory_profile;
DROP TABLE IF EXISTS a_agent_memory_store;

-- Drop legacy MCP marketplace table (domain moved to sdkwork-mcp).
DROP TABLE IF EXISTS a_agent_mcp_server;

-- Create composition slot table if not already created by baseline.
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
    CONSTRAINT uk_ai_agent_composition_slot_tenant_agent_slot UNIQUE (tenant_id, agent_id, slot_id)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_composition_slot_lookup
    ON ai_agent_composition_slot (tenant_id, agent_id, slot_kind, enabled, priority, slot_id);

-- Create outbox event table if not already created by baseline.
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
