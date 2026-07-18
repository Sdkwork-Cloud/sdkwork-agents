pub const SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json::text AS manifest_json, default_code_task_intent_json::text AS default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json::text AS tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 AND agent_id = $2 LIMIT 1";
pub const SQL_INSERT_AGENT: &str =
    "INSERT INTO ai_agent (id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at, updated_at, deleted_at, version) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, $11::jsonb, $12, $13, $14, $15, $16, $17::jsonb, $18::timestamptz, $19::timestamptz, $20::timestamptz, $21)";
pub const SQL_UPDATE_AGENT: &str =
    "UPDATE ai_agent SET organization_id = $1, owner_user_id = $2, code = $3, display_name = $4, description = $5, manifest_json = $6::jsonb, default_code_task_intent_json = $7::jsonb, implementation_provider_id = $8, implementation_kind = $9, implementation_type = $10, status = $11, visibility = $12, tags_json = $13::jsonb, updated_at = $14::timestamptz, deleted_at = $15::timestamptz, version = $16 WHERE tenant_id = $17 AND agent_id = $18 AND version = $19";
pub const SQL_LIST_AGENT: &str =
    "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json::text AS manifest_json, default_code_task_intent_json::text AS default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json::text AS tags_json, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, version FROM ai_agent WHERE tenant_id = $1 AND ($2 IS NULL OR organization_id = $2) AND ($3 IS NULL OR owner_user_id = $3) AND (deleted_at IS NULL OR $4::bool = true) AND ($5::text IS NULL OR LOWER(agent_id) LIKE LOWER($5::text) OR LOWER(code) LIKE LOWER($5::text) OR LOWER(display_name) LIKE LOWER($5::text) OR LOWER(COALESCE(description, '')) LIKE LOWER($5::text)) AND ($6::smallint IS NULL OR visibility = $6) ORDER BY updated_at DESC, id DESC LIMIT $7 OFFSET $8";

// All filtering (organization_id, owner_user_id, include_deleted, search_query, visibility)
// is pushed to SQL WHERE clause. Parameters: $1=tenant_id, $2=org_filter(NULL=any),
// $3=owner_filter(NULL=any), $4=include_deleted_flag, $5=search_query(NULL=none, wrapped %...% for LIKE),
// $6=visibility(NULL=any), $7=page_size, $8=offset.
pub const SQL_COUNT_AGENT: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent WHERE tenant_id = $1 AND ($2 IS NULL OR organization_id = $2) AND ($3 IS NULL OR owner_user_id = $3) AND (deleted_at IS NULL OR $4::bool = true) AND ($5::text IS NULL OR LOWER(agent_id) LIKE LOWER($5::text) OR LOWER(code) LIKE LOWER($5::text) OR LOWER(display_name) LIKE LOWER($5::text) OR LOWER(COALESCE(description, '')) LIKE LOWER($5::text)) AND ($6::smallint IS NULL OR visibility = $6)";
pub const SQL_INSERT_AGENT_PROVIDER_BINDING: &str =
    "INSERT INTO ai_agent_runtime_binding (id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10, $11, $12::timestamptz, $13::timestamptz)";
pub const SQL_UPDATE_AGENT_PROVIDER_BINDING: &str =
    "UPDATE ai_agent_runtime_binding SET provider_id = $1, implementation_kind = $2, configuration_profile_id = $3, capabilities_json = $4::jsonb, active = $5, version = $6, updated_at = $7::timestamptz WHERE tenant_id = $8 AND agent_id = $9 AND binding_id = $10 AND version = $11";
pub const SQL_DEACTIVATE_ACTIVE_AGENT_PROVIDER_BINDINGS: &str =
    "UPDATE ai_agent_runtime_binding SET active = false, version = version + 1, updated_at = $3::timestamptz WHERE tenant_id = $1 AND agent_id = $2 AND active = true";
pub const SQL_ACTIVATE_AGENT_PROVIDER_BINDING: &str =
    "UPDATE ai_agent_runtime_binding SET active = true, version = $4, updated_at = $5::timestamptz WHERE tenant_id = $1 AND agent_id = $2 AND binding_id = $3 AND version = $6";
pub const SQL_SELECT_AGENT_PROVIDER_BINDING: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json::text AS capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 AND binding_id = $3 LIMIT 1";
/// Load the single active provider binding for an agent without paginated scans.
/// Used on hot paths (chat, task, preview) where only the active binding matters.
pub const SQL_SELECT_ACTIVE_AGENT_PROVIDER_BINDING: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json::text AS capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 AND active = TRUE LIMIT 1";
pub const SQL_LIST_AGENT_PROVIDER_BINDINGS: &str =
    "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json::text AS capabilities_json, active, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2 ORDER BY active DESC, updated_at DESC, binding_id ASC LIMIT $3 OFFSET $4";
