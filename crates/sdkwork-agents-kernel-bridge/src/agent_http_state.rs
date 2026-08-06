//! Production and development `AgentHttpState` bootstrap for SDKWork Agents.

use anyhow::{Context, Result};
use sdkwork_agent_kernel::{
    AgentConfigurationStore, InMemoryAgentConfigurationStore, InMemorySecretProvider,
};
use sdkwork_agents_contract::{
    agents_use_dev_inline_auth_resolver, ensure_dev_auth_bypass_allowed,
};
use sdkwork_intelligence_agents_service::{
    AgentHttpState, AllowAllPolicyProvider, CloudRouterFirstTurnExecutor, IamGatedPolicyProvider,
    InMemoryAgentAuditSink, InMemoryAgentRepository, RuntimeFacadeTurnExecutor, SqlAgentAuditSink,
    PostgresAgentConfigurationStore, SqlAgentRepository, SyncPostgresAdapter,
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
    Ok(AgentHttpState::with_turn_executor(
        InMemoryAgentRepository::try_new().context("build agents dev in-memory repository")?,
        InMemoryAgentAuditSink::default(),
        AllowAllPolicyProvider::try_allow("policy.agents.dev")
            .map_err(anyhow::Error::msg)
            .context("build agents dev-only policy provider")?,
        Arc::new(CloudRouterFirstTurnExecutor::new(RuntimeFacadeTurnExecutor)),
    ))
}

fn production_postgres_agent_http_state() -> Result<AgentHttpState> {
    let repository_adapter = SyncPostgresAdapter::connect_from_agents_database_env()
        .context("connect canonical Agents PostgreSQL database")?;

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
        pool.block_on(sdkwork_agents_database_host::bootstrap_agents_database(
            database_pool,
        ))
        .map_err(anyhow::Error::msg)
        .context("apply canonical Agents schema via lifecycle orchestrator")?;
    }

    // Repository and audit writes share one process-level leased Snowflake
    // generator. Its cloned sequence state is the collision boundary for all
    // Agents modules in this process; other pods receive distinct node leases.
    let audit_adapter = repository_adapter.clone();
    let configuration_pool = repository_adapter.pool().clone();

    let repository = SqlAgentRepository::new(repository_adapter);
    let audit_sink = SqlAgentAuditSink::new_global(audit_adapter);

    let state = AgentHttpState::with_turn_executor(
        repository,
        audit_sink,
        IamGatedPolicyProvider::default(),
        // Chat turns carrying a user auth token route through the cloudrouter
        // account-pool gateway; turns without one (worker/backend flows) keep
        // the local agent-engine facade execution.
        Arc::new(CloudRouterFirstTurnExecutor::new(RuntimeFacadeTurnExecutor)),
    );

    // Persist applied model configuration profiles in the canonical Agents
    // PostgreSQL database (server-authoritative persistence; SQLite is
    // client-local only per DATABASE_SPEC).
    let configuration_store =
        PostgresAgentConfigurationStore::from_pool(configuration_pool);
    Ok(state.with_model_configuration_providers(
        Box::new(InMemorySecretProvider::new()),
        Box::new(configuration_store),
    ))
}
