use crate::domain::{
    AgentBusinessRecord, AgentDeploymentRecord, AgentMcpServerRecord, AgentMemoryRecord, AgentProviderBindingRecord,
};
use crate::id::{AgentBusinessIdGenerator, AgentIdGenerator};
use crate::ports::{AgentAuditSink, AgentListQuery, AgentMarketplaceListQuery, AgentRepository};
use sdkwork_agent_kernel::{
    KernelError, KernelEvent, KernelResult, PolicyDecision, PolicyProvider, PolicyRequest,
};
use std::cmp::Ordering;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct InMemoryAgentRepository {
    id_generator: AgentBusinessIdGenerator,
    records: Vec<AgentBusinessRecord>,
    provider_bindings: Vec<AgentProviderBindingRecord>,
    deployments: Vec<AgentDeploymentRecord>,
    mcp_servers: Vec<AgentMcpServerRecord>,
    composition_slots: Vec<AgentCompositionSlotRecord>,
}

impl InMemoryAgentRepository {
    pub fn new() -> Self {
        Self {
            id_generator: AgentBusinessIdGenerator::new_default()
                .expect("default agents managed store snowflake node id is valid"),
            records: Vec::new(),
            provider_bindings: Vec::new(),
            deployments: Vec::new(),
            mcp_servers: Vec::new(),
            composition_slots: Vec::new(),
        }
    }

    pub fn records(&self) -> &[AgentBusinessRecord] {
        &self.records
    }
}

impl Default for InMemoryAgentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRepository for InMemoryAgentRepository {
    fn next_id(&mut self) -> KernelResult<u64> {
        self.id_generator.next_id()
    }

    fn insert(&mut self, record: AgentBusinessRecord) -> KernelResult<()> {
        if self.records.iter().any(|existing| {
            existing.tenant_id == record.tenant_id && existing.agent_id == record.agent_id
        }) {
            return Err(KernelError::conflict("agent already exists"));
        }
        if self
            .records
            .iter()
            .any(|existing| existing.tenant_id == record.tenant_id && existing.code == record.code)
        {
            return Err(KernelError::conflict("agent code already exists"));
        }
        self.records.push(record);
        Ok(())
    }

