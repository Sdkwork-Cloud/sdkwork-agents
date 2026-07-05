//! MCP marketplace projection from agent composition slots.
//!
//! MCP server persistence is owned by `sdkwork-mcp`. Agents exposes only the
//! composition-plane references (`slot_kind = mcp`) for marketplace discovery.
//!
//! The projection is exposed as a list payload under
//! `SdkWorkPageData<McpServerMarketplaceRecord>` per `API_SPEC.md` §16.

use crate::domain::{AgentCompositionSlotKind, AgentCompositionSlotRecord};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerMarketplaceRecord {
    pub agent_id: String,
    pub slot_id: String,
    pub server_id: String,
    pub target_module: String,
    pub target_ref: String,
    pub target_version_ref: Option<String>,
    pub enabled: bool,
    pub priority: i32,
}

pub fn project_mcp_slot(slot: &AgentCompositionSlotRecord) -> McpServerMarketplaceRecord {
    McpServerMarketplaceRecord {
        agent_id: slot.agent_id.clone(),
        slot_id: slot.slot_id.clone(),
        server_id: slot.target_ref.clone(),
        target_module: slot.target_module.as_str().to_string(),
        target_ref: slot.target_ref.clone(),
        target_version_ref: slot.target_version_ref.clone(),
        enabled: slot.enabled,
        priority: slot.priority,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AgentBusinessStatus, AgentCompositionTargetModule};

    #[test]
    fn projects_mcp_slot_to_marketplace_record() {
        let slot = AgentCompositionSlotRecord {
            id: 1,
            tenant_id: 100_001,
            organization_id: 0,
            agent_id: "agent.demo".to_string(),
            slot_id: "slot.mcp.agent.tools".to_string(),
            slot_kind: AgentCompositionSlotKind::Mcp,
            target_module: AgentCompositionTargetModule::Mcp,
            target_ref: "ai_mcp_server.toolset".to_string(),
            target_version_ref: None,
            priority: 0,
            enabled: true,
            policy_json: "{}".to_string(),
            status: AgentBusinessStatus::Active,
            version: 1,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            deleted_at: None,
        };

        let record = project_mcp_slot(&slot);
        assert_eq!(record.agent_id, "agent.demo");
        assert_eq!(record.server_id, "ai_mcp_server.toolset");
    }
}
