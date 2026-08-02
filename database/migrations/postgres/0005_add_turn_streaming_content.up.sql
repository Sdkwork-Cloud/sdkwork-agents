-- 0005: turn streaming content checkpoint (7.3.0)
--
-- Streaming turn execution previously buffered assistant deltas only in
-- memory and wrote the response item once at turn completion: a crash during
-- a long turn lost the entire reply. The sink now checkpoints accumulated
-- deltas into `streaming_content` on the turn row (throttled), so a crashed
-- turn retains its partial reply and recovery can surface or merge it.
-- Completed turns clear the column in the same completion transaction.

ALTER TABLE ai_agent_turn
    ADD COLUMN IF NOT EXISTS streaming_content TEXT;

COMMIT;
