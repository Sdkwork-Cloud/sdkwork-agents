use std::sync::atomic::{AtomicBool, Ordering};

use sdkwork_intelligence_agents_service::AgentTaskWorkerHandle;
use sdkwork_web_bootstrap::{ReadinessCheck, ReadinessFuture};

#[derive(Default)]
pub struct SchedulerWorkerControl {
    started: AtomicBool,
    draining: AtomicBool,
}

impl SchedulerWorkerControl {
    pub fn mark_started(&self) {
        self.started.store(true, Ordering::Release);
    }

    pub fn begin_draining(&self) {
        self.draining.store(true, Ordering::Release);
    }

    pub fn is_ready(&self) -> bool {
        self.started.load(Ordering::Acquire) && !self.draining.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
pub struct SchedulerWorkerReadiness {
    worker: AgentTaskWorkerHandle,
    control: std::sync::Arc<SchedulerWorkerControl>,
}

impl SchedulerWorkerReadiness {
    pub fn new(
        worker: AgentTaskWorkerHandle,
        control: std::sync::Arc<SchedulerWorkerControl>,
    ) -> Self {
        Self { worker, control }
    }
}

impl ReadinessCheck for SchedulerWorkerReadiness {
    fn check(&self) -> ReadinessFuture<'_> {
        let worker = self.worker.clone();
        let control = self.control.clone();
        Box::pin(async move {
            if !control.is_ready() {
                return Err("scheduler worker is not accepting new work".to_string());
            }
            tokio::task::spawn_blocking(move || worker.check_readiness())
                .await
                .map_err(|error| format!("scheduler readiness task failed: {error}"))?
                .map_err(|error| format!("scheduler repository unavailable: {error}"))
        })
    }
}
