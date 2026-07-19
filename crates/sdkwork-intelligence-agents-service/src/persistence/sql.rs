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

#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_PROJECT: &str =
    "INSERT INTO ai_agent_project (id, uuid, tenant_id, organization_id, project_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, created_by, updated_by, version, created_at, updated_at, archived_at, archived_by, deleted_at, deleted_by, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17::timestamptz, $18::timestamptz, $19::timestamptz, $20, $21::timestamptz, $22, $23::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_PROJECT: &str =
    "UPDATE ai_agent_project SET name = $1, description = $2, visibility = $3, status = $4, drive_access_mode = $5, default_agent_id = $6, default_model_id = $7, updated_by = $8, version = $9, updated_at = $10::timestamptz, archived_at = $11::timestamptz, archived_by = $12, deleted_at = $13::timestamptz, deleted_by = $14, retention_until = $15::timestamptz WHERE tenant_id = $16 AND organization_id = $17 AND project_id = $18 AND version = $19";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_PROJECT: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_PROJECTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, owner_user_id, name, description, visibility, status, drive_access_mode, default_agent_id, default_model_id, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, archived_at::text AS archived_at, archived_by, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::smallint IS NULL OR status = $4) AND ($5::text IS NULL OR name ILIKE $5 OR COALESCE(description, '') ILIKE $5) AND ($6::bool = TRUE OR deleted_at IS NULL) ORDER BY updated_at DESC, id DESC LIMIT $7 OFFSET $8";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_PROJECTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_project WHERE tenant_id = $1 AND organization_id = $2 AND ($3::bigint IS NULL OR owner_user_id = $3) AND ($4::smallint IS NULL OR status = $4) AND ($5::text IS NULL OR name ILIKE $5 OR COALESCE(description, '') ILIKE $5) AND ($6::bool = TRUE OR deleted_at IS NULL)";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT: &str =
    "INSERT INTO ai_agent_project_composition_slot (id, uuid, tenant_id, organization_id, project_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, created_by, updated_by, version, created_at, updated_at, deleted_at, deleted_by, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::jsonb, $14, $15, $16, $17::timestamptz, $18::timestamptz, $19::timestamptz, $20, $21::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT: &str =
    "UPDATE ai_agent_project_composition_slot SET slot_kind = $1, target_module = $2, target_ref = $3, target_version_ref = $4, priority = $5, enabled = $6, policy_json = $7::jsonb, updated_by = $8, version = $9, updated_at = $10::timestamptz, deleted_at = $11::timestamptz, deleted_by = $12, retention_until = $13::timestamptz WHERE tenant_id = $14 AND organization_id = $15 AND project_id = $16 AND slot_id = $17 AND version = $18";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project_composition_slot WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND slot_id = $4 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS: &str =
    "SELECT id, uuid, tenant_id, organization_id, project_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json::text AS policy_json, created_by, updated_by, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, deleted_by, retention_until::text AS retention_until FROM ai_agent_project_composition_slot WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND deleted_at IS NULL AND ($4::text IS NULL OR slot_kind = $4) AND ($5::bool IS NULL OR enabled = $5) ORDER BY priority ASC, id ASC LIMIT $6 OFFSET $7";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_project_composition_slot WHERE tenant_id = $1 AND organization_id = $2 AND project_id = $3 AND deleted_at IS NULL AND ($4::text IS NULL OR slot_kind = $4) AND ($5::bool IS NULL OR enabled = $5)";
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
    "INSERT INTO ai_agent_audit_event (id, uuid, tenant_id, organization_id, aggregate_type, aggregate_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7::bigint, (SELECT id FROM ai_agent WHERE tenant_id = $3 AND agent_id = $8)), $8, $9, $10, $11, $12, $13, $14::jsonb, $15::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID: &str =
    "SELECT id, uuid, tenant_id, organization_id, aggregate_type, aggregate_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json::text AS payload_json, created_at::text AS created_at FROM ai_agent_audit_event WHERE tenant_id = $1 AND agent_id = $2 AND ($3::text IS NULL OR action = $3) AND ($4::text IS NULL OR created_at >= $4::timestamptz) AND ($5::text IS NULL OR created_at <= $5::timestamptz) ORDER BY created_at DESC, id DESC LIMIT $6 OFFSET $7";
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
    "INSERT INTO ai_agent_session (id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, project_id, title, status, provider_binding_id, model_id, message_count, last_message_sequence, total_input_tokens, total_output_tokens, metadata_json, version, created_at, updated_at, last_message_at, closed_at, archived_at, deleted_at, created_by, updated_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, COALESCE(NULLIF(BTRIM($9), ''), 'New chat'), $10, $11, $12, $13, $14, $15, $16, $17::jsonb, $18, $19::timestamptz, $20::timestamptz, $21::timestamptz, $22::timestamptz, $23::timestamptz, $24::timestamptz, $6, $6)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_SESSION: &str =
    "UPDATE ai_agent_session SET project_id = $1, title = COALESCE(NULLIF(BTRIM($2), ''), title), status = $3, provider_binding_id = $4, model_id = $5, message_count = $6, last_message_sequence = GREATEST(last_message_sequence, $7), total_input_tokens = $8, total_output_tokens = $9, metadata_json = $10::jsonb, version = $11, updated_at = $12::timestamptz, updated_by = owner_user_id, last_message_at = $13::timestamptz, closed_at = $14::timestamptz, archived_at = $15::timestamptz, deleted_at = $16::timestamptz, deleted_by = CASE WHEN $16::timestamptz IS NULL THEN NULL ELSE owner_user_id END WHERE tenant_id = $17 AND organization_id = $18 AND session_id = $19 AND deleted_at IS NULL AND version = $20";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_SESSION: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, project_id, title, status, provider_binding_id, model_id, message_count, last_message_sequence, total_input_tokens, total_output_tokens, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, last_message_at::text AS last_message_at, closed_at::text AS closed_at, archived_at::text AS archived_at, deleted_at::text AS deleted_at FROM ai_agent_session WHERE tenant_id = $1 AND session_id = $2 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_SESSIONS: &str =
    "SELECT id, uuid, tenant_id, organization_id, agent_id, owner_user_id, session_id, project_id, title, status, provider_binding_id, model_id, message_count, last_message_sequence, total_input_tokens, total_output_tokens, metadata_json::text AS metadata_json, version, created_at::text AS created_at, updated_at::text AS updated_at, last_message_at::text AS last_message_at, closed_at::text AS closed_at, archived_at::text AS archived_at, deleted_at::text AS deleted_at FROM ai_agent_session WHERE tenant_id = $1 AND deleted_at IS NULL AND ($2::bigint IS NULL OR organization_id = $2) AND ($3::text IS NULL OR agent_id = $3) AND ($4::text IS NULL OR project_id = $4) AND ($5::bigint IS NULL OR owner_user_id = $5) AND ($6::smallint IS NULL OR status = $6) AND ($7::bool = true OR status != 3) ORDER BY updated_at DESC, id DESC LIMIT $8 OFFSET $9";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_SESSIONS: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_session WHERE tenant_id = $1 AND deleted_at IS NULL AND ($2::bigint IS NULL OR organization_id = $2) AND ($3::text IS NULL OR agent_id = $3) AND ($4::text IS NULL OR project_id = $4) AND ($5::bigint IS NULL OR owner_user_id = $5) AND ($6::smallint IS NULL OR status = $6) AND ($7::bool = true OR status != 3)";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPSERT_AGENT_RESOURCE_USER_STATE: &str =
    "INSERT INTO ai_agent_resource_user_state (id, uuid, tenant_id, organization_id, user_id, resource_type, resource_id, pinned_at, hidden_at, last_opened_at, last_read_message_sequence, custom_title, version, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $9::timestamptz, $10::timestamptz, $11, $12, 0, $13::timestamptz, $14::timestamptz) ON CONFLICT (tenant_id, organization_id, user_id, resource_type, resource_id) DO UPDATE SET pinned_at = EXCLUDED.pinned_at, hidden_at = EXCLUDED.hidden_at, last_opened_at = EXCLUDED.last_opened_at, last_read_message_sequence = EXCLUDED.last_read_message_sequence, custom_title = EXCLUDED.custom_title, version = ai_agent_resource_user_state.version + 1, updated_at = EXCLUDED.updated_at WHERE ai_agent_resource_user_state.version = $15 RETURNING id, uuid, tenant_id, organization_id, user_id, resource_type, resource_id, pinned_at::text AS pinned_at, hidden_at::text AS hidden_at, last_opened_at::text AS last_opened_at, last_read_message_sequence, custom_title, version, created_at::text AS created_at, updated_at::text AS updated_at";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_RESOURCE_USER_STATE: &str =
    "SELECT id, uuid, tenant_id, organization_id, user_id, resource_type, resource_id, pinned_at::text AS pinned_at, hidden_at::text AS hidden_at, last_opened_at::text AS last_opened_at, last_read_message_sequence, custom_title, version, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_resource_user_state WHERE tenant_id = $1 AND organization_id = $2 AND user_id = $3 AND resource_type = $4 AND resource_id = $5 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_RESOURCE_USER_STATES: &str =
    "SELECT state.id, state.uuid, state.tenant_id, state.organization_id, state.user_id, state.resource_type, state.resource_id, state.pinned_at::text AS pinned_at, state.hidden_at::text AS hidden_at, state.last_opened_at::text AS last_opened_at, state.last_read_message_sequence, state.custom_title, state.version, state.created_at::text AS created_at, state.updated_at::text AS updated_at FROM ai_agent_resource_user_state AS state WHERE state.tenant_id = $1 AND state.organization_id = $2 AND state.user_id = $3 AND state.resource_type = $4 AND ($5::text IS NULL OR EXISTS (SELECT 1 FROM ai_agent_session AS session WHERE state.resource_type = 0 AND session.tenant_id = state.tenant_id AND session.organization_id = state.organization_id AND session.owner_user_id = state.user_id AND session.session_id = state.resource_id AND session.agent_id = $5 AND session.deleted_at IS NULL)) AND ($6::bool = false OR state.pinned_at IS NOT NULL) AND ($7::bool = true OR state.hidden_at IS NULL) ORDER BY state.pinned_at DESC NULLS LAST, state.last_opened_at DESC NULLS LAST, state.id DESC LIMIT $8 OFFSET $9";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_RESOURCE_USER_STATES: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_resource_user_state AS state WHERE state.tenant_id = $1 AND state.organization_id = $2 AND state.user_id = $3 AND state.resource_type = $4 AND ($5::text IS NULL OR EXISTS (SELECT 1 FROM ai_agent_session AS session WHERE state.resource_type = 0 AND session.tenant_id = state.tenant_id AND session.organization_id = state.organization_id AND session.owner_user_id = state.user_id AND session.session_id = state.resource_id AND session.agent_id = $5 AND session.deleted_at IS NULL)) AND ($6::bool = false OR state.pinned_at IS NOT NULL) AND ($7::bool = true OR state.hidden_at IS NULL)";

