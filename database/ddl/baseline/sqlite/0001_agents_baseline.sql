-- SDKWork Agents SQLite managed-store schema.
-- Timestamps are canonical RFC3339 TEXT values; JSON columns are validated TEXT.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS ai_agent (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    owner_user_id INTEGER NOT NULL,
    agent_id TEXT NOT NULL,
    code TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    manifest_json TEXT NOT NULL CHECK (json_valid(manifest_json)),
    manifest_schema_version TEXT,
    default_code_task_intent_json TEXT CHECK (default_code_task_intent_json IS NULL OR json_valid(default_code_task_intent_json)),
    implementation_provider_id TEXT,
    implementation_kind TEXT CHECK (implementation_kind IS NULL OR implementation_kind IN ('manifest-only','typed-local-provider','process-adapter','protocol-adapter')),
    implementation_type TEXT NOT NULL DEFAULT 'sdkwork-native' CHECK (implementation_type IN ('sdkwork-native','rig-rust','openai-agents','langchain','langgraph','crewai','autogen','semantic-kernel','custom')),
    status INTEGER NOT NULL CHECK (status IN (0,1,2,3,4)),
    visibility INTEGER NOT NULL CHECK (visibility IN (0,1,2,3)),
    tags_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(tags_json) AND json_type(tags_json) = 'array'),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    version INTEGER NOT NULL DEFAULT 0 CHECK (version >= 0),
    UNIQUE (tenant_id, agent_id),
    UNIQUE (tenant_id, code),
    UNIQUE (tenant_id, id)
);
CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_org_status_updated ON ai_agent (tenant_id, organization_id, status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_tenant_owner_status ON ai_agent (tenant_id, owner_user_id, status);
CREATE INDEX IF NOT EXISTS idx_ai_agent_list_filters ON ai_agent (tenant_id, organization_id, owner_user_id, deleted_at) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS ai_agent_runtime_binding (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    agent_id TEXT NOT NULL,
    binding_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    implementation_kind TEXT NOT NULL CHECK (implementation_kind IN ('manifest-only','typed-local-provider','process-adapter','protocol-adapter')),
    configuration_profile_id TEXT NOT NULL,
    capabilities_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(capabilities_json) AND json_type(capabilities_json) = 'array'),
    active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0,1)),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, agent_id, binding_id),
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent (tenant_id, agent_id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_ai_agent_runtime_binding_active_default ON ai_agent_runtime_binding (tenant_id, agent_id) WHERE active = 1;
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_tenant_agent_updated ON ai_agent_runtime_binding (tenant_id, agent_id, active DESC, updated_at DESC, binding_id ASC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_runtime_binding_tenant_provider ON ai_agent_runtime_binding (tenant_id, provider_id);

CREATE TABLE IF NOT EXISTS ai_agent_composition_slot (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    agent_id TEXT NOT NULL,
    slot_id TEXT NOT NULL,
    slot_kind TEXT NOT NULL CHECK (slot_kind IN ('memory','knowledge','skill','prompt','drive','tool','mcp')),
    target_module TEXT NOT NULL CHECK (target_module IN ('memory','knowledgebase','skills','prompts','drive','mcp')),
    target_ref TEXT NOT NULL,
    target_version_ref TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0,1)),
    policy_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(policy_json) AND json_type(policy_json) = 'object'),
    status INTEGER NOT NULL DEFAULT 1 CHECK (status IN (0,1,2,3,4)),
    version INTEGER NOT NULL DEFAULT 0 CHECK (version >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    UNIQUE (tenant_id, agent_id, slot_id),
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent (tenant_id, agent_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_agent_composition_slot_lookup ON ai_agent_composition_slot (tenant_id, agent_id, slot_kind, enabled, priority, slot_id);

CREATE TABLE IF NOT EXISTS ai_agent_audit_event (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    agent_internal_id INTEGER NOT NULL,
    agent_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('created','updated','deleted','restored','status_changed','started','completed','failed','cancelled','provider_binding_changed','composition_slot_created','composition_slot_updated','composition_slot_deleted','session_created','session_closed','session_archived','message_created','message_failed')),
    subject_id TEXT NOT NULL,
    subject_tenant_id TEXT NOT NULL,
    request_id TEXT,
    trace_id TEXT,
    payload_json TEXT NOT NULL CHECK (json_valid(payload_json)),
    created_at TEXT NOT NULL,
    FOREIGN KEY (tenant_id, agent_internal_id) REFERENCES ai_agent (tenant_id, id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_agent_created ON ai_agent_audit_event (tenant_id, agent_id, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_audit_tenant_action_created ON ai_agent_audit_event (tenant_id, action, created_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS ai_agent_session (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    agent_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    owner_user_id INTEGER NOT NULL,
    title TEXT,
    status INTEGER NOT NULL DEFAULT 0 CHECK (status IN (0,1,2,3)),
    provider_binding_id TEXT,
    model_id TEXT,
    message_count INTEGER NOT NULL DEFAULT 0 CHECK (message_count >= 0),
    total_input_tokens INTEGER NOT NULL DEFAULT 0 CHECK (total_input_tokens >= 0),
    total_output_tokens INTEGER NOT NULL DEFAULT 0 CHECK (total_output_tokens >= 0),
    metadata_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(metadata_json) AND json_type(metadata_json) = 'object'),
    version INTEGER NOT NULL DEFAULT 0 CHECK (version >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_message_at TEXT,
    closed_at TEXT,
    UNIQUE (tenant_id, session_id),
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent (tenant_id, agent_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_tenant_agent_status_updated ON ai_agent_session (tenant_id, agent_id, status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_tenant_owner_status ON ai_agent_session (tenant_id, owner_user_id, status);
CREATE INDEX IF NOT EXISTS idx_ai_agent_session_tenant_org_status ON ai_agent_session (tenant_id, organization_id, status, updated_at DESC);

CREATE TABLE IF NOT EXISTS ai_agent_message (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    session_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    role INTEGER NOT NULL CHECK (role IN (0,1,2,3)),
    content TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text/plain',
    status INTEGER NOT NULL DEFAULT 0 CHECK (status IN (0,1,2,3,4)),
    sequence INTEGER NOT NULL CHECK (sequence >= 1),
    input_tokens INTEGER NOT NULL DEFAULT 0 CHECK (input_tokens >= 0),
    output_tokens INTEGER NOT NULL DEFAULT 0 CHECK (output_tokens >= 0),
    model_id TEXT,
    provider_id TEXT,
    artifacts_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(artifacts_json) AND json_type(artifacts_json) = 'array'),
    metadata_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(metadata_json) AND json_type(metadata_json) = 'object'),
    parent_message_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, session_id, message_id),
    UNIQUE (tenant_id, session_id, sequence),
    FOREIGN KEY (tenant_id, session_id) REFERENCES ai_agent_session (tenant_id, session_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_agent_message_tenant_session_sequence ON ai_agent_message (tenant_id, session_id, sequence ASC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_message_tenant_session_role_created ON ai_agent_message (tenant_id, session_id, role, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_message_tenant_agent_created ON ai_agent_message (tenant_id, agent_id, created_at DESC);

CREATE TABLE IF NOT EXISTS ai_agent_interaction (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    session_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    engine_key TEXT NOT NULL,
    interaction_id TEXT NOT NULL,
    kind INTEGER NOT NULL CHECK (kind IN (0,1)),
    status INTEGER NOT NULL DEFAULT 0 CHECK (status IN (0,1,2,3,4)),
    prompt TEXT NOT NULL,
    options_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(options_json) AND json_type(options_json) = 'array'),
    resolution_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(resolution_json)),
    version INTEGER NOT NULL DEFAULT 0 CHECK (version >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    resolved_at TEXT,
    UNIQUE (tenant_id, session_id, interaction_id),
    FOREIGN KEY (tenant_id, session_id) REFERENCES ai_agent_session (tenant_id, session_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_agent_interaction_tenant_session_status ON ai_agent_interaction (tenant_id, session_id, status, created_at DESC);

CREATE TABLE IF NOT EXISTS ai_agent_task (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    agent_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    owner_user_id INTEGER NOT NULL,
    title TEXT,
    prompt TEXT NOT NULL,
    status INTEGER NOT NULL DEFAULT 0 CHECK (status IN (0,1,2,3,4)),
    external_ref TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(metadata_json) AND json_type(metadata_json) = 'object'),
    version INTEGER NOT NULL DEFAULT 0 CHECK (version >= 0),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    cancelled_at TEXT,
    UNIQUE (tenant_id, task_id),
    FOREIGN KEY (tenant_id, agent_id) REFERENCES ai_agent (tenant_id, agent_id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_tenant_agent_status_updated ON ai_agent_task (tenant_id, agent_id, status, updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_agent_task_tenant_owner_status ON ai_agent_task (tenant_id, owner_user_id, status);
