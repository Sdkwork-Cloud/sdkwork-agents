BEGIN;

ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT ck_ai_agent_audit_action_v4;

ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT ck_ai_agent_audit_action_runtime CHECK (
        action IN (
            'created', 'updated', 'deleted', 'restored', 'status_changed',
            'started', 'completed', 'failed', 'cancelled', 'runtime_executed',
            'provider_binding_changed',
            'skill_created', 'skill_updated', 'skill_deleted', 'skill_restored',
            'composition_slot_created', 'composition_slot_updated',
            'composition_slot_deleted',
            'session_created', 'session_closed', 'session_archived',
            'message_created', 'message_failed',
            'interaction_created', 'interaction_resolved', 'interaction_rejected',
            'interaction_expired', 'interaction_cancelled',
            'task_created', 'task_completed', 'task_failed', 'task_cancelled',
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
