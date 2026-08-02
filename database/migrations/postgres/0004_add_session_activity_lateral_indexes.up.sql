-- 0004: session activity head projection indexes (7.3.0)
--
-- SQL_LIST_AGENT_SESSION_ACTIVITY_HEADS drives seven LEFT JOIN LATERAL
-- subqueries per row, each filtered by (tenant_id, organization_id,
-- session_id) and ordered by updated_at DESC, id DESC (or kind ASC,
-- updated_at DESC, id DESC). The existing created_at-based indexes do not
-- match those orderings, so large tenants page with index scans per row.
-- These composite indexes serve the lateral lookups directly.

CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_session_activity
    ON ai_agent_turn (tenant_id, organization_id, session_id, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_session_activity
    ON ai_agent_interaction (tenant_id, organization_id, session_id, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_session_kind_activity
    ON ai_agent_interaction (tenant_id, organization_id, session_id, kind ASC, updated_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_ai_agent_resource_user_state_session_activity
    ON ai_agent_resource_user_state (tenant_id, organization_id, session_id, updated_at DESC, id DESC);

COMMIT;
