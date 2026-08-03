-- Add the full provider protocol payload (raw provider thread item JSON) to
-- agent session items so provider protocol data is preserved without loss
-- even when semantic parts are projected into item columns.
ALTER TABLE ai_agent_session_item
    ADD COLUMN provider_payload_json JSONB;
