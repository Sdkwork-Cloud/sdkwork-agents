use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct SchedulerWorkerMetrics {
    materialized_total: AtomicU64,
    claimed_total: AtomicU64,
    recovered_leases_total: AtomicU64,
    reconciliation_examined_total: AtomicU64,
    reconciliation_reconciled_total: AtomicU64,
    reconciliation_pending_total: AtomicU64,
    reconciliation_conflicts_total: AtomicU64,
    heartbeats_total: AtomicU64,
    heartbeat_failures_total: AtomicU64,
    executions_succeeded_total: AtomicU64,
    executions_failed_total: AtomicU64,
    executions_reconciling_total: AtomicU64,
    executions_dead_letter_total: AtomicU64,
    executions_cancelled_total: AtomicU64,
    operation_errors_total: AtomicU64,
    forced_drain_total: AtomicU64,
    inflight: AtomicU64,
}

impl SchedulerWorkerMetrics {
    pub(crate) fn add_materialized(&self, count: usize) {
        add(&self.materialized_total, count as u64);
    }

    pub(crate) fn add_claimed(&self, count: usize) {
        add(&self.claimed_total, count as u64);
    }

    pub(crate) fn add_recovered(&self, count: u64) {
        add(&self.recovered_leases_total, count);
    }

    pub(crate) fn record_reconciliation(
        &self,
        result: &sdkwork_intelligence_agents_service::TaskRunReconciliationResult,
    ) {
        add(&self.reconciliation_examined_total, result.examined as u64);
        add(
            &self.reconciliation_reconciled_total,
            result.reconciled.len() as u64,
        );
        add(&self.reconciliation_pending_total, result.pending as u64);
        add(
            &self.reconciliation_conflicts_total,
            result.skipped_conflicts as u64,
        );
    }

    pub(crate) fn record_heartbeat(&self, succeeded: bool) {
        add(&self.heartbeats_total, 1);
        if !succeeded {
            add(&self.heartbeat_failures_total, 1);
        }
    }

    pub(crate) fn execution_started(&self) {
        add(&self.inflight, 1);
    }

    pub(crate) fn execution_finished(
        &self,
        result: &sdkwork_agent_kernel::KernelResult<
            sdkwork_intelligence_agents_service::AgentTaskRunRecord,
        >,
    ) {
        subtract(&self.inflight, 1);
        use sdkwork_intelligence_agents_service::AgentTaskRunStatus;
        match result.as_ref().map(|run| run.status) {
            Ok(AgentTaskRunStatus::Succeeded) => add(&self.executions_succeeded_total, 1),
            Ok(AgentTaskRunStatus::Reconciling) => add(&self.executions_reconciling_total, 1),
            Ok(AgentTaskRunStatus::DeadLetter) => add(&self.executions_dead_letter_total, 1),
            Ok(AgentTaskRunStatus::Cancelled) => add(&self.executions_cancelled_total, 1),
            Ok(_) | Err(_) => add(&self.executions_failed_total, 1),
        }
    }

    pub(crate) fn record_operation_error(&self) {
        add(&self.operation_errors_total, 1);
    }

    pub(crate) fn record_forced_drain(&self) {
        add(&self.forced_drain_total, 1);
    }

    pub fn inflight(&self) -> u64 {
        self.inflight.load(Ordering::Relaxed)
    }

