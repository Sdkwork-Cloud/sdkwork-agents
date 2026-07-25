use sdkwork_intelligence_agents_service::AgentHttpState;
use sdkwork_web_bootstrap::{ReadinessCheck, ReadinessFuture};

#[derive(Clone)]
pub(crate) struct AgentHttpReadinessCheck {
    state: AgentHttpState,
}

impl AgentHttpReadinessCheck {
    pub(crate) fn new(state: AgentHttpState) -> Self {
        Self { state }
    }
}

impl ReadinessCheck for AgentHttpReadinessCheck {
    fn check(&self) -> ReadinessFuture<'_> {
        let state = self.state.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                state.check_readiness().map_err(|error| error.to_string())
            })
            .await
            .map_err(|error| format!("agents readiness worker failed: {error}"))?
        })
    }
}
