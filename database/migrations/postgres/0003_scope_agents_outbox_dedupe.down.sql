BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT dedupe_key
        FROM ai_agent_outbox_event
        GROUP BY dedupe_key
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION
            'agents outbox dedupe rollback refused: cross-scope duplicate keys exist';
    END IF;
END;
$$;

ALTER TABLE ai_agent_outbox_event
    DROP CONSTRAINT uk_ai_agent_outbox_event_dedupe_scope;
ALTER TABLE ai_agent_outbox_event
    ADD CONSTRAINT uk_ai_agent_outbox_event_dedupe UNIQUE (dedupe_key);

COMMIT;
