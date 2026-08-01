-- sdkwork:migration
-- id: 0001_complete_agents_7_0_0_schema
-- engine: postgres
-- module: agents
-- purpose: Complete existing shared development schemas with the canonical Agents 7.0.0 execution tables.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: catalog-and-new-table-ddl
-- lock_timeout: 2s
-- statement_timeout: 2min
-- rewrite: Adds one validated unique constraint and creates tables; no existing row is rewritten.
-- replication_impact: Bounded to constraint validation and catalog DDL; no row backfill is required.
-- backfill: None; the new queue and scheduling tables have no legacy rows to transform.
-- observability: Migration history, PostgreSQL lock waits, schema readiness, and drift verification.
-- cancellation: Transaction rollback removes every table and index created by this migration.
-- recovery: Resolve a rejected partial schema and deploy a reviewed forward-fix before accepting Agents writes.
-- contract_version: 7.0.0

BEGIN;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'ai_agent_session'::regclass
          AND conname = 'uk_ai_agent_session_scope_agent_owner'
    ) THEN
        ALTER TABLE ai_agent_session
            ADD CONSTRAINT uk_ai_agent_session_scope_agent_owner UNIQUE (
                tenant_id, organization_id, session_id, agent_id, owner_user_id
            );
    END IF;
END
$$;

