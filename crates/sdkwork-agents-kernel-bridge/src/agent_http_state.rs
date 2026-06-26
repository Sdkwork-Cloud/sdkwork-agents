//! Production and development `AgentHttpState` bootstrap for SDKWork Agents.

use anyhow::{Context, Result};
use sdkwork_intelligence_agents_service::{
    AgentHttpState, AllowAllPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    PostgresAgentRepository, SyncPostgresAdapter,
};
use sdkwork_agents_contract::{agents_is_production_like_environment, agents_use_dev_inline_auth_resolver};

/// Build agents managed store HTTP state using postgres in production-like profiles and
/// in-memory fixtures only when dev inline auth is explicitly enabled.
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
    let repository = PostgresAgentRepository::new(repository_adapter);

    Ok(AgentHttpState::new(
        repository,
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::allow("policy.agents.production.iam-gated"),
    ))
}
