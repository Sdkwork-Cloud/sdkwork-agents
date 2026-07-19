BEGIN;

ALTER TABLE ai_agent_audit_event
    ADD COLUMN aggregate_type VARCHAR(64),
    ADD COLUMN aggregate_id VARCHAR(128);

UPDATE ai_agent_audit_event
SET aggregate_type = 'agent',
    aggregate_id = agent_id;

ALTER TABLE ai_agent_audit_event
    ALTER COLUMN aggregate_type SET NOT NULL,
    ALTER COLUMN aggregate_id SET NOT NULL,
    DROP CONSTRAINT fk_ai_agent_audit_event_agent_restrict,
    ALTER COLUMN agent_internal_id DROP NOT NULL,
    ALTER COLUMN agent_id DROP NOT NULL,
    ADD CONSTRAINT ck_ai_agent_audit_aggregate_agent CHECK (
        (aggregate_type = 'agent' AND agent_internal_id IS NOT NULL AND agent_id IS NOT NULL)
        OR (aggregate_type <> 'agent' AND agent_internal_id IS NULL AND agent_id IS NULL)
    ),
    ADD CONSTRAINT fk_ai_agent_audit_event_agent_optional
        FOREIGN KEY (tenant_id, agent_internal_id)
        REFERENCES ai_agent (tenant_id, id) ON DELETE RESTRICT;

CREATE INDEX idx_ai_agent_audit_aggregate_created
    ON ai_agent_audit_event (
        tenant_id, organization_id, aggregate_type, aggregate_id, created_at DESC, id DESC
    );

COMMIT;
