BEGIN;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM ai_agent_project LIMIT 1)
       OR EXISTS (SELECT 1 FROM ai_agent_chat_turn LIMIT 1)
       OR EXISTS (SELECT 1 FROM ai_agent_message_drive_ref LIMIT 1)
       OR EXISTS (SELECT 1 FROM ai_agent_message_feedback LIMIT 1)
       OR EXISTS (SELECT 1 FROM ai_agent_resource_user_state LIMIT 1)
       OR EXISTS (SELECT 1 FROM ai_agent_project_member LIMIT 1)
       OR EXISTS (SELECT 1 FROM ai_agent_share_link LIMIT 1)
       OR EXISTS (SELECT 1 FROM ai_agent_outbox_event LIMIT 1) THEN
        RAISE EXCEPTION
            'agents chat 4.0 rollback refused: target tables contain commercial data';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM ai_agent_audit_event
        WHERE action IN (
            'project_created', 'project_updated', 'project_archived', 'project_deleted',
            'project_member_added', 'project_member_role_changed', 'project_member_removed',
            'project_composition_slot_created', 'project_composition_slot_updated',
            'project_composition_slot_deleted', 'session_renamed', 'session_moved',
            'session_deleted', 'turn_requested', 'turn_cancel_requested',
            'turn_completed', 'turn_failed', 'turn_cancelled', 'message_redacted',
            'message_feedback_changed', 'share_link_created', 'share_link_revoked',
            'share_link_expired'
        )
    ) THEN
        RAISE EXCEPTION
            'agents chat 4.0 rollback refused: target audit facts exist';
    END IF;
END;
$$;

ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT fk_ai_agent_audit_event_agent_restrict,
    DROP CONSTRAINT ck_ai_agent_audit_action_v4;
ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT ck_ai_agent_audit_action CHECK (
        action IN (
            'created', 'updated', 'deleted', 'restored', 'status_changed',
            'started', 'completed', 'failed', 'cancelled',
            'provider_binding_changed', 'composition_slot_created',
            'composition_slot_updated', 'composition_slot_deleted',
            'session_created', 'session_closed', 'session_archived',
            'message_created', 'message_failed'
        )
    ),
    ADD CONSTRAINT fk_ai_agent_audit_event_agent
        FOREIGN KEY (tenant_id, agent_internal_id)
        REFERENCES ai_agent (tenant_id, id) ON DELETE CASCADE;

DROP TABLE ai_agent_outbox_event;
DROP TABLE ai_agent_share_link;
DROP TABLE ai_agent_project_member;
DROP TABLE ai_agent_resource_user_state;
DROP TABLE ai_agent_message_feedback;
DROP TABLE ai_agent_message_drive_ref;

ALTER TABLE ai_agent_message DROP CONSTRAINT fk_ai_agent_message_turn;
DROP TABLE ai_agent_chat_turn;

ALTER TABLE ai_agent_message
    DROP CONSTRAINT fk_ai_agent_message_parent,
    DROP CONSTRAINT fk_ai_agent_message_session_restrict,
    DROP CONSTRAINT uk_ai_agent_message_scope_sequence,
    DROP CONSTRAINT uk_ai_agent_message_scope_session_id,
    DROP CONSTRAINT uk_ai_agent_message_scope_id,
    DROP CONSTRAINT ck_ai_agent_message_sender_user,
    DROP CONSTRAINT ck_ai_agent_message_sender_type;
DROP INDEX idx_ai_agent_message_retention;
DROP INDEX idx_ai_agent_message_scope_timeline;
ALTER TABLE ai_agent_message
    DROP COLUMN retention_until,
    DROP COLUMN deleted_by,
    DROP COLUMN deleted_at,
    DROP COLUMN created_by,
    DROP COLUMN turn_id,
    DROP COLUMN sender_user_id,
    DROP COLUMN sender_type,
    DROP COLUMN owner_user_id,
    DROP COLUMN organization_id,
    ADD CONSTRAINT fk_ai_agent_message_session
        FOREIGN KEY (tenant_id, session_id)
        REFERENCES ai_agent_session (tenant_id, session_id) ON DELETE CASCADE;

ALTER TABLE ai_agent_session
    DROP CONSTRAINT fk_ai_agent_session_agent_restrict,
    DROP CONSTRAINT fk_ai_agent_session_project,
    DROP CONSTRAINT uk_ai_agent_session_scope_id,
    DROP CONSTRAINT ck_ai_agent_session_last_sequence,
    DROP CONSTRAINT ck_ai_agent_session_title_source;
DROP INDEX idx_ai_agent_session_retention;
DROP INDEX idx_ai_agent_session_project_keyset;
DROP INDEX idx_ai_agent_session_owner_keyset;
DROP INDEX uk_ai_agent_session_create_idempotency;
ALTER TABLE ai_agent_session
    ALTER COLUMN title DROP NOT NULL,
    DROP COLUMN retention_until,
    DROP COLUMN deleted_by,
    DROP COLUMN deleted_at,
    DROP COLUMN archived_by,
    DROP COLUMN archived_at,
    DROP COLUMN updated_by,
    DROP COLUMN created_by,
    DROP COLUMN payload_hash,
    DROP COLUMN idempotency_key,
    DROP COLUMN last_message_sequence,
    DROP COLUMN title_source,
    DROP COLUMN project_id,
    ADD CONSTRAINT fk_ai_agent_session_agent
        FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE CASCADE;

DROP TABLE ai_agent_project_composition_slot;
DROP TABLE ai_agent_project;

COMMIT;
