BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM ai_agent_audit_event
        WHERE aggregate_type <> 'agent'
           OR agent_internal_id IS NULL
           OR agent_id IS NULL
    ) THEN
        RAISE EXCEPTION
            'rollback refused: non-agent aggregate audit history cannot fit the previous schema';
    END IF;
END $$;

DROP INDEX idx_ai_agent_audit_aggregate_created;

ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT fk_ai_agent_audit_event_agent_optional,
    DROP CONSTRAINT ck_ai_agent_audit_aggregate_agent,
    ALTER COLUMN agent_internal_id SET NOT NULL,
    ALTER COLUMN agent_id SET NOT NULL,
    ADD CONSTRAINT fk_ai_agent_audit_event_agent_restrict
        FOREIGN KEY (tenant_id, agent_internal_id)
        REFERENCES ai_agent (tenant_id, id) ON DELETE RESTRICT,
    DROP COLUMN aggregate_id,
    DROP COLUMN aggregate_type;

COMMIT;
