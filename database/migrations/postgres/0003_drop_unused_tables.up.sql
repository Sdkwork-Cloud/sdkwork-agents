-- ═══════════════════════════════════════════════════════
-- Migration 0003: Drop unused tables (schema simplification v3)
-- ═══════════════════════════════════════════════════════
-- Removes 3 tables that were identified as dead code or over-designed:
--
-- 1. ai_app_registry — Dead code: schema + seed existed but zero business
--    reads in Rust source. Application identity is managed by IAM, not by
--    this table.
-- 2. ai_agent_deployment — Incomplete: only INSERT/LIST implemented, no
--    state machine transitions (promote/archive/rollback). Was effectively
--    a deployment log, not a lifecycle table. Removed to avoid confusion.
-- 3. ai_agent_outbox_event — Dead code: no publisher, no consumer, no
--    business transaction INSERTs. Outbox Pattern was designed (with monthly
--    partitioning + RLS) but never implemented. Removed as over-design.
--
-- Also removes 'deployment_created' from ai_agent_audit_event CHECK
-- constraint since deployment table no longer exists.
-- ═══════════════════════════════════════════════════════

-- Step 1: Drop ai_agent_outbox_event (no dependencies, safe to drop first)
DROP TABLE IF EXISTS ai_agent_outbox_event CASCADE;

-- Step 2: Drop ai_agent_deployment
-- Note: ai_agent_audit_event.action CHECK constraint references 'deployment_created'
-- We update the constraint before dropping the table to keep audit log consistent.
ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT IF EXISTS ck_ai_agent_audit_action;

ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT ck_ai_agent_audit_action CHECK (
        action IN (
            'created',
            'updated',
            'deleted',
            'restored',
            'status_changed',
            'started',
            'completed',
            'failed',
            'cancelled',
            'provider_binding_changed',
            'composition_slot_created',
            'composition_slot_updated',
            'composition_slot_deleted'
        )
    );

DROP TABLE IF EXISTS ai_agent_deployment CASCADE;

-- Step 3: Drop ai_app_registry
DROP TABLE IF EXISTS ai_app_registry CASCADE;
