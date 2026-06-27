//! Production and development `AgentHttpState` bootstrap for SDKWork Agents.

use anyhow::{Context, Result};
use sdkwork_intelligence_agents_service::{
    AgentBusinessIdGenerator, AgentHttpState, AllowAllPolicyProvider, AUDIT_SINK_NODE_ID,
    IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    PostgresAgentAuditSink, PostgresAgentRepository, SyncPostgresAdapter,
};
use sdkwork_agents_contract::{agents_is_production_like_environment, agents_use_dev_inline_auth_resolver};

/// Build agents managed store HTTP state using postgres in production-like profiles and
/// in-memory fixtures only when dev inline auth is explicitly enabled.
///
/// **Security layers:**
/// 1. Web framework layer: `IamAuthorizationPolicy` (from `sdkwork-iam-web-adapter`)
///    performs HTTP route-level authorization based on IAM roles and organization scope.
/// 2. Application service layer: `IamGatedPolicyProvider` (from this crate) maps
///    agent business actions to IAM permission scopes (`ai.agents.read` /
///    `ai.agents.manage`) and checks the request subject's permission scope.
///
/// `AllowAllPolicyProvider` is only used for development scenarios where the
/// dev inline auth resolver is explicitly enabled.
pub fn build_agent_http_state() -> Result<AgentHttpState> {
    if agents_use_dev_inline_auth_resolver() {
        return Ok(dev_agent_http_state());
    }

    if agents_is_production_like_environment() {
        return production_postgres_agent_http_state();
    }

    production_postgres_agent_http_state().or_else(|error| {
        tracing::warn!(
            %error,
            "agents postgres managed store unavailable; falling back to in-memory dev fixtures for non-production profile"
        );
        Ok(dev_agent_http_state())
    })
}

fn dev_agent_http_state() -> AgentHttpState {
    AgentHttpState::new(
        InMemoryAgentRepository::new(),
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.agents.dev"),
    )
}

fn production_postgres_agent_http_state() -> Result<AgentHttpState> {
    let repository_adapter = SyncPostgresAdapter::connect_from_agents_managed_store_env()
        .context("connect agents managed store postgres adapter")?;
    repository_adapter
        .apply_managed_store_schema()
        .context("apply agents managed store postgres schema")?;

    // The audit sink shares the same physical postgres pool as the repository
    // but uses a dedicated snowflake node id so concurrent `next_id` calls
    // cannot collide. The audit sink extracts tenant/organization/agent
    // metadata from each event's structured context (populated by
    // `AgentsService::emit_audit_event`), so a single global sink serves
    // audit events for every agent in the process.
    let audit_adapter = {
        let audit_pool = repository_adapter.pool().clone();
        let audit_id_generator = AgentBusinessIdGenerator::with_node_id(AUDIT_SINK_NODE_ID)
            .context("build agents audit sink snowflake id generator")?;
        SyncPostgresAdapter::with_pool_and_id_generator(audit_pool, audit_id_generator)
    };

    let repository = PostgresAgentRepository::new(repository_adapter);
    let audit_sink = PostgresAgentAuditSink::new_global(audit_adapter);

    Ok(AgentHttpState::new(
        repository,
        audit_sink,
        IamGatedPolicyProvider::default(),
    ))
}
