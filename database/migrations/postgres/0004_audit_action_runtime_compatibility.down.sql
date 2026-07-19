BEGIN;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM ai_agent_audit_event
        WHERE action IN (
            'runtime_executed',
            'skill_created', 'skill_updated', 'skill_deleted', 'skill_restored',
            'interaction_created', 'interaction_resolved', 'interaction_rejected',
            'interaction_expired', 'interaction_cancelled',
            'task_created', 'task_completed', 'task_failed', 'task_cancelled'
        )
    ) THEN
        RAISE EXCEPTION
            'rollback refused: audit rows use runtime actions not accepted by the previous constraint';
    END IF;
END $$;

ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT ck_ai_agent_audit_action_runtime;

ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT ck_ai_agent_audit_action_v4 CHECK (
        action IN (
            'created', 'updated', 'deleted', 'restored', 'status_changed',
            'started', 'completed', 'failed', 'cancelled',
            'provider_binding_changed', 'composition_slot_created',
            'composition_slot_updated', 'composition_slot_deleted',
            'session_created', 'session_closed', 'session_archived',
            'message_created', 'message_failed',
            'project_created', 'project_updated', 'project_archived', 'project_deleted',
            'project_member_added', 'project_member_role_changed', 'project_member_removed',
            'project_composition_slot_created', 'project_composition_slot_updated',
            'project_composition_slot_deleted', 'session_renamed', 'session_moved',
            'session_deleted', 'turn_requested', 'turn_cancel_requested',
            'turn_completed', 'turn_failed', 'turn_cancelled', 'message_redacted',
            'message_feedback_changed', 'share_link_created', 'share_link_revoked',
            'share_link_expired'
        )
    );

COMMIT;