#[cfg(feature = "postgres-sync")]
pub const SQL_UPSERT_AGENT_MESSAGE_FEEDBACK: &str =
    "INSERT INTO ai_agent_message_feedback (id, uuid, tenant_id, organization_id, message_id, user_id, rating, reason_code, comment, version, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, $10::timestamptz, $11::timestamptz, $12::timestamptz) ON CONFLICT (tenant_id, organization_id, message_id, user_id) DO UPDATE SET rating = EXCLUDED.rating, reason_code = EXCLUDED.reason_code, comment = EXCLUDED.comment, version = ai_agent_message_feedback.version + 1, updated_at = EXCLUDED.updated_at, deleted_at = EXCLUDED.deleted_at WHERE ai_agent_message_feedback.version = $13 OR (ai_agent_message_feedback.deleted_at IS NOT NULL AND $13 = -1 AND EXCLUDED.deleted_at IS NULL) RETURNING id, uuid, tenant_id, organization_id, message_id, user_id, rating, reason_code, comment, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_MESSAGE_FEEDBACK: &str =
    "SELECT id, uuid, tenant_id, organization_id, message_id, user_id, rating, reason_code, comment, version, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at FROM ai_agent_message_feedback WHERE tenant_id = $1 AND organization_id = $2 AND message_id = $3 AND user_id = $4 AND ($5::bool = TRUE OR deleted_at IS NULL) LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGE_FEEDBACK: &str =
    "SELECT feedback.id, feedback.uuid, feedback.tenant_id, feedback.organization_id, feedback.message_id, feedback.user_id, feedback.rating, feedback.reason_code, feedback.comment, feedback.version, feedback.created_at::text AS created_at, feedback.updated_at::text AS updated_at, feedback.deleted_at::text AS deleted_at FROM ai_agent_message_feedback AS feedback INNER JOIN ai_agent_message AS message ON message.tenant_id = feedback.tenant_id AND message.organization_id = feedback.organization_id AND message.message_id = feedback.message_id WHERE feedback.tenant_id = $1 AND feedback.organization_id = $2 AND feedback.user_id = $3 AND message.session_id = $4 AND message.deleted_at IS NULL AND feedback.deleted_at IS NULL ORDER BY message.sequence ASC, feedback.id ASC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_MESSAGE_FEEDBACK: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_message_feedback AS feedback INNER JOIN ai_agent_message AS message ON message.tenant_id = feedback.tenant_id AND message.organization_id = feedback.organization_id AND message.message_id = feedback.message_id WHERE feedback.tenant_id = $1 AND feedback.organization_id = $2 AND feedback.user_id = $3 AND message.session_id = $4 AND message.deleted_at IS NULL AND feedback.deleted_at IS NULL";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_MESSAGE_DRIVE_REF: &str =
    "INSERT INTO ai_agent_message_drive_ref (id, uuid, tenant_id, organization_id, message_id, media_role, drive_space_id, drive_node_id, drive_uri, media_resource_id, object_blob_id, resource_snapshot, resource_hash, alt_text, sort_order, status, created_by, created_at, updated_at, deleted_at, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::jsonb, $13, $14, $15, $16, $17, $18::timestamptz, $19::timestamptz, $20::timestamptz, $21::timestamptz)";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGE_DRIVE_REFS: &str =
    "SELECT id, uuid, tenant_id, organization_id, message_id, media_role, drive_space_id, drive_node_id, drive_uri, media_resource_id, object_blob_id, resource_snapshot::text AS resource_snapshot_json, resource_hash, alt_text, sort_order, status, created_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, retention_until::text AS retention_until FROM ai_agent_message_drive_ref WHERE tenant_id = $1 AND organization_id = $2 AND message_id = $3 AND status = 0 AND deleted_at IS NULL ORDER BY sort_order ASC, id ASC LIMIT 200";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGE_DRIVE_REFS_BATCH: &str =
    "SELECT id, uuid, tenant_id, organization_id, message_id, media_role, drive_space_id, drive_node_id, drive_uri, media_resource_id, object_blob_id, resource_snapshot::text AS resource_snapshot_json, resource_hash, alt_text, sort_order, status, created_by, created_at::text AS created_at, updated_at::text AS updated_at, deleted_at::text AS deleted_at, retention_until::text AS retention_until FROM ai_agent_message_drive_ref WHERE tenant_id = $1 AND organization_id = $2 AND message_id = ANY($3::text[]) AND status = 0 AND deleted_at IS NULL ORDER BY message_id ASC, sort_order ASC, id ASC";

