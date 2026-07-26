-- sdkwork:migration
-- id: 0001_add_agent_workspaces
-- engine: postgres
-- module: agents
-- purpose: Add user-owned Workspaces and assign every Agent Project to one Workspace
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: share-row-exclusive on Workspace and Project during the bounded backfill
-- lock_timeout: 5s
-- statement_timeout: 120s
-- contract_version: 6.0.0
-- rewrite_expectation: metadata-only columns followed by a bounded indexed-row update
-- wal_impact: proportional to the number of existing ai_agent_project rows
-- backfill_plan: one stable default Workspace per distinct Project owner, then Project workspace_id update
-- observability: verify zero null workspace_id rows before constraints become active
-- cancellation_point: before ALTER COLUMN workspace_id SET NOT NULL
-- recovery_command: rerun this idempotent migration after resolving the reported conflict

SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '120s';

CREATE TABLE IF NOT EXISTS ai_agent_workspace (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    workspace_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    status SMALLINT NOT NULL DEFAULT 0,
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
    CONSTRAINT uk_ai_agent_workspace_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_workspace_scope UNIQUE (
        tenant_id, organization_id, workspace_id
    ),
    CONSTRAINT ck_ai_agent_workspace_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_workspace_version CHECK (version >= 0)
);

ALTER TABLE ai_agent_audit_event
    DROP CONSTRAINT IF EXISTS ck_ai_agent_audit_aggregate_type,
    DROP CONSTRAINT IF EXISTS ck_ai_agent_audit_action;

ALTER TABLE ai_agent_audit_event
    ADD CONSTRAINT ck_ai_agent_audit_aggregate_type CHECK (
        aggregate_type IN (
            'agent', 'runtime_binding', 'composition_slot', 'workspace', 'project',
            'project_member', 'session', 'turn', 'session_item', 'item_feedback',
            'interaction', 'checkpoint', 'task', 'share_link'
        )
    ),
    ADD CONSTRAINT ck_ai_agent_audit_action CHECK (
        action IN (
            'created', 'updated', 'deleted', 'restored', 'status_changed',
            'started', 'completed', 'failed', 'cancelled', 'runtime_executed',
            'provider_binding_changed', 'composition_slot_created',
            'composition_slot_updated', 'composition_slot_deleted',
            'workspace_created', 'workspace_updated', 'workspace_archived', 'workspace_deleted',
            'project_created', 'project_updated', 'project_archived', 'project_deleted',
            'project_member_added', 'project_member_role_changed', 'project_member_removed',
            'project_composition_slot_created', 'project_composition_slot_updated',
            'project_composition_slot_deleted', 'session_created', 'session_closed',
            'session_renamed', 'session_moved', 'session_archived', 'session_deleted',
            'session_runtime_binding_created', 'session_runtime_binding_updated',
            'session_runtime_binding_activated', 'session_runtime_binding_deactivated',
            'turn_requested', 'turn_cancel_requested', 'turn_completed', 'turn_failed',
            'turn_cancelled', 'session_item_created', 'session_item_failed',
            'session_item_redacted', 'item_feedback_changed', 'interaction_created',
            'interaction_claimed', 'interaction_resolved', 'interaction_rejected',
            'interaction_expired', 'interaction_cancelled', 'session_checkpoint_created',
            'session_checkpoint_restored', 'session_checkpoint_invalidated', 'task_created',
            'task_completed', 'task_failed', 'task_cancelled', 'share_link_created',
            'share_link_revoked', 'share_link_expired'
        )
    );

LOCK TABLE ai_agent_workspace IN SHARE ROW EXCLUSIVE MODE;
LOCK TABLE ai_agent_project IN SHARE ROW EXCLUSIVE MODE;

ALTER TABLE ai_agent_project
    ADD COLUMN IF NOT EXISTS workspace_id VARCHAR(128),
    ADD COLUMN IF NOT EXISTS import_source_kind VARCHAR(64),
    ADD COLUMN IF NOT EXISTS import_source_ref VARCHAR(512),
    ADD COLUMN IF NOT EXISTS drive_space_id VARCHAR(128),
    ADD COLUMN IF NOT EXISTS drive_root_entry_id VARCHAR(128),
    ADD COLUMN IF NOT EXISTS drive_logical_path VARCHAR(1024);

