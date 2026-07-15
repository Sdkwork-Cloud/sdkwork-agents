//! SQLite SQL authority for the agents managed-store adapter.

pub const SELECT_AGENT: &str = "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at, updated_at, deleted_at, version FROM ai_agent WHERE tenant_id = ? AND agent_id = ? LIMIT 1";
pub const INSERT_AGENT: &str = "INSERT INTO ai_agent (id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at, updated_at, deleted_at, version) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
pub const UPDATE_AGENT: &str = "UPDATE ai_agent SET organization_id = ?, owner_user_id = ?, code = ?, display_name = ?, description = ?, manifest_json = ?, default_code_task_intent_json = ?, implementation_provider_id = ?, implementation_kind = ?, implementation_type = ?, status = ?, visibility = ?, tags_json = ?, updated_at = ?, deleted_at = ?, version = ? WHERE tenant_id = ? AND agent_id = ? AND version = ?";
pub const LIST_AGENTS: &str = "SELECT id, uuid, tenant_id, organization_id, owner_user_id, agent_id, code, display_name, description, manifest_json, default_code_task_intent_json, implementation_provider_id, implementation_kind, implementation_type, status, visibility, tags_json, created_at, updated_at, deleted_at, version FROM ai_agent WHERE tenant_id = ? AND (? IS NULL OR organization_id = ?) AND (? IS NULL OR owner_user_id = ?) AND (deleted_at IS NULL OR ? = 1) AND (? IS NULL OR LOWER(agent_id) LIKE LOWER(?) OR LOWER(code) LIKE LOWER(?) OR LOWER(display_name) LIKE LOWER(?) OR LOWER(COALESCE(description, '')) LIKE LOWER(?)) AND (? IS NULL OR visibility = ?) ORDER BY updated_at DESC, id DESC LIMIT ? OFFSET ?";
pub const COUNT_AGENTS: &str = "SELECT COUNT(*) AS total_count FROM ai_agent WHERE tenant_id = ? AND (? IS NULL OR organization_id = ?) AND (? IS NULL OR owner_user_id = ?) AND (deleted_at IS NULL OR ? = 1) AND (? IS NULL OR LOWER(agent_id) LIKE LOWER(?) OR LOWER(code) LIKE LOWER(?) OR LOWER(display_name) LIKE LOWER(?) OR LOWER(COALESCE(description, '')) LIKE LOWER(?)) AND (? IS NULL OR visibility = ?)";

pub const SELECT_BINDING: &str = "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at FROM ai_agent_runtime_binding WHERE tenant_id = ? AND agent_id = ? AND binding_id = ? LIMIT 1";
pub const SELECT_ACTIVE_BINDING: &str = "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at FROM ai_agent_runtime_binding WHERE tenant_id = ? AND agent_id = ? AND active = 1 LIMIT 1";
pub const INSERT_BINDING: &str = "INSERT INTO ai_agent_runtime_binding (id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
pub const UPDATE_BINDING: &str = "UPDATE ai_agent_runtime_binding SET provider_id = ?, implementation_kind = ?, configuration_profile_id = ?, capabilities_json = ?, active = ?, version = ?, updated_at = ? WHERE tenant_id = ? AND agent_id = ? AND binding_id = ? AND version = ?";
pub const LIST_BINDINGS: &str = "SELECT id, uuid, tenant_id, agent_id, binding_id, provider_id, implementation_kind, configuration_profile_id, capabilities_json, active, version, created_at, updated_at FROM ai_agent_runtime_binding WHERE tenant_id = ? AND agent_id = ? ORDER BY active DESC, updated_at DESC, binding_id ASC LIMIT ? OFFSET ?";
pub const COUNT_BINDINGS: &str = "SELECT COUNT(*) AS total_count FROM ai_agent_runtime_binding WHERE tenant_id = ? AND agent_id = ?";
pub const DEACTIVATE_BINDINGS: &str = "UPDATE ai_agent_runtime_binding SET active = 0, version = version + 1, updated_at = ? WHERE tenant_id = ? AND agent_id = ? AND active = 1";
pub const ACTIVATE_BINDING: &str = "UPDATE ai_agent_runtime_binding SET active = 1, version = ?, updated_at = ? WHERE tenant_id = ? AND agent_id = ? AND binding_id = ? AND version = ?";

