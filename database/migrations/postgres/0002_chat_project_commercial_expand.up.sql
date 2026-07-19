BEGIN;

-- Repair pre-3.1.0 databases whose baseline predates the tenant-scoped internal key.
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_tenant_id
    ON ai_agent (tenant_id, id);

CREATE TABLE ai_agent_project (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    project_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    visibility SMALLINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    drive_access_mode SMALLINT NOT NULL DEFAULT 0,
    default_agent_id VARCHAR(128),
    default_model_id VARCHAR(128),
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    archived_at TIMESTAMPTZ,
    archived_by BIGINT,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_project_scope_id
        UNIQUE (tenant_id, organization_id, project_id),
    CONSTRAINT ck_ai_agent_project_visibility CHECK (visibility IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_drive_access CHECK (drive_access_mode IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_shared_drive CHECK (
        visibility <> 2 OR drive_access_mode <> 1
    ),
    CONSTRAINT ck_ai_agent_project_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_project_default_agent
        FOREIGN KEY (tenant_id, default_agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT
);

CREATE INDEX idx_ai_agent_project_owner_list
    ON ai_agent_project (
        tenant_id, organization_id, owner_user_id, status, updated_at DESC, id DESC
    );
CREATE INDEX idx_ai_agent_project_org_list
    ON ai_agent_project (
        tenant_id, organization_id, visibility, status, updated_at DESC, id DESC
    );
CREATE INDEX idx_ai_agent_project_retention
    ON ai_agent_project (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE ai_agent_project_composition_slot (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    project_id VARCHAR(128) NOT NULL,
    slot_id VARCHAR(128) NOT NULL,
    slot_kind VARCHAR(64) NOT NULL,
    target_module VARCHAR(64) NOT NULL,
    target_ref VARCHAR(256) NOT NULL,
    target_version_ref VARCHAR(128),
    priority INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    policy_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by BIGINT NOT NULL,
    updated_by BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    deleted_by BIGINT,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_project_slot_scope
        UNIQUE (tenant_id, organization_id, project_id, slot_id),
    CONSTRAINT ck_ai_agent_project_slot_kind CHECK (
        slot_kind IN ('prompt', 'memory', 'knowledge', 'skill', 'mcp', 'drive', 'tool')
    ),
    CONSTRAINT ck_ai_agent_project_slot_module CHECK (
        target_module IN (
            'prompts', 'memory', 'knowledgebase', 'skills', 'mcp', 'drive', 'tools'
        )
    ),
    CONSTRAINT ck_ai_agent_project_slot_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_project_slot_project
        FOREIGN KEY (tenant_id, organization_id, project_id)
        REFERENCES ai_agent_project (tenant_id, organization_id, project_id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_ai_agent_project_slot_lookup
    ON ai_agent_project_composition_slot (
        tenant_id, organization_id, project_id, slot_kind, enabled, priority, id
    ) WHERE deleted_at IS NULL;

ALTER TABLE ai_agent_session
    ADD COLUMN project_id VARCHAR(128),
    ADD COLUMN title_source SMALLINT NOT NULL DEFAULT 0,
    ADD COLUMN last_message_sequence BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN idempotency_key VARCHAR(256),
    ADD COLUMN payload_hash VARCHAR(128),
    ADD COLUMN created_by BIGINT,
    ADD COLUMN updated_by BIGINT,
    ADD COLUMN archived_at TIMESTAMPTZ,
    ADD COLUMN archived_by BIGINT,
    ADD COLUMN deleted_at TIMESTAMPTZ,
    ADD COLUMN deleted_by BIGINT,
    ADD COLUMN retention_until TIMESTAMPTZ;

UPDATE ai_agent_session
SET title = COALESCE(NULLIF(BTRIM(title), ''), 'New chat'),
    created_by = owner_user_id,
    updated_by = owner_user_id;

UPDATE ai_agent_session AS session
SET last_message_sequence = message_state.max_sequence
FROM (
    SELECT tenant_id, session_id, MAX(sequence) AS max_sequence
    FROM ai_agent_message
    GROUP BY tenant_id, session_id
) AS message_state
WHERE session.tenant_id = message_state.tenant_id
  AND session.session_id = message_state.session_id;

ALTER TABLE ai_agent_session
    ALTER COLUMN title SET NOT NULL,
    ALTER COLUMN created_by SET NOT NULL,
    ALTER COLUMN updated_by SET NOT NULL,
    ADD CONSTRAINT ck_ai_agent_session_title_source CHECK (title_source IN (0, 1, 2)),
    ADD CONSTRAINT ck_ai_agent_session_last_sequence CHECK (last_message_sequence >= 0),
    ADD CONSTRAINT uk_ai_agent_session_scope_id
        UNIQUE (tenant_id, organization_id, session_id),
    ADD CONSTRAINT fk_ai_agent_session_project
        FOREIGN KEY (tenant_id, organization_id, project_id)
        REFERENCES ai_agent_project (tenant_id, organization_id, project_id)
        ON DELETE RESTRICT;

CREATE UNIQUE INDEX uk_ai_agent_session_create_idempotency
    ON ai_agent_session (tenant_id, organization_id, owner_user_id, idempotency_key)
    WHERE idempotency_key IS NOT NULL;
CREATE INDEX idx_ai_agent_session_owner_keyset
    ON ai_agent_session (
        tenant_id, organization_id, owner_user_id, status, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX idx_ai_agent_session_project_keyset
    ON ai_agent_session (
        tenant_id, organization_id, project_id, status, updated_at DESC, id DESC
    ) WHERE project_id IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX idx_ai_agent_session_retention
    ON ai_agent_session (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

ALTER TABLE ai_agent_session DROP CONSTRAINT fk_ai_agent_session_agent;
ALTER TABLE ai_agent_session
    ADD CONSTRAINT fk_ai_agent_session_agent_restrict
    FOREIGN KEY (tenant_id, agent_id)
    REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT;

ALTER TABLE ai_agent_message
    ADD COLUMN organization_id BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN owner_user_id BIGINT,
    ADD COLUMN sender_type SMALLINT,
    ADD COLUMN sender_user_id BIGINT,
    ADD COLUMN turn_id VARCHAR(128),
    ADD COLUMN created_by BIGINT,
    ADD COLUMN deleted_at TIMESTAMPTZ,
    ADD COLUMN deleted_by BIGINT,
    ADD COLUMN retention_until TIMESTAMPTZ;

UPDATE ai_agent_message AS message
SET organization_id = session.organization_id,
    owner_user_id = session.owner_user_id,
    sender_type = message.role,
    sender_user_id = CASE WHEN message.role = 0 THEN session.owner_user_id ELSE NULL END,
    created_by = session.owner_user_id
FROM ai_agent_session AS session
WHERE message.tenant_id = session.tenant_id
  AND message.session_id = session.session_id;

ALTER TABLE ai_agent_message
    ALTER COLUMN owner_user_id SET NOT NULL,
    ALTER COLUMN sender_type SET NOT NULL,
    ALTER COLUMN created_by SET NOT NULL,
    ADD CONSTRAINT ck_ai_agent_message_sender_type CHECK (sender_type IN (0, 1, 2, 3)),
    ADD CONSTRAINT ck_ai_agent_message_sender_user CHECK (
        (sender_type = 0 AND sender_user_id IS NOT NULL)
        OR (sender_type <> 0 AND sender_user_id IS NULL)
    ),
    ADD CONSTRAINT uk_ai_agent_message_scope_id
        UNIQUE (tenant_id, organization_id, message_id),
    ADD CONSTRAINT uk_ai_agent_message_scope_session_id
        UNIQUE (tenant_id, organization_id, session_id, message_id),
    ADD CONSTRAINT uk_ai_agent_message_scope_sequence
        UNIQUE (tenant_id, organization_id, session_id, sequence);

ALTER TABLE ai_agent_message DROP CONSTRAINT fk_ai_agent_message_session;
ALTER TABLE ai_agent_message
    ADD CONSTRAINT fk_ai_agent_message_session_restrict
    FOREIGN KEY (tenant_id, organization_id, session_id)
    REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
    ON DELETE RESTRICT,
    ADD CONSTRAINT fk_ai_agent_message_parent
    FOREIGN KEY (tenant_id, organization_id, session_id, parent_message_id)
    REFERENCES ai_agent_message (tenant_id, organization_id, session_id, message_id)
    ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX idx_ai_agent_message_scope_timeline
    ON ai_agent_message (
        tenant_id, organization_id, session_id, sequence ASC, id ASC
    ) WHERE deleted_at IS NULL;
CREATE INDEX idx_ai_agent_message_retention
    ON ai_agent_message (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE ai_agent_chat_turn (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    turn_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    client_request_id VARCHAR(128),
    idempotency_key VARCHAR(256) NOT NULL,
    payload_hash VARCHAR(128) NOT NULL,
    request_message_id VARCHAR(128) NOT NULL,
    response_message_id VARCHAR(128),
    mode SMALLINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    requested_model_id VARCHAR(128),
    provider_binding_id VARCHAR(128),
    model_id VARCHAR(128),
    provider_id VARCHAR(128),
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cached_tokens BIGINT NOT NULL DEFAULT 0,
    finish_reason VARCHAR(64),
    error_code VARCHAR(128),
    error_detail VARCHAR(2048),
    trace_id VARCHAR(128),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancel_requested_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_chat_turn_scope_id
        UNIQUE (tenant_id, organization_id, turn_id),
    CONSTRAINT uk_ai_agent_chat_turn_idempotency
        UNIQUE (tenant_id, organization_id, owner_user_id, idempotency_key),
    CONSTRAINT ck_ai_agent_chat_turn_mode CHECK (mode IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_chat_turn_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_chat_turn_tokens CHECK (
        input_tokens >= 0 AND output_tokens >= 0 AND cached_tokens >= 0
    ),
    CONSTRAINT ck_ai_agent_chat_turn_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_chat_turn_session
        FOREIGN KEY (tenant_id, organization_id, session_id)
        REFERENCES ai_agent_session (tenant_id, organization_id, session_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_chat_turn_agent
        FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id) ON DELETE RESTRICT
);

CREATE UNIQUE INDEX uk_ai_agent_chat_turn_client_request
    ON ai_agent_chat_turn (
        tenant_id, organization_id, owner_user_id, client_request_id
    ) WHERE client_request_id IS NOT NULL;
CREATE INDEX idx_ai_agent_chat_turn_session_timeline
    ON ai_agent_chat_turn (
        tenant_id, organization_id, session_id, created_at ASC, id ASC
    );
CREATE INDEX idx_ai_agent_chat_turn_reconcile
    ON ai_agent_chat_turn (
        tenant_id, organization_id, status, updated_at ASC, id ASC
    ) WHERE status IN (0, 1);
CREATE INDEX idx_ai_agent_chat_turn_trace
    ON ai_agent_chat_turn (tenant_id, organization_id, trace_id)
    WHERE trace_id IS NOT NULL;

ALTER TABLE ai_agent_message
    ADD CONSTRAINT fk_ai_agent_message_turn
    FOREIGN KEY (tenant_id, organization_id, turn_id)
    REFERENCES ai_agent_chat_turn (tenant_id, organization_id, turn_id)
    ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE ai_agent_message_drive_ref (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    message_id VARCHAR(128) NOT NULL,
    media_role VARCHAR(64) NOT NULL,
    drive_space_id VARCHAR(128) NOT NULL,
    drive_node_id VARCHAR(128) NOT NULL,
    drive_uri VARCHAR(512) NOT NULL,
    media_resource_id VARCHAR(128),
    object_blob_id VARCHAR(128),
    resource_snapshot JSONB NOT NULL DEFAULT '{}'::jsonb,
    resource_hash VARCHAR(128) NOT NULL,
    alt_text VARCHAR(512),
    sort_order INTEGER NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_message_drive_ref_resource
        UNIQUE (
            tenant_id, organization_id, message_id, drive_node_id, media_role
        ),
    CONSTRAINT ck_ai_agent_message_drive_ref_role CHECK (
        media_role IN ('attachment', 'image', 'voice', 'generated_output', 'artifact')
    ),
    CONSTRAINT ck_ai_agent_message_drive_ref_order CHECK (sort_order >= 0),
    CONSTRAINT ck_ai_agent_message_drive_ref_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_message_drive_ref_uri CHECK (
        drive_uri = 'drive://spaces/' || drive_space_id || '/nodes/' || drive_node_id
    ),
    CONSTRAINT fk_ai_agent_message_drive_ref_message
        FOREIGN KEY (tenant_id, organization_id, message_id)
        REFERENCES ai_agent_message (tenant_id, organization_id, message_id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_ai_agent_message_drive_ref_list
    ON ai_agent_message_drive_ref (
        tenant_id, organization_id, message_id, status, sort_order, id
    );
CREATE INDEX idx_ai_agent_message_drive_ref_drive
    ON ai_agent_message_drive_ref (
        tenant_id, organization_id, drive_space_id, drive_node_id
    );

CREATE TABLE ai_agent_message_feedback (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    message_id VARCHAR(128) NOT NULL,
    user_id BIGINT NOT NULL,
    rating SMALLINT NOT NULL,
    reason_code VARCHAR(64),
    comment VARCHAR(1024),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_message_feedback_user
        UNIQUE (tenant_id, organization_id, message_id, user_id),
    CONSTRAINT ck_ai_agent_message_feedback_rating CHECK (rating IN (1, -1)),
    CONSTRAINT ck_ai_agent_message_feedback_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_message_feedback_message
        FOREIGN KEY (tenant_id, organization_id, message_id)
        REFERENCES ai_agent_message (tenant_id, organization_id, message_id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_ai_agent_message_feedback_analytics
    ON ai_agent_message_feedback (
        tenant_id, organization_id, rating, created_at DESC, id DESC
    ) WHERE deleted_at IS NULL;

CREATE TABLE ai_agent_resource_user_state (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    user_id BIGINT NOT NULL,
    resource_type SMALLINT NOT NULL,
    resource_id VARCHAR(128) NOT NULL,
    pinned_at TIMESTAMPTZ,
    hidden_at TIMESTAMPTZ,
    last_opened_at TIMESTAMPTZ,
    last_read_message_sequence BIGINT,
    custom_title VARCHAR(512),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uk_ai_agent_resource_user_state_resource
        UNIQUE (
            tenant_id, organization_id, user_id, resource_type, resource_id
        ),
    CONSTRAINT ck_ai_agent_resource_user_state_type CHECK (resource_type IN (0, 1)),
    CONSTRAINT ck_ai_agent_resource_user_state_sequence CHECK (
        last_read_message_sequence IS NULL OR last_read_message_sequence >= 0
    ),
    CONSTRAINT ck_ai_agent_resource_user_state_version CHECK (version >= 0)
);

CREATE INDEX idx_ai_agent_resource_user_state_recent
    ON ai_agent_resource_user_state (
        tenant_id, organization_id, user_id, resource_type,
        pinned_at DESC, last_opened_at DESC, id DESC
    ) WHERE hidden_at IS NULL;

CREATE TABLE ai_agent_project_member (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    project_id VARCHAR(128) NOT NULL,
    member_user_id BIGINT NOT NULL,
    role SMALLINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    invited_by BIGINT,
    joined_at TIMESTAMPTZ,
    removed_at TIMESTAMPTZ,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_project_member_user
        UNIQUE (tenant_id, organization_id, project_id, member_user_id),
    CONSTRAINT ck_ai_agent_project_member_role CHECK (role IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_project_member_status CHECK (status IN (0, 1, 2, 3)),
    CONSTRAINT ck_ai_agent_project_member_version CHECK (version >= 0),
    CONSTRAINT fk_ai_agent_project_member_project
        FOREIGN KEY (tenant_id, organization_id, project_id)
        REFERENCES ai_agent_project (tenant_id, organization_id, project_id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_ai_agent_project_member_user_list
    ON ai_agent_project_member (
        tenant_id, organization_id, member_user_id, status, updated_at DESC, id DESC
    );
CREATE INDEX idx_ai_agent_project_member_project_list
    ON ai_agent_project_member (
        tenant_id, organization_id, project_id, status, updated_at DESC, id DESC
    );

CREATE TABLE ai_agent_share_link (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    link_id VARCHAR(128) NOT NULL,
    target_type SMALLINT NOT NULL,
    target_id VARCHAR(128) NOT NULL,
    permission SMALLINT NOT NULL,
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    token_prefix VARCHAR(16) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    created_by BIGINT NOT NULL,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    revoked_by BIGINT,
    max_uses BIGINT,
    use_count BIGINT NOT NULL DEFAULT 0,
    last_used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_share_link_scope_id
        UNIQUE (tenant_id, organization_id, link_id),
    CONSTRAINT ck_ai_agent_share_link_target CHECK (target_type IN (0, 1)),
    CONSTRAINT ck_ai_agent_share_link_permission CHECK (permission IN (0, 1)),
    CONSTRAINT ck_ai_agent_share_link_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_share_link_usage CHECK (
        use_count >= 0 AND (max_uses IS NULL OR (max_uses > 0 AND use_count <= max_uses))
    )
);

CREATE INDEX idx_ai_agent_share_link_target
    ON ai_agent_share_link (
        tenant_id, organization_id, target_type, target_id, status, expires_at, id
    );

CREATE TABLE ai_agent_outbox_event (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    event_id VARCHAR(128) NOT NULL,
    aggregate_type VARCHAR(64) NOT NULL,
    aggregate_id VARCHAR(128) NOT NULL,
    aggregate_version BIGINT NOT NULL,
    event_type VARCHAR(128) NOT NULL,
    payload_json JSONB NOT NULL,
    headers_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    dedupe_key VARCHAR(256) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 10,
    available_at TIMESTAMPTZ NOT NULL,
    lease_owner VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    last_error_code VARCHAR(128),
    last_error_detail VARCHAR(2048),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_outbox_event_scope_id
        UNIQUE (tenant_id, organization_id, event_id),
    CONSTRAINT uk_ai_agent_outbox_event_dedupe UNIQUE (dedupe_key),
    CONSTRAINT ck_ai_agent_outbox_event_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_outbox_event_attempts CHECK (
        attempt_count >= 0 AND max_attempts > 0 AND attempt_count <= max_attempts
    )
);

CREATE INDEX idx_ai_agent_outbox_event_worker
    ON ai_agent_outbox_event (
        status, available_at, lease_expires_at, id
    ) WHERE status IN (0, 1, 3);
CREATE INDEX idx_ai_agent_outbox_event_retention
    ON ai_agent_outbox_event (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

ALTER TABLE ai_agent_audit_event DROP CONSTRAINT ck_ai_agent_audit_action;
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

ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT IF EXISTS fk_ai_agent_audit_event_agent;
ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT fk_ai_agent_audit_event_agent_restrict
    FOREIGN KEY (tenant_id, agent_internal_id)
    REFERENCES ai_agent (tenant_id, id) ON DELETE RESTRICT;

COMMIT;