    pub fn render_prometheus(&self) -> String {
        format!(
            concat!(
                "# HELP sdkwork_agents_task_worker_materialized_total Logical task runs materialized.\n",
                "# TYPE sdkwork_agents_task_worker_materialized_total counter\n",
                "sdkwork_agents_task_worker_materialized_total {}\n",
                "# HELP sdkwork_agents_task_worker_claimed_total Task run delivery attempts claimed.\n",
                "# TYPE sdkwork_agents_task_worker_claimed_total counter\n",
                "sdkwork_agents_task_worker_claimed_total {}\n",
                "# HELP sdkwork_agents_task_worker_recovered_leases_total Expired task run leases recovered.\n",
                "# TYPE sdkwork_agents_task_worker_recovered_leases_total counter\n",
                "sdkwork_agents_task_worker_recovered_leases_total {}\n",
                "# HELP sdkwork_agents_task_worker_reconciliation_examined_total Reconciling task runs examined.\n",
                "# TYPE sdkwork_agents_task_worker_reconciliation_examined_total counter\n",
                "sdkwork_agents_task_worker_reconciliation_examined_total {}\n",
                "# HELP sdkwork_agents_task_worker_reconciliation_reconciled_total Task runs moved from reconciling to a canonical terminal outcome.\n",
                "# TYPE sdkwork_agents_task_worker_reconciliation_reconciled_total counter\n",
                "sdkwork_agents_task_worker_reconciliation_reconciled_total {}\n",
                "# HELP sdkwork_agents_task_worker_reconciliation_pending_total Task runs whose canonical turn outcome is not terminal yet.\n",
                "# TYPE sdkwork_agents_task_worker_reconciliation_pending_total counter\n",
                "sdkwork_agents_task_worker_reconciliation_pending_total {}\n",
                "# HELP sdkwork_agents_task_worker_reconciliation_conflicts_total Task run reconciliation writes skipped after a concurrent version change.\n",
                "# TYPE sdkwork_agents_task_worker_reconciliation_conflicts_total counter\n",
                "sdkwork_agents_task_worker_reconciliation_conflicts_total {}\n",
                "# HELP sdkwork_agents_task_worker_heartbeats_total Task run lease heartbeat operations.\n",
                "# TYPE sdkwork_agents_task_worker_heartbeats_total counter\n",
                "sdkwork_agents_task_worker_heartbeats_total {}\n",
                "# HELP sdkwork_agents_task_worker_heartbeat_failures_total Failed task run lease heartbeats.\n",
                "# TYPE sdkwork_agents_task_worker_heartbeat_failures_total counter\n",
                "sdkwork_agents_task_worker_heartbeat_failures_total {}\n",
                "# HELP sdkwork_agents_task_worker_executions_total Task run executions by bounded outcome.\n",
                "# TYPE sdkwork_agents_task_worker_executions_total counter\n",
                "sdkwork_agents_task_worker_executions_total{{outcome=\"succeeded\"}} {}\n",
                "sdkwork_agents_task_worker_executions_total{{outcome=\"failed\"}} {}\n",
                "sdkwork_agents_task_worker_executions_total{{outcome=\"reconciling\"}} {}\n",
                "sdkwork_agents_task_worker_executions_total{{outcome=\"dead_letter\"}} {}\n",
                "sdkwork_agents_task_worker_executions_total{{outcome=\"cancelled\"}} {}\n",
                "# HELP sdkwork_agents_task_worker_operation_errors_total Scheduler repository operation errors.\n",
                "# TYPE sdkwork_agents_task_worker_operation_errors_total counter\n",
                "sdkwork_agents_task_worker_operation_errors_total {}\n",
                "# HELP sdkwork_agents_task_worker_forced_drain_total Shutdowns that exceeded the configured drain timeout.\n",
                "# TYPE sdkwork_agents_task_worker_forced_drain_total counter\n",
                "sdkwork_agents_task_worker_forced_drain_total {}\n",
                "# HELP sdkwork_agents_task_worker_inflight Current task run executions.\n",
                "# TYPE sdkwork_agents_task_worker_inflight gauge\n",
                "sdkwork_agents_task_worker_inflight {}\n"
            ),
            self.materialized_total.load(Ordering::Relaxed),
            self.claimed_total.load(Ordering::Relaxed),
            self.recovered_leases_total.load(Ordering::Relaxed),
            self.reconciliation_examined_total.load(Ordering::Relaxed),
            self.reconciliation_reconciled_total.load(Ordering::Relaxed),
            self.reconciliation_pending_total.load(Ordering::Relaxed),
            self.reconciliation_conflicts_total.load(Ordering::Relaxed),
            self.heartbeats_total.load(Ordering::Relaxed),
            self.heartbeat_failures_total.load(Ordering::Relaxed),
            self.executions_succeeded_total.load(Ordering::Relaxed),
            self.executions_failed_total.load(Ordering::Relaxed),
            self.executions_reconciling_total.load(Ordering::Relaxed),
            self.executions_dead_letter_total.load(Ordering::Relaxed),
            self.executions_cancelled_total.load(Ordering::Relaxed),
            self.operation_errors_total.load(Ordering::Relaxed),
            self.forced_drain_total.load(Ordering::Relaxed),
            self.inflight.load(Ordering::Relaxed),
        )
    }
}

fn add(value: &AtomicU64, amount: u64) {
    let _ = value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_add(amount))
    });
}

fn subtract(value: &AtomicU64, amount: u64) {
    let _ = value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        Some(current.saturating_sub(amount))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_use_only_bounded_outcome_labels() {
        let rendered = SchedulerWorkerMetrics::default().render_prometheus();
        assert!(rendered.contains("outcome=\"succeeded\""));
        assert!(!rendered.contains("tenant"));
        assert!(!rendered.contains("run_id"));
        assert!(!rendered.contains("worker_id"));
    }
}
