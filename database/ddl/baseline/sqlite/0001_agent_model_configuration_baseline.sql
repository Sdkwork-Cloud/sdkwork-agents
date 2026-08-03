-- Agent model configuration profile baseline (SQLite local adapter).
-- Single source of truth for `SqliteAgentConfigurationStore`; mirrors the
-- kernel `AgentConfigurationProfile` shape so profiles survive restarts.
-- The canonical PostgreSQL baseline remains the authority for the managed
-- agents database; this local adapter is the persistence surface for the
-- model configuration runtime profiles.

CREATE TABLE IF NOT EXISTS agent_model_configuration_profile (
    profile_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    configuration_version TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    configuration_json TEXT NOT NULL DEFAULT '{}',
    secret_bindings_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_agent_model_configuration_profile_agent
    ON agent_model_configuration_profile (agent_id, status);
