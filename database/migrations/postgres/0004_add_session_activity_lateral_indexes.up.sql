-- sdkwork:migration
-- id: 0004_add_session_activity_lateral_indexes
-- engine: postgres
-- module: agents
-- purpose: Serve the session activity head projection lateral lookups with composite (tenant, organization, session) ordering indexes.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: index-ddl
-- lock_timeout: 2s
-- statement_timeout: 2min
-- rewrite: Pure index DDL; no row rewrite.
-- replication_impact: Concurrent index builds on hot session tables; no table rewrite.
-- backfill: None; indexes are built over existing rows.
-- observability: Migration history, PostgreSQL lock waits, schema readiness, and drift verification.
-- contract_version: 7.3.0

BEGIN;

-- SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS drives seven LEFT JOIN LATERAL
-- subqueries per row, each filtered by (tenant_id, organization_id,
-- session_id) and ordered by updated_at DESC, id DESC (or kind ASC,
-- updated_at DESC, id DESC). The existing created_at-based indexes do not
-- match those orderings, so large tenants page with index scans per row.
-- These composite indexes serve the lateral lookups directly. The
-- session_user_state lateral is the exception: it filters by
-- (tenant_id, organization_id, user_id, resource_type, resource_id) because
-- session-scoped user state rows store the session id in resource_id.

CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_session_activity
    ON ai_agent_turn (tenant_id, organization_id, session_id, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_session_activity
    ON ai_agent_interaction (tenant_id, organization_id, session_id, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_session_kind_activity
    ON ai_agent_interaction (tenant_id, organization_id, session_id, kind ASC, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_resource_user_state_session_activity
    ON ai_agent_resource_user_state (tenant_id, organization_id, user_id, resource_type, resource_id, updated_at DESC, id DESC);

COMMIT;
