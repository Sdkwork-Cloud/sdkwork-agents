-- sdkwork:migration
-- id: 0002_add_provider_session_directory
-- engine: postgres
-- module: agents
-- purpose: Persist provider-native session directory metadata without mutating provider-owned stores.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: bounded-table-ddl
-- lock_timeout: 2s
-- statement_timeout: 2min
-- rewrite: Nullable metadata columns plus constant-default booleans; PostgreSQL metadata-only defaults are expected.
-- replication_impact: Catalog DDL and one partial index build.
-- backfill: Existing runtime bindings retain neutral provider directory defaults until the next inventory synchronization.
-- observability: Migration history, PostgreSQL lock waits, schema readiness, and drift verification.
-- cancellation: Transaction rollback removes all added fields and the index.
-- recovery: Deploy a reviewed forward-fix and rerun provider inventory synchronization.
-- contract_version: 7.1.0

BEGIN;

ALTER TABLE ai_agent_session_runtime_binding
    ADD COLUMN IF NOT EXISTS provider_title VARCHAR(512),
    ADD COLUMN IF NOT EXISTS provider_title_source VARCHAR(64),
    ADD COLUMN IF NOT EXISTS provider_preview VARCHAR(4096),
    ADD COLUMN IF NOT EXISTS provider_created_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_recency_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS provider_pinned BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS provider_archived BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS provider_visible BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS provider_sort_key VARCHAR(512),
    ADD COLUMN IF NOT EXISTS provider_source VARCHAR(256);

CREATE INDEX IF NOT EXISTS idx_ai_agent_session_runtime_binding_provider_directory
    ON ai_agent_session_runtime_binding (
        tenant_id, organization_id, owner_user_id, provider_binding_id,
        provider_visible, provider_archived, provider_pinned DESC,
        provider_recency_at DESC, provider_sort_key, id DESC
    ) WHERE provider_session_id IS NOT NULL AND is_current = TRUE;

COMMIT;
