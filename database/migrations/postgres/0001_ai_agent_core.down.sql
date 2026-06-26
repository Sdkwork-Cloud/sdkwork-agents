-- Roll back ai_* composition-plane migration (destructive for removed domain tables).

DROP TABLE IF EXISTS ai_agent_outbox_event;
DROP TABLE IF EXISTS ai_agent_composition_slot;

ALTER TABLE IF EXISTS ai_agent_audit_event
    RENAME COLUMN agent_internal_id TO agent_business_id;
ALTER TABLE IF EXISTS ai_agent_audit_event RENAME TO a_agent_business_audit_event;
ALTER TABLE IF EXISTS ai_agent_deployment RENAME TO a_agent_deployment;
ALTER TABLE IF EXISTS ai_agent_runtime_binding RENAME TO a_agent_provider_binding;
ALTER TABLE IF EXISTS ai_agent RENAME TO a_agent_business;
ALTER TABLE IF EXISTS ai_app_registry RENAME TO agents_app_registry;

ALTER TABLE IF EXISTS agents_app_registry DROP COLUMN IF EXISTS default_agent_id;
ALTER TABLE IF EXISTS a_agent_business DROP COLUMN IF EXISTS manifest_schema_version;
