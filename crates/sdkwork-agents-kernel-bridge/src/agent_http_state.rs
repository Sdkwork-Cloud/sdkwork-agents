//! Production and development `AgentHttpState` bootstrap for SDKWork Agents.

use anyhow::{Context, Result};
use sdkwork_agents_contract::{
    agents_use_dev_inline_auth_resolver, ensure_dev_auth_bypass_allowed,
};
use sdkwork_intelligence_agents_service::{
    AgentBusinessIdGenerator, AgentHttpState, AllowAllPolicyProvider, IamGatedPolicyProvider,
    InMemoryAgentAuditSink, InMemoryAgentRepository, PostgresAgentAuditSink,
    PostgresAgentRepository, RuntimeFacadeChatCompleter, SyncPostgresAdapter, AUDIT_SINK_NODE_ID,
};
use std::sync::Arc;

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
    ensure_dev_auth_bypass_allowed()
        .map_err(|message| anyhow::anyhow!("agents security bootstrap: {message}"))?;

    if agents_use_dev_inline_auth_resolver() {
        tracing::warn!(
            env = %sdkwork_agents_contract::agents_deployment_environment_name(),
            "agents dev inline auth bypass is active; using in-memory repository and AllowAllPolicyProvider"
        );
        return dev_agent_http_state();
    }

    production_postgres_agent_http_state()
}

fn dev_agent_http_state() -> Result<AgentHttpState> {
    Ok(AgentHttpState::new(
        InMemoryAgentRepository::try_new().context("build agents dev in-memory repository")?,
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::try_allow("policy.agents.dev")
            .map_err(anyhow::Error::msg)
            .context("build agents dev-only policy provider")?,
    ))
}

fn production_postgres_agent_http_state() -> Result<AgentHttpState> {
    let repository_adapter = SyncPostgresAdapter::connect_from_agents_managed_store_env()
        .context("connect agents managed store postgres adapter")?;

    // Apply schema via the sdkwork-database lifecycle orchestrator instead of
    // directly executing baseline SQL. This ensures:
    // 1. Baseline is applied once and tracked in `ops_schema_migration_history`.
    // 2. Incremental migrations in `database/migrations/postgres/` are applied.
    // 3. Checksums are recorded for drift detection.
    // The `database/` directory is shipped in the production image (see
    // `deployments/docker/Dockerfile`) and `SDKWORK_AGENTS_APP_ROOT` is set.
    {
        let pool = repository_adapter.pool().clone();
        let database_pool = pool.database_pool().clone();
        pool.block_on(sdkwork_agents_database_host::bootstrap_agents_database(database_pool))
            .map_err(anyhow::Error::msg)
            .context("apply agents managed store schema via lifecycle orchestrator")?;
    }

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

    Ok(AgentHttpState::with_chat_completer(
        repository,
        audit_sink,
        IamGatedPolicyProvider::default(),
        Arc::new(RuntimeFacadeChatCompleter),
    ))
}