// Message SQL constants (postgres-sync only)
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_CHAT_TURN: &str =
    "INSERT INTO ai_agent_chat_turn (id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, client_request_id, idempotency_key, payload_hash, request_message_id, response_message_id, mode, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, cached_tokens, finish_reason, error_code, error_detail, trace_id, version, created_at, updated_at, started_at, completed_at, cancel_requested_at, cancelled_at, retention_until) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 0, $14, $15, $16, $17, $18, $19, $20, 0, $21, $22, $23, $24, $25, $26::timestamptz, $27::timestamptz, $28::timestamptz, $29::timestamptz, $30::timestamptz, $31::timestamptz, $32::timestamptz) ON CONFLICT (tenant_id, organization_id, turn_id) DO UPDATE SET response_message_id = EXCLUDED.response_message_id, status = EXCLUDED.status, requested_model_id = EXCLUDED.requested_model_id, provider_binding_id = EXCLUDED.provider_binding_id, model_id = EXCLUDED.model_id, provider_id = EXCLUDED.provider_id, input_tokens = EXCLUDED.input_tokens, output_tokens = EXCLUDED.output_tokens, finish_reason = EXCLUDED.finish_reason, error_code = EXCLUDED.error_code, error_detail = EXCLUDED.error_detail, trace_id = EXCLUDED.trace_id, version = EXCLUDED.version, updated_at = EXCLUDED.updated_at, started_at = EXCLUDED.started_at, completed_at = EXCLUDED.completed_at, cancel_requested_at = EXCLUDED.cancel_requested_at, cancelled_at = EXCLUDED.cancelled_at, retention_until = EXCLUDED.retention_until WHERE ai_agent_chat_turn.payload_hash = EXCLUDED.payload_hash AND ai_agent_chat_turn.status IN (0, 1) AND EXCLUDED.version = ai_agent_chat_turn.version + 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_CHAT_TURN_BY_IDEMPOTENCY: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, client_request_id, idempotency_key, payload_hash, request_message_id, response_message_id, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, finish_reason, error_code, error_detail, trace_id, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until FROM ai_agent_chat_turn WHERE tenant_id = $1 AND organization_id = $2 AND owner_user_id = $3 AND idempotency_key = $4 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_CHAT_TURN: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, client_request_id, idempotency_key, payload_hash, request_message_id, response_message_id, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, finish_reason, error_code, error_detail, trace_id, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until FROM ai_agent_chat_turn WHERE tenant_id = $1 AND organization_id = $2 AND turn_id = $3 LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_RECONCILABLE_AGENT_CHAT_TURNS: &str =
    "SELECT id, uuid, tenant_id, organization_id, turn_id, session_id, agent_id, owner_user_id, client_request_id, idempotency_key, payload_hash, request_message_id, response_message_id, status, requested_model_id, provider_binding_id, model_id, provider_id, input_tokens, output_tokens, finish_reason, error_code, error_detail, trace_id, version, created_at::text AS created_at, updated_at::text AS updated_at, started_at::text AS started_at, completed_at::text AS completed_at, cancel_requested_at::text AS cancel_requested_at, cancelled_at::text AS cancelled_at, retention_until::text AS retention_until FROM ai_agent_chat_turn WHERE status IN (0, 1) AND updated_at < $1::timestamptz ORDER BY updated_at ASC, id ASC LIMIT $2";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_CHAT_TURN_STATE: &str =
    "UPDATE ai_agent_chat_turn SET response_message_id = $1, status = $2, requested_model_id = $3, provider_binding_id = $4, model_id = $5, provider_id = $6, input_tokens = $7, output_tokens = $8, finish_reason = $9, error_code = $10, error_detail = $11, trace_id = $12, version = $13, updated_at = $14::timestamptz, started_at = $15::timestamptz, completed_at = $16::timestamptz, cancel_requested_at = $17::timestamptz, cancelled_at = $18::timestamptz, retention_until = $19::timestamptz WHERE tenant_id = $20 AND organization_id = $21 AND turn_id = $22 AND version = $23";
