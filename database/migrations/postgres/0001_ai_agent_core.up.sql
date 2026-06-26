-- Upgrade legacy agents managed-store tables to ai_* composition plane.
-- MCP table (a_agent_mcp_server) is intentionally unchanged.

ALTER TABLE IF EXISTS a_agent_business RENAME TO ai_agent;
ALTER TABLE IF EXISTS a_agent_provider_binding RENAME TO ai_agent_runtime_binding;
ALTER TABLE IF EXISTS a_agent_deployment RENAME TO ai_agent_deployment;
ALTER TABLE IF EXISTS a_agent_business_audit_event RENAME TO ai_agent_audit_event;
ALTER TABLE IF EXISTS agents_app_registry RENAME TO ai_app_registry;

ALTER TABLE IF EXISTS ai_agent
    ADD COLUMN IF NOT EXISTS manifest_schema_version VARCHAR(32);

ALTER TABLE IF EXISTS ai_app_registry
    ADD COLUMN IF NOT EXISTS default_agent_id VARCHAR(128);

ALTER TABLE IF EXISTS ai_agent_audit_event
    RENAME COLUMN agent_business_id TO agent_internal_id;

DROP TABLE IF EXISTS a_agent_knowledge_sync_job;
DROP TABLE IF EXISTS a_agent_knowledge_binding;
DROP TABLE IF EXISTS a_agent_knowledge_index;
DROP TABLE IF EXISTS a_agent_knowledge_chunk;
DROP TABLE IF EXISTS a_agent_knowledge_document;
DROP TABLE IF EXISTS a_agent_knowledge_source;
DROP TABLE IF EXISTS a_agent_knowledge_base;
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

CREATE INDEX IF NOT EXISTS idx_ai_agent_composition_slot_lookup
    ON ai_agent_composition_slot (tenant_id, agent_id, slot_kind, enabled, priority, slot_id);

CREATE INDEX IF NOT EXISTS idx_ai_agent_outbox_event_status_created
    ON ai_agent_outbox_event (tenant_id, status, created_at);