pub const SQL_COUNT_AGENT_PROVIDER_BINDINGS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_runtime_binding WHERE tenant_id = $1 AND agent_id = $2";
pub const SQL_INSERT_AUDIT_EVENT: &str =
    "INSERT INTO ai_agent_audit_event (id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb, $13::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json::text AS payload_json, created_at::text AS created_at FROM ai_agent_audit_event WHERE tenant_id = $1 AND agent_id = $2 AND ($3::text IS NULL OR action = $3) AND ($4::text IS NULL OR created_at >= $4::timestamptz) AND ($5::text IS NULL OR created_at <= $5::timestamptz) ORDER BY created_at DESC, id DESC LIMIT $6 OFFSET $7";
pub const SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_audit_event WHERE tenant_id = $1 AND agent_id = $2 AND ($3::text IS NULL OR action = $3) AND ($4::text IS NULL OR created_at >= $4::timestamptz) AND ($5::text IS NULL OR created_at <= $5::timestamptz)";
pub const SQL_INSERT_AGENT_COMPOSITION_SLOT: &str =
    "INSERT INTO ai_agent_composition_slot (id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, status, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb, $14, $15, $16::timestamptz, $17::timestamptz, $18::timestamptz)";
pub const SQL_UPDATE_AGENT_COMPOSITION_SLOT: &str =
    "UPDATE ai_agent_composition_slot SET organization_id = $1, slot_kind = $2, target_module = $3, target_ref = $4, target_version_ref = $5, priority = $6, enabled = $7, policy_json = $8::jsonb, status = $9, version = $10, updated_at = $11::timestamptz, deleted_at = $12::timestamptz WHERE tenant_id = $13 AND agent_id = $14 AND slot_id = $15 AND version = $16";
pub const SQL_SELECT_AGENT_COMPOSITION_SLOT: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND slot_id = $3 LIMIT 1";
pub const SQL_LIST_AGENT_COMPOSITION_SLOTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, status, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND deleted_at IS NULL ORDER BY priority ASC, slot_id ASC LIMIT $3 OFFSET $4";
pub const SQL_COUNT_AGENT_COMPOSITION_SLOTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_composition_slot WHERE tenant_id = $1 AND agent_id = $2 AND deleted_at IS NULL";
pub const SQL_LIST_MCP_MARKETPLACE_SLOTS: &str =
    "SELECT s.id, s.uuid, s.tenant_id, s.organization_id, s.agent_id, s.slot_id, s.slot_kind, s.target_module, s.target_ref, s.target_version_ref, s.priority, s.enabled, s.policy_json::text AS policy_json, s.status, s.version, s.created_at::text AS created_at, s.updated_at::text AS updated_at, s.deleted_at::text AS deleted_at FROM ai_agent_composition_slot s INNER JOIN ai_agent a ON a.tenant_id = s.tenant_id AND a.agent_id = s.agent_id WHERE s.tenant_id = $1 AND s.slot_kind = 'mcp' AND s.deleted_at IS NULL AND a.deleted_at IS NULL AND ($2::text IS NULL OR (s.target_ref ILIKE $2 OR s.slot_id ILIKE $2 OR s.agent_id ILIKE $2)) ORDER BY s.priority ASC, s.slot_id ASC LIMIT $3 OFFSET $4";
pub const SQL_COUNT_MCP_MARKETPLACE_SLOTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_composition_slot s INNER JOIN ai_agent a ON a.tenant_id = s.tenant_id AND a.agent_id = s.agent_id WHERE s.tenant_id = $1 AND s.slot_kind = 'mcp' AND s.deleted_at IS NULL AND a.deleted_at IS NULL AND ($2::text IS NULL OR (s.target_ref ILIKE $2 OR s.slot_id ILIKE $2 OR s.agent_id ILIKE $2))";

// Session SQL constants
// Session SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_SESSION: &str =
    "INSERT INTO ai_agent_session (id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, title, status, provider_binding_id, model_id, message_count, total_input_tokens, total_output_tokens, metadata_json, version, created_at, updated_at, last_message_at, closed_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15::jsonb, $16, $17::timestamptz, $18::timestamptz, $19::timestamptz, $20::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_SESSION: &str =
    "UPDATE ai_agent_session SET title = $1, status = $2, provider_binding_id = $3, model_id = $4, message_count = $5, total_input_tokens = $6, total_output_tokens = $7, metadata_json = $8::jsonb, version = $9, updated_at = $10::timestamptz, last_message_at = $11::timestamptz, closed_at = $12::timestamptz WHERE tenant_id = $13 AND session_id = $14 AND version = $15";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, title, status, provider_binding_id, model_id, message_count, total_input_tokens, total_output_tokens, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, last_message_at::text AS last_message_at, closed_at::text AS closed_at FROM ai_agent_session WHERE tenant_id = $1 AND session_id = $2 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSIONS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, title, status, provider_binding_id, model_id, message_count, total_input_tokens, total_output_tokens, metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, last_message_at::text AS last_message_at, closed_at::text AS closed_at FROM ai_agent_session WHERE tenant_id = $1 AND ($2::text IS NULL OR agent_id = $2) AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::smallint IS NULL OR status = $4) AND ($5::bool = true OR status != 3) ORDER BY updated_at DESC LIMIT $6 OFFSET $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_SESSIONS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_session WHERE tenant_id = $1 AND ($2::text IS NULL OR agent_id = $2) AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::smallint IS NULL OR status = $4) AND ($5::bool = true OR status != 3)";

