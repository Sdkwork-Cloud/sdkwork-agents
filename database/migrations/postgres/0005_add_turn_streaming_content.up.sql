-- sdkwork:migration
-- id: 0005_add_turn_streaming_content
-- engine: postgres
-- module: agents
-- purpose: Persist throttled streaming turn deltas on the turn row so a crashed long turn retains its partial reply.
-- reversible: false
-- rollback: forward-fix
-- transactional: true
-- lock: table-metadata-ddl
-- lock_timeout: 2s
-- statement_timeout: 2min
-- rewrite: Nullable TEXT column; completed turns clear it in the completion transaction.
-- replication_impact: Catalog DDL only.
-- backfill: Existing turn rows retain a null streaming_content.
-- observability: Migration history, PostgreSQL lock waits, schema readiness, and drift verification.
-- contract_version: 7.3.0

BEGIN;

-- Streaming turn execution previously buffered assistant deltas only in
-- memory and wrote the response item once at turn completion: a crash during
-- a long turn lost the entire reply. The sink now checkpoints accumulated
-- deltas into `streaming_content` on the turn row (throttled), so a crashed
-- turn retains its partial reply and recovery can surface or merge it.
-- Completed turns clear the column in the same completion transaction.

ALTER TABLE ai_agent_turn
    ADD COLUMN IF NOT EXISTS streaming_content TEXT;

COMMIT;