#[cfg(feature = "postgres-sync")]
pub const SQL_INSERT_AGENT_MESSAGE: &str =
    "INSERT INTO ai_agent_message (id, uuid, tenant_id, organization_id, session_id, agent_id, owner_user_id, sender_type, sender_user_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json, metadata_json, parent_message_id, turn_id, created_by, created_at, updated_at) SELECT $1, $2, $3, session.organization_id, $4, $5, session.owner_user_id, $6, CASE WHEN $6 = 0 THEN session.owner_user_id ELSE NULL END, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16::jsonb, $17::jsonb, $18, $21, session.owner_user_id, $19::timestamptz, $20::timestamptz FROM ai_agent_session AS session WHERE session.tenant_id = $3 AND session.session_id = $4 AND session.deleted_at IS NULL";
#[cfg(feature = "postgres-sync")]
pub const SQL_UPDATE_AGENT_MESSAGE: &str =
    "UPDATE ai_agent_message SET content = $1, content_type = $2, status = $3, model_id = $4, provider_id = $5, artifacts_json = $6::jsonb, metadata_json = $7::jsonb, updated_at = $8::timestamptz WHERE tenant_id = $9 AND session_id = $10 AND message_id = $11 AND deleted_at IS NULL";
#[cfg(feature = "postgres-sync")]
pub const SQL_SELECT_AGENT_MESSAGE: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json::text AS artifacts_json, metadata_json::text AS metadata_json, parent_message_id, turn_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND message_id = $3 AND deleted_at IS NULL LIMIT 1";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGES: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json::text AS artifacts_json, metadata_json::text AS metadata_json, parent_message_id, turn_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND deleted_at IS NULL AND ($3::smallint IS NULL OR role = $3) AND ($4::smallint IS NULL OR status = $4) ORDER BY sequence ASC, id ASC LIMIT $5 OFFSET $6";
#[cfg(feature = "postgres-sync")]
pub const SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT: &str =
    "SELECT id, uuid, tenant_id, session_id, agent_id, role, message_id, content, content_type, status, sequence, input_tokens, output_tokens, model_id, provider_id, artifacts_json::text AS artifacts_json, metadata_json::text AS metadata_json, parent_message_id, turn_id, created_at::text AS created_at, updated_at::text AS updated_at FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND deleted_at IS NULL AND ($3::smallint IS NULL OR role = $3) AND ($4::smallint IS NULL OR status = $4) ORDER BY sequence DESC, id DESC LIMIT $5";
#[cfg(feature = "postgres-sync")]
pub const SQL_COUNT_AGENT_MESSAGES: &str =
    "SELECT COUNT(*)::bigint AS total_count FROM ai_agent_message WHERE tenant_id = $1 AND session_id = $2 AND deleted_at IS NULL AND ($3::smallint IS NULL OR role = $3) AND ($4::smallint IS NULL OR status = $4)";
#[cfg(feature = "postgres-sync")]
pub const SQL_NEXT_MESSAGE_SEQUENCE: &str =
    "SELECT last_message_sequence + 1 AS next_sequence FROM ai_agent_session WHERE tenant_id = $1 AND session_id = $2 AND deleted_at IS NULL";
#[cfg(feature = "postgres-sync")]
pub const SQL_LOCK_AGENT_SESSION_FOR_UPDATE: &str =
    "SELECT session_id FROM ai_agent_session WHERE tenant_id = $1 AND session_id = $2 AND organization_id = $3 AND deleted_at IS NULL FOR UPDATE";

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
