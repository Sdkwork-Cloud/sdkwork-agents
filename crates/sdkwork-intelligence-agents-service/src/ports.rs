use crate::domain::{
    AgentBusinessRecord, AgentBusinessStatus, AgentDeploymentRecord, AgentMcpServerRecord,
    AgentProviderBindingRecord,
    AgentVisibility,
};
use sdkwork_agent_kernel::{KernelError, KernelEvent, KernelResult};
use crate::validation::optional_non_blank;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentListQuery {
    pub tenant_id: u64,
    pub organization_id: Option<u64>,
    pub owner_user_id: Option<u64>,
    pub include_deleted: bool,
    pub search_query: Option<String>,
}

impl AgentListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id: None,
            owner_user_id: None,
            include_deleted: false,
            search_query: None,
        }
    }

    pub fn for_organization(mut self, organization_id: u64) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        let query = query.into();
        self.search_query = optional_non_blank(query);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMarketplaceListQuery {
    pub tenant_id: u64,
    pub organization_id: Option<u64>,
    pub owner_user_id: Option<u64>,
    pub status: Option<AgentBusinessStatus>,
    pub visibility: Option<AgentVisibility>,
    pub include_deleted: bool,
    pub search_query: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
}

impl AgentMarketplaceListQuery {
    pub fn for_tenant(tenant_id: u64) -> Self {
        Self {
            tenant_id,
            organization_id: None,
            owner_user_id: None,
            status: None,
            visibility: None,
            include_deleted: false,
            search_query: None,
            category: None,
            tag: None,
        }
    }

    pub fn for_organization(mut self, organization_id: u64) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn for_owner(mut self, owner_user_id: u64) -> Self {
        self.owner_user_id = Some(owner_user_id);
        self
    }

    pub fn with_status(mut self, status: AgentBusinessStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_visibility(mut self, visibility: AgentVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }

    pub fn with_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    pub fn with_search(mut self, query: impl Into<String>) -> Self {
        let query = query.into();
        self.search_query = optional_non_blank(query);
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        let category = category.into();
        self.category = optional_non_blank(category);
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        self.tag = optional_non_blank(tag);
        self
    }
}

pub trait AgentRepository {
    fn next_id(&mut self) -> KernelResult<u64>;

    fn insert(&mut self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn update(&mut self, record: AgentBusinessRecord) -> KernelResult<()>;

    fn get(&self, tenant_id: u64, agent_id: &str) -> Option<AgentBusinessRecord>;

    fn list(&self, query: &AgentListQuery) -> Vec<AgentBusinessRecord>;

    fn insert_provider_binding(&mut self, _record: AgentProviderBindingRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.provider_binding".to_string(),
        })
    }

    fn update_provider_binding(&mut self, _record: AgentProviderBindingRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.provider_binding".to_string(),
        })
    }

    fn get_provider_binding(
        &self,
        _tenant_id: u64,
        _agent_id: &str,
        _binding_id: &str) -> Option<AgentProviderBindingRecord> {
        None
    }

    fn list_provider_bindings(
        &self,
        _tenant_id: u64,
        _agent_id: &str) -> Vec<AgentProviderBindingRecord> {
        Vec::new()
    }

    fn insert_deployment(&mut self, _record: AgentDeploymentRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.deployment".to_string(),
        })
    }

    fn list_deployments(&self, _tenant_id: u64, _agent_id: &str) -> Vec<AgentDeploymentRecord> {
        Vec::new()
    }

    fn insert_mcp_server(&mut self, _record: AgentMcpServerRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.mcp".to_string(),
        })
    }

    fn update_mcp_server(&mut self, _record: AgentMcpServerRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.mcp".to_string(),
        })
    }

    fn get_mcp_server(
        &self,
        _tenant_id: u64,
        _mcp_server_id: &str) -> Option<AgentMcpServerRecord> {
        None
    }

    fn list_mcp_servers(&self, _query: &AgentMarketplaceListQuery) -> Vec<AgentMcpServerRecord> {
        Vec::new()
    }


    fn insert_composition_slot(&mut self, _record: AgentCompositionSlotRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.composition_slot".to_string(),
        })
    }

    fn update_composition_slot(&mut self, _record: AgentCompositionSlotRecord) -> KernelResult<()> {
        Err(KernelError::CapabilityMissing {
            capability_id: "agent.business.composition_slot".to_string(),
        })
    }

    fn get_composition_slot(
        &self,
        _tenant_id: u64,
        _agent_id: &str,
        _slot_id: &str) -> Option<AgentCompositionSlotRecord> {
        None
    }

    fn list_composition_slots(
        &self,
        _tenant_id: u64,
        _agent_id: &str) -> Vec<AgentCompositionSlotRecord> {
        Vec::new()
    }

}

pub trait AgentAuditSink {
    fn record(&mut self, event: KernelEvent) -> KernelResult<()>;

    fn list_events(&self, _tenant_id: u64, _agent_id: &str) -> KernelResult<Vec<KernelEvent>> {
        Ok(Vec::new())
    }
}