CREATE TABLE IF NOT EXISTS ai_agent_turn_input_queue_entry (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    queue_entry_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    content TEXT NOT NULL,
    display_text TEXT NOT NULL DEFAULT '',
    content_type VARCHAR(128) NOT NULL DEFAULT 'text/plain',
    attachment_names_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    drive_refs_json JSONB NOT NULL DEFAULT '[]'::jsonb,
    turn_mode SMALLINT NOT NULL DEFAULT 0,
    runtime_binding_id VARCHAR(128),
    requested_model_id VARCHAR(128),
    access_mode_id VARCHAR(64),
    idempotency_key VARCHAR(256) NOT NULL,
    payload_hash VARCHAR(128) NOT NULL,
    client_request_id VARCHAR(128) NOT NULL,
    position BIGINT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    claim_owner VARCHAR(128),
    claim_token_hash VARCHAR(128),
    claim_expires_at TIMESTAMPTZ,
    fencing_token BIGINT NOT NULL DEFAULT 0,
    error_code VARCHAR(128),
    error_detail VARCHAR(1024),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    claimed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_turn_input_queue_entry_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_turn_input_queue_entry_scope UNIQUE (
        tenant_id, organization_id, session_id, queue_entry_id
    ),
    CONSTRAINT uk_ai_agent_turn_input_queue_entry_idempotency UNIQUE (
        tenant_id, organization_id, owner_user_id, idempotency_key
    ),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_status CHECK (status IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_mode CHECK (turn_mode IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_content CHECK (
        octet_length(BTRIM(content)) > 0 AND octet_length(content) <= 262144
    ),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_position CHECK (position > 0),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_fencing CHECK (fencing_token >= 0),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_version CHECK (version >= 0),
    CONSTRAINT ck_ai_agent_turn_input_queue_entry_claim CHECK (
        (status = 1 AND claim_owner IS NOT NULL AND claim_token_hash IS NOT NULL AND claim_expires_at IS NOT NULL)
        OR (status <> 1 AND claim_owner IS NULL AND claim_token_hash IS NULL AND claim_expires_at IS NULL)
    ),
    CONSTRAINT fk_ai_agent_turn_input_queue_entry_session FOREIGN KEY (
        tenant_id, organization_id, session_id
    ) REFERENCES ai_agent_session (tenant_id, organization_id, session_id) ON DELETE CASCADE,
    CONSTRAINT fk_ai_agent_turn_input_queue_entry_agent FOREIGN KEY (tenant_id, agent_id)
        REFERENCES ai_agent (tenant_id, agent_id)
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_input_queue_session_order
    ON ai_agent_turn_input_queue_entry (
        tenant_id, organization_id, session_id, owner_user_id, position, id
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_turn_input_queue_owner_quota
    ON ai_agent_turn_input_queue_entry (tenant_id, organization_id, owner_user_id, id);
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_turn_input_queue_executing_session
    ON ai_agent_turn_input_queue_entry (tenant_id, organization_id, session_id)
    WHERE status = 1;

CREATE TABLE IF NOT EXISTS ai_agent_task_run (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    run_id VARCHAR(128) NOT NULL,
    task_id VARCHAR(128) NOT NULL,
    session_id VARCHAR(128) NOT NULL,
    agent_id VARCHAR(128) NOT NULL,
    owner_user_id BIGINT NOT NULL,
    trigger_kind SMALLINT NOT NULL,
    schedule_generation BIGINT NOT NULL,
    scheduled_for TIMESTAMPTZ NOT NULL,
    retry_of_run_id VARCHAR(128),
    priority SMALLINT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    idempotency_key VARCHAR(256) NOT NULL,
    payload_hash VARCHAR(128) NOT NULL,
    turn_id VARCHAR(128),
    attempt_count SMALLINT NOT NULL DEFAULT 0,
    max_attempts SMALLINT NOT NULL,
    available_at TIMESTAMPTZ NOT NULL,
    lease_owner VARCHAR(128),
    lease_token_hash VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    fencing_token BIGINT NOT NULL DEFAULT 0,
    timeout_at TIMESTAMPTZ,
    failure_class VARCHAR(64),
    error_code VARCHAR(128),
    error_detail VARCHAR(1024),
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    claimed_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_task_run_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_task_run_scope UNIQUE (
        tenant_id, organization_id, run_id
    ),
    CONSTRAINT uk_ai_agent_task_run_idempotency UNIQUE (
        tenant_id, organization_id, owner_user_id, idempotency_key
    ),
    CONSTRAINT ck_ai_agent_task_run_trigger CHECK (trigger_kind IN (0, 1, 2)),
    CONSTRAINT ck_ai_agent_task_run_status CHECK (status IN (0, 1, 2, 3, 4, 5, 6, 7)),
    CONSTRAINT ck_ai_agent_task_run_generation CHECK (schedule_generation > 0),
    CONSTRAINT ck_ai_agent_task_run_priority CHECK (priority BETWEEN -100 AND 100),
    CONSTRAINT ck_ai_agent_task_run_attempts CHECK (
        max_attempts BETWEEN 1 AND 20
        AND attempt_count BETWEEN 0 AND max_attempts
    ),
    CONSTRAINT ck_ai_agent_task_run_fencing CHECK (fencing_token >= 0),
    CONSTRAINT ck_ai_agent_task_run_version CHECK (version >= 0),
    CONSTRAINT ck_ai_agent_task_run_retry CHECK (
        (trigger_kind = 2 AND retry_of_run_id IS NOT NULL)
        OR (trigger_kind <> 2 AND retry_of_run_id IS NULL)
    ),
    CONSTRAINT ck_ai_agent_task_run_lease CHECK (
        (status IN (1, 2) AND lease_owner IS NOT NULL AND lease_token_hash IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR (status NOT IN (1, 2) AND lease_owner IS NULL AND lease_token_hash IS NULL AND lease_expires_at IS NULL)
    ),
    CONSTRAINT ck_ai_agent_task_run_terminal CHECK (
        (status IN (3, 4, 5, 7) AND finished_at IS NOT NULL)
        OR (status NOT IN (3, 4, 5, 7) AND finished_at IS NULL)
    ),
    CONSTRAINT fk_ai_agent_task_run_task FOREIGN KEY (
        tenant_id, organization_id, task_id
    ) REFERENCES ai_agent_task (tenant_id, organization_id, task_id) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_task_run_session FOREIGN KEY (
        tenant_id, organization_id, session_id, agent_id, owner_user_id
    ) REFERENCES ai_agent_session (
        tenant_id, organization_id, session_id, agent_id, owner_user_id
    ) ON DELETE RESTRICT,
    CONSTRAINT fk_ai_agent_task_run_retry FOREIGN KEY (
        tenant_id, organization_id, retry_of_run_id
    ) REFERENCES ai_agent_task_run (tenant_id, organization_id, run_id)
        ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);

CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_task_run_occurrence
    ON ai_agent_task_run (
        tenant_id, organization_id, task_id, schedule_generation, scheduled_for
    ) WHERE trigger_kind = 0;
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_claim
    ON ai_agent_task_run (priority DESC, available_at, scheduled_for, id)
    WHERE status = 0;
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_active_task
    ON ai_agent_task_run (
        tenant_id, organization_id, task_id, status, scheduled_for, id
    ) WHERE status IN (1, 2, 6);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_list
    ON ai_agent_task_run (
        tenant_id, organization_id, task_id, id DESC
    ) INCLUDE (owner_user_id, status, trigger_kind);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_expired_lease
    ON ai_agent_task_run (lease_expires_at, id)
    WHERE status IN (1, 2);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_reconcile
    ON ai_agent_task_run (updated_at, id) WHERE status = 6;
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_retention
    ON ai_agent_task_run (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

CREATE TABLE IF NOT EXISTS ai_agent_task_run_attempt (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    attempt_id VARCHAR(128) NOT NULL,
    run_id VARCHAR(128) NOT NULL,
    attempt_no SMALLINT NOT NULL,
    worker_id VARCHAR(128) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    lease_token_hash VARCHAR(128) NOT NULL,
    fencing_token BIGINT NOT NULL,
    failure_class VARCHAR(64),
    error_code VARCHAR(128),
    error_detail VARCHAR(1024),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    heartbeat_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    retention_until TIMESTAMPTZ,
    CONSTRAINT uk_ai_agent_task_run_attempt_uuid UNIQUE (uuid),
    CONSTRAINT uk_ai_agent_task_run_attempt_scope UNIQUE (
        tenant_id, organization_id, attempt_id
    ),
    CONSTRAINT uk_ai_agent_task_run_attempt_number UNIQUE (
        tenant_id, organization_id, run_id, attempt_no
    ),
    CONSTRAINT ck_ai_agent_task_run_attempt_number CHECK (attempt_no BETWEEN 1 AND 20),
    CONSTRAINT ck_ai_agent_task_run_attempt_status CHECK (status IN (0, 1, 2, 3, 4, 5)),
    CONSTRAINT ck_ai_agent_task_run_attempt_fencing CHECK (fencing_token > 0),
    CONSTRAINT ck_ai_agent_task_run_attempt_terminal CHECK (
        (status IN (2, 3, 4, 5) AND finished_at IS NOT NULL)
        OR (status IN (0, 1) AND finished_at IS NULL)
    ),
    CONSTRAINT fk_ai_agent_task_run_attempt_run FOREIGN KEY (
        tenant_id, organization_id, run_id
    ) REFERENCES ai_agent_task_run (tenant_id, organization_id, run_id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_attempt_run
    ON ai_agent_task_run_attempt (
        tenant_id, organization_id, run_id, attempt_no DESC, id DESC
    );
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_attempt_worker
    ON ai_agent_task_run_attempt (worker_id, status, heartbeat_at, id)
    WHERE status IN (0, 1);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_run_attempt_retention
    ON ai_agent_task_run_attempt (tenant_id, organization_id, retention_until, id)
    WHERE retention_until IS NOT NULL;

COMMIT;
