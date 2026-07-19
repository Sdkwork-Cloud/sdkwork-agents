BEGIN;

ALTER TABLE ai_agent_outbox_event
    DROP CONSTRAINT uk_ai_agent_outbox_event_dedupe;
ALTER TABLE ai_agent_outbox_event
    ADD CONSTRAINT uk_ai_agent_outbox_event_dedupe_scope
    UNIQUE (tenant_id, organization_id, dedupe_key);

COMMIT;