pub const SELECT_SLOT: &str = "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, status, version, created_at, updated_at, deleted_at FROM ai_agent_composition_slot WHERE tenant_id = ? AND agent_id = ? AND slot_id = ? LIMIT 1";
pub const INSERT_SLOT: &str = "INSERT INTO ai_agent_composition_slot (id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, status, version, created_at, updated_at, deleted_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
pub const UPDATE_SLOT: &str = "UPDATE ai_agent_composition_slot SET organization_id = ?, slot_kind = ?, target_module = ?, target_ref = ?, target_version_ref = ?, priority = ?, enabled = ?, policy_json = ?, status = ?, version = ?, updated_at = ?, deleted_at = ? WHERE tenant_id = ? AND agent_id = ? AND slot_id = ? AND version = ?";
pub const LIST_SLOTS: &str = "SELECT id, uuid, tenant_id, organization_id, agent_id, slot_id, slot_kind, target_module, target_ref, target_version_ref, priority, enabled, policy_json, status, version, created_at, updated_at, deleted_at FROM ai_agent_composition_slot WHERE tenant_id = ? AND agent_id = ? AND deleted_at IS NULL ORDER BY priority ASC, slot_id ASC LIMIT ? OFFSET ?";
pub const COUNT_SLOTS: &str = "SELECT COUNT(*) AS total_count FROM ai_agent_composition_slot WHERE tenant_id = ? AND agent_id = ? AND deleted_at IS NULL";

pub const INSERT_AUDIT: &str = "INSERT INTO ai_agent_audit_event (id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
pub const LIST_AUDIT: &str = "SELECT id, uuid, tenant_id, organization_id, agent_internal_id, agent_id, action, subject_id, subject_tenant_id, request_id, trace_id, payload_json, created_at FROM ai_agent_audit_event WHERE tenant_id = ? AND agent_id = ? AND (? IS NULL OR action = ?) AND (? IS NULL OR created_at >= ?) AND (? IS NULL OR created_at <= ?) ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?";
pub const COUNT_AUDIT: &str = "SELECT COUNT(*) AS total_count FROM ai_agent_audit_event WHERE tenant_id = ? AND agent_id = ? AND (? IS NULL OR action = ?) AND (? IS NULL OR created_at >= ?) AND (? IS NULL OR created_at <= ?)";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_queries_are_parameterized_and_dialect_clean() {
        for query in [
            SELECT_AGENT,
            INSERT_AGENT,
            UPDATE_AGENT,
            LIST_AGENTS,
            COUNT_AGENTS,
            SELECT_BINDING,
            SELECT_ACTIVE_BINDING,
            INSERT_BINDING,
            UPDATE_BINDING,
            LIST_BINDINGS,
            COUNT_BINDINGS,
            DEACTIVATE_BINDINGS,
            ACTIVATE_BINDING,
            SELECT_SLOT,
            INSERT_SLOT,
            UPDATE_SLOT,
            LIST_SLOTS,
            COUNT_SLOTS,
            INSERT_AUDIT,
            LIST_AUDIT,
            COUNT_AUDIT,
        ] {
            assert!(query.contains('?'));
            assert!(!query.contains("$1"));
            assert!(!query.contains("::"));
            assert!(!query.contains("ILIKE"));
        }
    }

    #[test]
    fn all_list_queries_apply_store_level_limits() {
        for query in [LIST_AGENTS, LIST_BINDINGS, LIST_SLOTS, LIST_AUDIT] {
            assert!(query.contains("LIMIT ?"));
            assert!(query.contains("OFFSET ?"));
        }
    }
}
