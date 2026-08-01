-- sdkwork:migration
-- id: 0003_add_typed_agent_interaction_envelope
-- engine: postgres
-- module: agents
-- purpose: Persist a bounded provider-neutral typed Interaction request envelope and expanded categories.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: bounded-table-ddl
-- lock_timeout: 2s
-- statement_timeout: 2min
-- rewrite: Nullable JSONB column plus a check-constraint replacement; no row backfill.
-- replication_impact: Catalog DDL only.
-- backfill: Legacy approval and user-question rows retain a null request_json value.
-- observability: Migration history, PostgreSQL lock waits, schema readiness, and drift verification.
-- cancellation: Transaction rollback restores the prior table shape and kind constraint.
-- recovery: Deploy a reviewed forward-fix; do not encode typed requests in options_json.
-- contract_version: 7.2.0

BEGIN;

ALTER TABLE ai_agent_interaction
    ADD COLUMN IF NOT EXISTS request_json JSONB;

ALTER TABLE ai_agent_interaction
    DROP CONSTRAINT IF EXISTS ck_ai_agent_interaction_kind;

ALTER TABLE ai_agent_interaction
    ADD CONSTRAINT ck_ai_agent_interaction_kind
        CHECK (kind IN (0, 1, 2, 3));

ALTER TABLE ai_agent_interaction
    DROP CONSTRAINT IF EXISTS ck_ai_agent_interaction_request;

ALTER TABLE ai_agent_interaction
    ADD CONSTRAINT ck_ai_agent_interaction_request CHECK (
        request_json IS NULL
        OR (
            jsonb_typeof(request_json) = 'object'
            AND octet_length(request_json::text) <= 65536
        )
    );

COMMIT;