    fn update(&mut self, record: AgentBusinessRecord) -> KernelResult<()> {
        let Some(index) = self.records.iter().position(|existing| {
            existing.tenant_id == record.tenant_id && existing.agent_id == record.agent_id
        }) else {
            return Err(KernelError::validation("agent not found"));
        };
        let expected_version = self.records[index].version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "agent version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if self.records.iter().enumerate().any(|(current, existing)| {
            current != index
                && existing.tenant_id == record.tenant_id
                && existing.code == record.code
        }) {
            return Err(KernelError::conflict("agent code already exists"));
        }
        self.records[index] = record;
        Ok(())
    }

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRecord> {
        self.records
            .iter()
            .find(|record| record.tenant_id == tenant_id && record.agent_id == agent_id)
            .cloned()
    }

    fn list(&self, query: &AgentListQuery) -> Vec<AgentBusinessRecord> {
        self.records
            .iter()
            .filter(|record| record.tenant_id == query.tenant_id)
            .filter(|record| {
                if let Some(organization_id) = query.organization_id {
                    record.organization_id == organization_id
                } else {
                    true
                }
            })
            .filter(|record| {
                if let Some(owner_user_id) = query.owner_user_id {
                    record.owner_user_id == owner_user_id
                } else {
                    true
                }
            })
            .filter(|record| query.include_deleted || !record.is_deleted())
            .filter(|record| {
                let Some(search_query) = query.search_query.as_ref() else {
                    return true;
                };
                let normalized_query = search_query.trim().to_lowercase();
                if normalized_query.is_empty() {
                    return true;
                }

                let description = record.description.as_deref().unwrap_or("");
                record
                    .agent_id
                    .to_lowercase()
                    .contains(normalized_query.as_str())
                    || record
                        .code
                        .to_lowercase()
                        .contains(normalized_query.as_str())
                    || record
                        .display_name
                        .to_lowercase()
                        .contains(normalized_query.as_str())
                    || description
                        .to_lowercase()
                        .contains(normalized_query.as_str())
            })
            .cloned()
            .collect()
    }

    fn insert_provider_binding(&mut self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        if self.provider_bindings.iter().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.agent_id == record.agent_id
                && existing.binding_id == record.binding_id
        }) {
            return Err(KernelError::conflict(
                "agent provider binding already exists"));
        }
        if record.active
            && self.provider_bindings.iter().any(|existing| {
                existing.tenant_id == record.tenant_id
                    && existing.agent_id == record.agent_id
                    && existing.active
            })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists"));
        }
        self.provider_bindings.push(record);
        Ok(())
    }

    fn update_provider_binding(&mut self, record: AgentProviderBindingRecord) -> KernelResult<()> {
        let Some(index) = self.provider_bindings.iter().position(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.agent_id == record.agent_id
                && existing.binding_id == record.binding_id
        }) else {
            return Err(KernelError::validation("agent provider binding not found"));
        };
        let expected_version = self.provider_bindings[index].version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "provider binding version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if record.active
            && self
                .provider_bindings
                .iter()
                .enumerate()
                .any(|(current, existing)| {
                    current != index
                        && existing.tenant_id == record.tenant_id
                        && existing.agent_id == record.agent_id
                        && existing.active
                })
        {
            return Err(KernelError::conflict(
                "active provider binding already exists"));
        }
        self.provider_bindings[index] = record;
        Ok(())
    }

    fn get_provider_binding(
        &self,
        tenant_id: u64,
        agent_id: &str,
        binding_id: &str) -> Option<AgentProviderBindingRecord> {
        self.provider_bindings
            .iter()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.agent_id == agent_id
                    && record.binding_id == binding_id
            })
            .cloned()
    }

    fn list_provider_bindings(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentProviderBindingRecord> {
        let mut records: Vec<AgentProviderBindingRecord> = self
            .provider_bindings
            .iter()
            .filter(|record| record.tenant_id == tenant_id && record.agent_id == agent_id)
            .cloned()
            .collect();
        records.sort_by(compare_provider_bindings_standard_order);
        records
    }

    fn insert_deployment(&mut self, record: AgentDeploymentRecord) -> KernelResult<()> {
        if self.deployments.iter().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.agent_id == record.agent_id
                && existing.deployment_id == record.deployment_id
        }) {
            return Err(KernelError::conflict("agent deployment already exists"));
        }
        self.deployments.push(record);
        Ok(())
    }

    fn list_deployments(&self, tenant_id: u64, agent_id: &str) -> Vec<AgentDeploymentRecord> {
        let mut records: Vec<AgentDeploymentRecord> = self
            .deployments
            .iter()
            .filter(|record| record.tenant_id == tenant_id && record.agent_id == agent_id)
            .cloned()
            .collect();
        records.sort_by(compare_deployments_standard_order);
        records
    }

    fn insert_mcp_server(&mut self, record: AgentMcpServerRecord) -> KernelResult<()> {
        if self.mcp_servers.iter().any(|existing| {
            existing.tenant_id == record.tenant_id && existing.mcp_server_id == record.mcp_server_id
        }) {
            return Err(KernelError::conflict("agent mcp server already exists"));
        }
        if self
            .mcp_servers
            .iter()
            .any(|existing| existing.tenant_id == record.tenant_id && existing.code == record.code)
        {
            return Err(KernelError::conflict(
                "agent mcp server code already exists"));
        }
        self.mcp_servers.push(record);
        Ok(())
    }

    fn update_mcp_server(&mut self, record: AgentMcpServerRecord) -> KernelResult<()> {
        let Some(index) = self.mcp_servers.iter().position(|existing| {
            existing.tenant_id == record.tenant_id && existing.mcp_server_id == record.mcp_server_id
        }) else {
            return Err(KernelError::validation("agent mcp server not found"));
        };
        let expected_version = self.mcp_servers[index].version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "agent mcp server version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        if self
            .mcp_servers
            .iter()
            .enumerate()
            .any(|(current, existing)| {
                current != index
                    && existing.tenant_id == record.tenant_id
                    && existing.code == record.code
            })
        {
            return Err(KernelError::conflict(
                "agent mcp server code already exists"));
        }
        self.mcp_servers[index] = record;
        Ok(())
    }

    fn get_mcp_server(&self, tenant_id: u64, mcp_server_id: &str) -> Option<AgentMcpServerRecord> {
        self.mcp_servers
            .iter()
            .find(|record| record.tenant_id == tenant_id && record.mcp_server_id == mcp_server_id)
            .cloned()
    }

    fn list_mcp_servers(&self, query: &AgentMarketplaceListQuery) -> Vec<AgentMcpServerRecord> {
        let mut records: Vec<AgentMcpServerRecord> = self
            .mcp_servers
            .iter()
            .filter(|record| {
                marketplace_record_matches(
                    query,
                    MarketplaceRecordView {
                        tenant_id: record.tenant_id,
                        organization_id: record.organization_id,
                        owner_user_id: record.owner_user_id,
                        status: record.status,
                        visibility: record.visibility,
                        deleted: record.is_deleted(),
                        id: record.mcp_server_id.as_str(),
                        code: record.code.as_str(),
                        display_name: record.display_name.as_str(),
                        description: record.description.as_deref(),
                        categories: record.categories.as_slice(),
                        tags: record.tags.as_slice(),
                    })
            })
            .cloned()
            .collect();
        records.sort_by(|left, right| {
            compare_marketplace_standard_order(
                left.updated_at.as_str(),
                left.code.as_str(),
                right.updated_at.as_str(),
                right.code.as_str())
        });
        records
    }


    fn insert_composition_slot(&mut self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        if self.composition_slots.iter().any(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.agent_id == record.agent_id
                && existing.slot_id == record.slot_id
        }) {
            return Err(KernelError::conflict("composition slot already exists"));
        }
        self.composition_slots.push(record);
        Ok(())
    }

    fn update_composition_slot(&mut self, record: AgentCompositionSlotRecord) -> KernelResult<()> {
        let Some(index) = self.composition_slots.iter().position(|existing| {
            existing.tenant_id == record.tenant_id
                && existing.agent_id == record.agent_id
                && existing.slot_id == record.slot_id
        }) else {
            return Err(KernelError::validation("composition slot not found"));
        };
        let expected_version = self.composition_slots[index].version.saturating_add(1);
        if record.version != expected_version {
            return Err(KernelError::conflict(format!(
                "composition slot version mismatch: expected={expected_version}, actual={}",
                record.version
            )));
        }
        self.composition_slots[index] = record;
        Ok(())
    }

    fn get_composition_slot(
        &self,
        tenant_id: u64,
        agent_id: &str,
        slot_id: &str) -> Option<AgentCompositionSlotRecord> {
        self.composition_slots
            .iter()
            .find(|record| {
                record.tenant_id == tenant_id
                    && record.agent_id == agent_id
                    && record.slot_id == slot_id
            })
            .cloned()
    }

    fn list_composition_slots(
        &self,
        tenant_id: u64,
        agent_id: &str) -> Vec<AgentCompositionSlotRecord> {
        self.composition_slots
            .iter()
            .filter(|record| record.tenant_id == tenant_id && record.agent_id == agent_id)
            .cloned()
            .collect()
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        AgentBusinessStatus, AgentImplementationKind, AgentImplementationType,
        AgentProviderBindingRecord,
        AgentVisibility,
    };
    use sdkwork_agent_kernel::AgentManifest;
    use sdkwork_agent_kernel::KernelErrorKind;

    fn sample_manifest(agent_id: &str) -> AgentManifest {
        AgentManifest {
            schema_version: "1.0.0".to_string(),
            manifest_type: "agent".to_string(),
            agent_id: agent_id.to_string(),
            name: "sample-agent".to_string(),
            display_name: "Sample Agent".to_string(),
            description: "sample".to_string(),
            version: "0.1.0".to_string(),
            domain: "intelligence".to_string(),
            required_capabilities: vec!["model.chat".to_string()],
            optional_capabilities: vec!["tool.invoke".to_string()],
            required_capability_requirements: vec![],
            optional_capability_requirements: vec![],
            event_families: vec!["agent.lifecycle".to_string()],
            owner_name: "sdkwork".to_string(),
            status: "active".to_string(),
        }
    }

    #[test]
    fn in_memory_repository_rejects_stale_record_version_update() {
        let mut repository = InMemoryAgentRepository::new();
        let record = AgentBusinessRecord {
            id: 1,
            agent_id: "agent.alpha".to_string(),
            tenant_id: 100_001,
            organization_id: 0,
            owner_user_id: 100,
            code: "alpha".to_string(),
            display_name: "Alpha".to_string(),
            description: None,
            manifest: sample_manifest("agent.alpha"),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: AgentImplementationType::SdkworkNative,
            status: AgentBusinessStatus::Draft,
            visibility: AgentVisibility::Organization,
            tags: vec!["starter".to_string()],
            version: 1,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
            deleted_at: None,
        };
        repository
            .insert(record.clone())
            .expect("initial insert should succeed");

        let mut stale = record.clone();
        stale.display_name = "Alpha stale".to_string();
        let error = repository
            .update(stale)
            .expect_err("stale version should fail");
        match error {
            KernelError::Structured { info } => {
                assert_eq!(info.kind.as_str(), "conflict");
                assert!(info.message.contains("version mismatch"));
            }
            _ => panic!("expected structured conflict"),
        }
    }

    #[test]
    fn in_memory_repository_rejects_stale_provider_binding_update() {
        let mut repository = InMemoryAgentRepository::new();
        let record = AgentProviderBindingRecord {
            id: 101,
            tenant_id: 100_001,
            agent_id: "agent.alpha".to_string(),
            binding_id: "binding.rig.default".to_string(),
            provider_id: "provider.model.rig-rust".to_string(),
            implementation_kind: AgentImplementationKind::TypedLocalProvider,
            configuration_profile_id: "profile.rig.local".to_string(),
            capabilities: vec!["model.chat".to_string()],
            active: true,
            version: 1,
            created_at: "2026-06-01T00:00:00Z".to_string(),
            updated_at: "2026-06-01T00:00:00Z".to_string(),
        };
        repository
            .insert_provider_binding(record.clone())
            .expect("initial binding insert should succeed");

        let mut stale = record.clone();
        stale.provider_id = "provider.model.rig-alt".to_string();
        let error = repository
            .update_provider_binding(stale)
            .expect_err("stale binding version should fail");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("provider binding version mismatch"));
    }

    #[test]
    fn in_memory_repository_rejects_second_active_provider_binding() {
        let mut repository = InMemoryAgentRepository::new();
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 102,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.local".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:00:00Z".to_string(),
            })
            .expect("first active binding insert should succeed");

        let error = repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 103,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.model.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            })
            .expect_err("second active binding should fail");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("active provider binding already exists"));
    }

    #[test]
    fn in_memory_repository_rejects_update_that_creates_second_active_provider_binding() {
        let mut repository = InMemoryAgentRepository::new();
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 104,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.local".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:00:00Z".to_string(),
            })
            .expect("first active binding insert should succeed");
        repository
            .insert_provider_binding(AgentProviderBindingRecord {
                id: 105,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.model.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            })
            .expect("inactive binding insert should succeed");

        let error = repository
            .update_provider_binding(AgentProviderBindingRecord {
                id: 105,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alt".to_string(),
                provider_id: "provider.model.rig-alt".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alt".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 2,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            })
            .expect_err("update cannot create a second active binding");

        assert_eq!(error.kind(), KernelErrorKind::Conflict);
        assert!(error
            .message()
            .contains("active provider binding already exists"));
    }

    #[test]
    fn in_memory_repository_lists_provider_bindings_in_standard_order() {
        let mut repository = InMemoryAgentRepository::new();
        for record in [
            AgentProviderBindingRecord {
                id: 106,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.beta".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.beta".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            },
            AgentProviderBindingRecord {
                id: 107,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.default".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.default".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: true,
                version: 1,
                created_at: "2026-06-01T00:01:00Z".to_string(),
                updated_at: "2026-06-01T00:01:00Z".to_string(),
            },
            AgentProviderBindingRecord {
                id: 108,
                tenant_id: 100_001,
                agent_id: "agent.alpha".to_string(),
                binding_id: "binding.rig.alpha".to_string(),
                provider_id: "provider.model.rig-rust".to_string(),
                implementation_kind: AgentImplementationKind::TypedLocalProvider,
                configuration_profile_id: "profile.rig.alpha".to_string(),
                capabilities: vec!["model.chat".to_string()],
                active: false,
                version: 1,
                created_at: "2026-06-01T00:00:00Z".to_string(),
                updated_at: "2026-06-01T00:02:00Z".to_string(),
            },
        ] {
            repository
                .insert_provider_binding(record)
                .expect("binding insert should succeed");
        }

        let binding_ids: Vec<String> = repository
            .list_provider_bindings(100_001, "agent.alpha")
            .into_iter()
            .map(|record| record.binding_id)
            .collect();

        assert_eq!(
            binding_ids,
            vec![
                "binding.rig.default".to_string(),
                "binding.rig.alpha".to_string(),
                "binding.rig.beta".to_string()
            ]
        );
    }

}