WITH owner_defaults AS (
    SELECT
        tenant_id,
        organization_id,
        owner_user_id,
        'workspace.default.' || owner_user_id::text AS workspace_id,
        MIN(created_at) AS created_at,
        MAX(updated_at) AS updated_at,
        ROW_NUMBER() OVER (
            ORDER BY tenant_id, organization_id, owner_user_id
        ) AS sequence
    FROM ai_agent_project
    GROUP BY tenant_id, organization_id, owner_user_id
), id_base AS (
    SELECT COALESCE(MAX(id), 0) AS max_id
    FROM ai_agent_workspace
)
INSERT INTO ai_agent_workspace (
    id,
    uuid,
    tenant_id,
    organization_id,
    workspace_id,
    owner_user_id,
    name,
    description,
    is_default,
    status,
    created_by,
    updated_by,
    version,
    created_at,
    updated_at,
    archived_at,
    archived_by,
    deleted_at,
    deleted_by,
    retention_until
)
SELECT
    id_base.max_id + owner_defaults.sequence,
    md5(
        'sdkwork.agents.workspace.v1:'
        || owner_defaults.tenant_id::text || ':'
        || owner_defaults.organization_id::text || ':'
        || owner_defaults.workspace_id
    ),
    owner_defaults.tenant_id,
    owner_defaults.organization_id,
    owner_defaults.workspace_id,
    owner_defaults.owner_user_id,
    'Workspace',
    NULL,
    TRUE,
    0,
    owner_defaults.owner_user_id,
    owner_defaults.owner_user_id,
    0,
    owner_defaults.created_at,
    owner_defaults.updated_at,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL
FROM owner_defaults
CROSS JOIN id_base
ON CONFLICT (tenant_id, organization_id, workspace_id) DO NOTHING;

UPDATE ai_agent_project
SET workspace_id = 'workspace.default.' || owner_user_id::text
WHERE workspace_id IS NULL;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM ai_agent_project WHERE workspace_id IS NULL) THEN
        RAISE EXCEPTION 'ai_agent_project workspace backfill is incomplete';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM ai_agent_project AS project
        LEFT JOIN ai_agent_workspace AS workspace
          ON workspace.tenant_id = project.tenant_id
         AND workspace.organization_id = project.organization_id
         AND workspace.workspace_id = project.workspace_id
        WHERE workspace.id IS NULL
    ) THEN
        RAISE EXCEPTION 'ai_agent_project contains a workspace_id without an owning Workspace';
    END IF;
END $$;

ALTER TABLE ai_agent_project
    ALTER COLUMN workspace_id SET NOT NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'ck_ai_agent_project_import_source'
    ) THEN
        ALTER TABLE ai_agent_project
            ADD CONSTRAINT ck_ai_agent_project_import_source CHECK (
                (import_source_kind IS NULL AND import_source_ref IS NULL)
                OR (import_source_kind IS NOT NULL AND import_source_ref IS NOT NULL)
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'ck_ai_agent_project_drive_source'
    ) THEN
        ALTER TABLE ai_agent_project
            ADD CONSTRAINT ck_ai_agent_project_drive_source CHECK (
                import_source_kind IS DISTINCT FROM 'drive_sandbox'
                OR (drive_space_id IS NOT NULL AND drive_root_entry_id IS NOT NULL)
            );
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_ai_agent_project_workspace'
    ) THEN
        ALTER TABLE ai_agent_project
            ADD CONSTRAINT fk_ai_agent_project_workspace FOREIGN KEY (
                tenant_id, organization_id, workspace_id
            ) REFERENCES ai_agent_workspace (tenant_id, organization_id, workspace_id)
                ON DELETE RESTRICT;
    END IF;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_workspace_active_default
    ON ai_agent_workspace (tenant_id, organization_id, owner_user_id)
    WHERE is_default = TRUE AND status = 0 AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_workspace_owner_list
    ON ai_agent_workspace (
        tenant_id, organization_id, owner_user_id, status,
        is_default DESC, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_workspace_retention
    ON ai_agent_workspace (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_ai_agent_project_workspace_list
    ON ai_agent_project (
        tenant_id, organization_id, workspace_id, status, updated_at DESC, id DESC
    ) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_project_active_import_source
    ON ai_agent_project (
        tenant_id, organization_id, owner_user_id, import_source_kind, import_source_ref
    ) WHERE import_source_kind IS NOT NULL
        AND import_source_ref IS NOT NULL
        AND deleted_at IS NULL;
