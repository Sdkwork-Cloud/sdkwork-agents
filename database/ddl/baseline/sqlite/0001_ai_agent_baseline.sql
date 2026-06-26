-- SDKWork Agents SQLite baseline (composition plane, L2 portable subset)

CREATE TABLE IF NOT EXISTS ai_app_registry (
    id INTEGER NOT NULL PRIMARY KEY,
    tenant_id INTEGER NOT NULL,
    application_key TEXT NOT NULL,
    kernel_slot_id TEXT NOT NULL,
    default_agent_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, application_key)
);

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
    manifest_json TEXT NOT NULL,
    manifest_schema_version TEXT,
    default_code_task_intent_json TEXT,
    implementation_provider_id TEXT,
    implementation_kind TEXT,
    implementation_type TEXT NOT NULL DEFAULT 'sdkwork-native',
    status INTEGER NOT NULL,
    visibility INTEGER NOT NULL,
    tags_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    version INTEGER NOT NULL DEFAULT 0,
    UNIQUE (tenant_id, agent_id),
    UNIQUE (tenant_id, code)
);

CREATE TABLE IF NOT EXISTS ai_agent_runtime_binding (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    agent_id TEXT NOT NULL,
    binding_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    implementation_kind TEXT NOT NULL,
    configuration_profile_id TEXT NOT NULL,
    capabilities_json TEXT NOT NULL DEFAULT '[]',
    active INTEGER NOT NULL DEFAULT 0,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, agent_id, binding_id)
);

CREATE TABLE IF NOT EXISTS ai_agent_deployment (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    agent_id TEXT NOT NULL,
    deployment_id TEXT NOT NULL,
    binding_id TEXT NOT NULL,
    provider_id_snapshot TEXT NOT NULL,
    implementation_kind_snapshot TEXT NOT NULL,
    configuration_profile_id_snapshot TEXT NOT NULL,
    capabilities_snapshot_json TEXT NOT NULL DEFAULT '[]',
    status INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, agent_id, deployment_id)
);

CREATE TABLE IF NOT EXISTS ai_agent_composition_slot (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    agent_id TEXT NOT NULL,
    slot_id TEXT NOT NULL,
    slot_kind TEXT NOT NULL,
    target_module TEXT NOT NULL,
    target_ref TEXT NOT NULL,
    target_version_ref TEXT,
    priority INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    policy_json TEXT NOT NULL DEFAULT '{}',
    status INTEGER NOT NULL DEFAULT 1,
    version INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT,
    UNIQUE (tenant_id, agent_id, slot_id)
);

CREATE TABLE IF NOT EXISTS ai_agent_audit_event (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL UNIQUE,
    tenant_id INTEGER NOT NULL,
    organization_id INTEGER NOT NULL DEFAULT 0,
    agent_internal_id INTEGER NOT NULL,
    agent_id TEXT NOT NULL,
    action TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    subject_tenant_id TEXT NOT NULL,
    request_id TEXT,
    trace_id TEXT,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS ai_agent_outbox_event (
    id INTEGER NOT NULL PRIMARY KEY,
    uuid TEXT NOT NULL,
    tenant_id INTEGER NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    status INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    published_at TEXT,
    version INTEGER NOT NULL DEFAULT 0,
    UNIQUE (tenant_id, uuid)
);