// Message SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_MESSAGE: &str =
    "INSERT INTO ai_agent_message (id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json, metadata_json, parent_message_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16::jsonb, $17::jsonb, $18, $19::timestamptz, $20::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_MESSAGE: &str =
    "UPDATE ai_agent_message SET content = $1, content_type = $2, status = $3, model_id = $4, provider_id = $5, artifacts_json = $6::jsonb, metadata_json = $7::jsonb, updated_at = $8::timestamptz WHERE tenant_id = $9 AND session_id = $10 AND message_id = $11";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_MESSAGE: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json::text AS artifacts_json, metadata_json::text AS metadata_json, parent_message_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND message_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGES: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json::text AS artifacts_json, metadata_json::text AS metadata_json, parent_message_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND ($3::smallint IS NULL OR role = $3) AND ($4::smallint IS NULL OR status = $4) ORDER BY sequence ASC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json::text AS artifacts_json, metadata_json::text AS metadata_json, parent_message_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND ($3::smallint IS NULL OR role = $3) AND ($4::smallint IS NULL OR status = $4) ORDER BY sequence DESC LIMIT $5";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_MESSAGES: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND ($3::smallint IS NULL OR role = $3) AND ($4::smallint IS NULL OR status = $4)";
#[cfg(feature = "postgres-sync")]
pub const SQL_NEXT_MESSAGE_SEQUENCE: &str =
    "SELECT COALESCE(MAX(sequence), 0) + 1 AS next_sequence FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2";
#[cfg(feature = "postgres-sync")]
pub const SQL_LOCK_AGENT_SESSION_FOR_UPDATE: &str =
    "SELECT session_id FROM ai_agent_session WHERE tenant_id = $1 AND session_id = $2 FOR UPDATE";

// Interaction SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_INTERACTION: &str =
    "INSERT INTO ai_agent_interaction (id, uuid, tenant_id, organization_id, session_id, agent_id, engine_key, interaction_id, kind, status, prompt, options_json, resolution_json, version, created_at, updated_at, resolved_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb, $13::jsonb, $14, $15::timestamptz, $16::timestamptz, $17::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_INTERACTION: &str =
    "UPDATE ai_agent_interaction SET kind = $1, status = $2, prompt = $3, options_json = $4::jsonb, resolution_json = $5::jsonb, version = $6, updated_at = $7::timestamptz, resolved_at = $8::timestamptz WHERE tenant_id = $9 AND session_id = $10 AND interaction_id = $11 AND version = $12";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_INTERACTION: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, agent_id, engine_key, interaction_id, kind, status, prompt, options_json::text AS options_json, resolution_json::text AS resolution_json, version, created_at::text AS created_at, updated_at::text AS updated_at, resolved_at::text AS resolved_at FROM ai_agent_interaction WHERE tenant_id = $1 AND session_id = $2 AND interaction_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_INTERACTIONS: &str =
    "SELECT id, uuid, tenant_id, organization_id, session_id, agent_id, engine_key, interaction_id, kind, status, prompt, options_json::text AS options_json, resolution_json::text AS resolution_json, version, created_at::text AS created_at, updated_at::text AS updated_at, resolved_at::text AS resolved_at FROM ai_agent_interaction WHERE tenant_id = $1 AND session_id = $2 AND ($3::smallint IS NULL OR status = $3) ORDER BY created_at DESC LIMIT $4 OFFSET $5";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_INTERACTIONS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_interaction WHERE tenant_id = $1 AND session_id = $2 AND ($3::smallint IS NULL OR status = $3)";

// Task SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_TASK: &str =
    "INSERT INTO ai_agent_task (id, uuid, tenant_id, organization_id, agent_id, task_id, owner_user_id, title, prompt, status, external_ref, metadata_json, version, created_at, updated_at, started_at, completed_at, cancelled_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb, $13, $14::timestamptz, $15::timestamptz, $16::timestamptz, $17::timestamptz, $18::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_TASK: &str =
    "UPDATE ai_agent_task SET title = $1, prompt = $2, status = $3, external_ref = $4, metadata_json = $5::jsonb, version = $6, updated_at = $7::timestamptz, started_at = $8::timestamptz, completed_at = $9::timestamptz, cancelled_at = $10::timestamptz WHERE tenant_id = $11 AND task_id = $12 AND version = $13";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_TASK: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, task_id, owner_user_id, title, prompt, status, external_ref, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancelled_at::text AS cancelled_at FROM ai_agent_task WHERE tenant_id = $1 AND task_id = $2 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_TASKS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, task_id, owner_user_id, title, prompt, status, external_ref, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancelled_at::text AS cancelled_at FROM ai_agent_task WHERE tenant_id = $1 AND ($2::text IS NULL OR agent_id = $2) AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::smallint IS NULL OR status = $4) ORDER BY updated_at DESC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_TASKS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_task WHERE tenant_id = $1 AND ($2::text IS NULL OR agent_id = $2) AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::smallint IS NULL OR status = $4)";
