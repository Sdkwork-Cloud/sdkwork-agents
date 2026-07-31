use std::str::FromStr;

use chrono::{Duration, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use sdkwork_agent_kernel::{KernelError, KernelResult};
use sdkwork_utils_rust::{format_datetime, parse_datetime};

pub const MIN_TASK_TIMEOUT_SECONDS: u32 = 1;
pub const MAX_TASK_TIMEOUT_SECONDS: u32 = 86_400;
pub const MAX_TASK_CONCURRENT_RUNS: u16 = 32;
pub const MAX_TASK_CATCH_UP_RUNS: u16 = 100;
pub const MAX_TASK_RUN_ATTEMPTS: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskScheduleKind {
    OneTime,
    Cron,
}

impl AgentTaskScheduleKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneTime => "one_time",
            Self::Cron => "cron",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "one_time" => Some(Self::OneTime),
            "cron" => Some(Self::Cron),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::OneTime => 0,
            Self::Cron => 1,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::OneTime),
            1 => Some(Self::Cron),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskMisfirePolicy {
    Skip,
    FireOnce,
    CatchUp,
}

impl AgentTaskMisfirePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::FireOnce => "fire_once",
            Self::CatchUp => "catch_up",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "skip" => Some(Self::Skip),
            "fire_once" => Some(Self::FireOnce),
            "catch_up" => Some(Self::CatchUp),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Skip => 0,
            Self::FireOnce => 1,
            Self::CatchUp => 2,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Skip),
            1 => Some(Self::FireOnce),
            2 => Some(Self::CatchUp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskOverlapPolicy {
    Skip,
    Queue,
}

impl AgentTaskOverlapPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Queue => "queue",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "skip" => Some(Self::Skip),
            "queue" => Some(Self::Queue),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Skip => 0,
            Self::Queue => 1,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Skip),
            1 => Some(Self::Queue),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

impl AgentTaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "completed" => Some(Self::Completed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Active => 0,
            Self::Paused => 1,
            Self::Completed => 2,
            Self::Cancelled => 3,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Paused),
            2 => Some(Self::Completed),
            3 => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn is_cancellable(self) -> bool {
        matches!(self, Self::Active | Self::Paused)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSchedule {
    pub kind: AgentTaskScheduleKind,
    pub cron_expression: Option<String>,
    pub timezone: String,
    pub scheduled_at: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
}

impl TaskSchedule {
    pub fn validate(&self) -> KernelResult<()> {
        let timezone = Tz::from_str(self.timezone.trim())
            .map_err(|_| KernelError::validation("timezone must be a valid IANA timezone"))?;
        let starts_at = parse_optional_datetime(self.starts_at.as_deref(), "startsAt")?;
        let ends_at = parse_optional_datetime(self.ends_at.as_deref(), "endsAt")?;
        if matches!((starts_at, ends_at), (Some(start), Some(end)) if start >= end) {
            return Err(KernelError::validation("startsAt must be before endsAt"));
        }
        match self.kind {
            AgentTaskScheduleKind::OneTime => {
                if self.cron_expression.is_some() {
                    return Err(KernelError::validation(
                        "cronExpression is forbidden for a one-time task",
                    ));
                }
                parse_required_datetime(self.scheduled_at.as_deref(), "scheduledAt")?;
            }
            AgentTaskScheduleKind::Cron => {
                if self.scheduled_at.is_some() {
                    return Err(KernelError::validation(
                        "scheduledAt is forbidden for a cron task",
                    ));
                }
                let expression = self
                    .cron_expression
                    .as_deref()
                    .ok_or_else(|| KernelError::validation("cronExpression is required"))?;
                validate_cron(expression)?;
                let _ = timezone;
            }
        }
        Ok(())
    }

    pub fn next_after(&self, after: &str) -> KernelResult<Option<String>> {
        self.validate()?;
        let after = parse_required_datetime(Some(after), "after")?;
        let starts_at = parse_optional_datetime(self.starts_at.as_deref(), "startsAt")?;
        let ends_at = parse_optional_datetime(self.ends_at.as_deref(), "endsAt")?;
        let lower_bound = starts_at.map_or(after, |start| start.max(after));

        let candidate = match self.kind {
            AgentTaskScheduleKind::OneTime => {
                let scheduled_at =
                    parse_required_datetime(self.scheduled_at.as_deref(), "scheduledAt")?;
                (scheduled_at > after && scheduled_at >= lower_bound).then_some(scheduled_at)
            }
            AgentTaskScheduleKind::Cron => {
                let timezone = Tz::from_str(self.timezone.trim()).map_err(|_| {
                    KernelError::validation("timezone must be a valid IANA timezone")
                })?;
                let schedule = Schedule::from_str(
                    self.cron_expression
                        .as_deref()
                        .ok_or_else(|| KernelError::validation("cronExpression is required"))?,
                )
                .map_err(|_| KernelError::validation("cronExpression is invalid"))?;
                // `cron::Schedule::after` is exclusive. When startsAt raises the
                // lower bound, step back one nanosecond so the inclusive startsAt
                // contract can materialize an occurrence exactly on that boundary.
                let cron_after = if starts_at.is_some_and(|start| start > after) {
                    lower_bound - Duration::nanoseconds(1)
                } else {
                    lower_bound
                };
                schedule
                    .after(&cron_after.with_timezone(&timezone))
                    .next()
                    .map(|value| value.with_timezone(&Utc))
            }
        };

        if matches!((candidate, ends_at), (Some(next), Some(end)) if next >= end) {
            return Ok(None);
        }
        Ok(candidate.map(|value| format_datetime(value, None)))
    }
}

fn validate_cron(expression: &str) -> KernelResult<()> {
    let expression = expression.trim();
    if expression.split_whitespace().count() != 6 {
        return Err(KernelError::validation(
            "cronExpression must contain exactly six fields including seconds",
        ));
    }
    Schedule::from_str(expression)
        .map(|_| ())
        .map_err(|_| KernelError::validation("cronExpression is invalid"))
}

fn parse_required_datetime(
    value: Option<&str>,
    field: &'static str,
) -> KernelResult<chrono::DateTime<Utc>> {
    let value = value.ok_or_else(|| KernelError::validation(format!("{field} is required")))?;
    parse_datetime(value.trim(), None)
        .ok_or_else(|| KernelError::validation(format!("{field} must be an RFC 3339 instant")))
}

fn parse_optional_datetime(
    value: Option<&str>,
    field: &'static str,
) -> KernelResult<Option<chrono::DateTime<Utc>>> {
    value
        .map(|value| parse_required_datetime(Some(value), field))
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskRecord {
    pub id: u64,
    pub task_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub session_id: String,
    pub title: Option<String>,
    pub prompt: String,
    pub schedule_kind: AgentTaskScheduleKind,
    pub cron_expression: Option<String>,
    pub timezone: String,
    pub scheduled_at: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub next_fire_at: Option<String>,
    pub misfire_policy: AgentTaskMisfirePolicy,
    pub overlap_policy: AgentTaskOverlapPolicy,
    pub max_concurrent_runs: u16,
    pub max_catch_up_runs: u16,
    pub max_attempts: u16,
    pub retry_initial_delay_seconds: u32,
    pub retry_max_delay_seconds: u32,
    pub timeout_seconds: u32,
    pub priority: i16,
    pub status: AgentTaskStatus,
    pub generation: u64,
    pub external_ref: Option<String>,
    pub metadata_json: String,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub paused_at: Option<String>,
    pub cancelled_at: Option<String>,
}

impl AgentTaskRecord {
    pub fn schedule(&self) -> TaskSchedule {
        TaskSchedule {
            kind: self.schedule_kind,
            cron_expression: self.cron_expression.clone(),
            timezone: self.timezone.clone(),
            scheduled_at: self.scheduled_at.clone(),
            starts_at: self.starts_at.clone(),
            ends_at: self.ends_at.clone(),
        }
    }

    pub fn mark_updated(&mut self, updated_at: impl Into<String>) {
        self.updated_at = updated_at.into();
        self.version = self.version.saturating_add(1);
    }

    pub fn cancel(&mut self, cancelled_at: impl Into<String>) {
        let at = cancelled_at.into();
        self.status = AgentTaskStatus::Cancelled;
        self.next_fire_at = None;
        self.cancelled_at = Some(at.clone());
        self.paused_at = None;
        self.updated_at = at;
        self.version = self.version.saturating_add(1);
        self.generation = self.generation.saturating_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskTriggerKind {
    Scheduled,
    Manual,
    BusinessRetry,
}

impl AgentTaskTriggerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
            Self::BusinessRetry => "business_retry",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "scheduled" => Some(Self::Scheduled),
            "manual" => Some(Self::Manual),
            "business_retry" => Some(Self::BusinessRetry),
            _ => None,
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Scheduled => 0,
            Self::Manual => 1,
            Self::BusinessRetry => 2,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Scheduled),
            1 => Some(Self::Manual),
            2 => Some(Self::BusinessRetry),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskRunStatus {
    Pending,
    Claimed,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Reconciling,
    DeadLetter,
}

impl AgentTaskRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Claimed => "claimed",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Reconciling => "reconciling",
            Self::DeadLetter => "dead_letter",
        }
    }

    pub fn from_code(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "claimed" => Some(Self::Claimed),
            "running" => Some(Self::Running),
            "succeeded" => Some(Self::Succeeded),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            "reconciling" => Some(Self::Reconciling),
            "dead_letter" => Some(Self::DeadLetter),
            _ => None,
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::DeadLetter
        )
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Pending => 0,
            Self::Claimed => 1,
            Self::Running => 2,
            Self::Succeeded => 3,
            Self::Failed => 4,
            Self::Cancelled => 5,
            Self::Reconciling => 6,
            Self::DeadLetter => 7,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Pending),
            1 => Some(Self::Claimed),
            2 => Some(Self::Running),
            3 => Some(Self::Succeeded),
            4 => Some(Self::Failed),
            5 => Some(Self::Cancelled),
            6 => Some(Self::Reconciling),
            7 => Some(Self::DeadLetter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskRunRecord {
    pub id: u64,
    pub run_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub task_id: String,
    pub session_id: String,
    pub agent_id: String,
    pub owner_user_id: u64,
    pub trigger_kind: AgentTaskTriggerKind,
    pub schedule_generation: u64,
    pub scheduled_for: String,
    pub retry_of_run_id: Option<String>,
    pub priority: i16,
    pub status: AgentTaskRunStatus,
    pub idempotency_key: String,
    pub payload_hash: String,
    pub turn_id: Option<String>,
    pub attempt_count: u16,
    pub max_attempts: u16,
    pub available_at: String,
    pub lease_owner: Option<String>,
    pub lease_token_hash: Option<String>,
    pub lease_expires_at: Option<String>,
    pub fencing_token: u64,
    pub timeout_at: Option<String>,
    pub failure_class: Option<String>,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub version: u64,
    pub created_at: String,
    pub updated_at: String,
    pub claimed_at: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub cancelled_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentTaskRunAttemptStatus {
    Claimed,
    Running,
    Succeeded,
    Failed,
    LeaseExpired,
    Cancelled,
}

impl AgentTaskRunAttemptStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claimed => "claimed",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::LeaseExpired => "lease_expired",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn as_db_code(self) -> i16 {
        match self {
            Self::Claimed => 0,
            Self::Running => 1,
            Self::Succeeded => 2,
            Self::Failed => 3,
            Self::LeaseExpired => 4,
            Self::Cancelled => 5,
        }
    }

    pub fn from_db_code(value: i16) -> Option<Self> {
        match value {
            0 => Some(Self::Claimed),
            1 => Some(Self::Running),
            2 => Some(Self::Succeeded),
            3 => Some(Self::Failed),
            4 => Some(Self::LeaseExpired),
            5 => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTaskRunAttemptRecord {
    pub id: u64,
    pub attempt_id: String,
    pub tenant_id: u64,
    pub organization_id: u64,
    pub run_id: String,
    pub attempt_no: u16,
    pub worker_id: String,
    pub status: AgentTaskRunAttemptStatus,
    pub lease_token_hash: String,
    pub fencing_token: u64,
    pub failure_class: Option<String>,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub heartbeat_at: Option<String>,
    pub finished_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_time_schedule_returns_only_future_occurrence() {
        let schedule = TaskSchedule {
            kind: AgentTaskScheduleKind::OneTime,
            cron_expression: None,
            timezone: "UTC".to_string(),
            scheduled_at: Some("2026-08-01T00:00:00.000Z".to_string()),
            starts_at: None,
            ends_at: None,
        };
        assert_eq!(
            schedule
                .next_after("2026-07-31T00:00:00.000Z")
                .expect("schedule"),
            Some("2026-08-01T00:00:00.000Z".to_string())
        );
        assert_eq!(
            schedule
                .next_after("2026-08-01T00:00:00.000Z")
                .expect("schedule"),
            None
        );
    }

    #[test]
    fn cron_schedule_uses_iana_timezone() {
        let schedule = TaskSchedule {
            kind: AgentTaskScheduleKind::Cron,
            cron_expression: Some("0 0 9 * * *".to_string()),
            timezone: "Asia/Shanghai".to_string(),
            scheduled_at: None,
            starts_at: None,
            ends_at: None,
        };
        assert_eq!(
            schedule
                .next_after("2026-07-31T00:00:00.000Z")
                .expect("schedule"),
            Some("2026-07-31T01:00:00.000Z".to_string())
        );
    }

    #[test]
    fn cron_requires_exactly_six_fields() {
        let schedule = TaskSchedule {
            kind: AgentTaskScheduleKind::Cron,
            cron_expression: Some("0 9 * * *".to_string()),
            timezone: "UTC".to_string(),
            scheduled_at: None,
            starts_at: None,
            ends_at: None,
        };
        assert!(schedule.validate().is_err());
    }

    #[test]
    fn cron_starts_at_is_inclusive() {
        let schedule = TaskSchedule {
            kind: AgentTaskScheduleKind::Cron,
            cron_expression: Some("0 0 9 * * *".to_string()),
            timezone: "Asia/Shanghai".to_string(),
            scheduled_at: None,
            starts_at: Some("2026-08-01T01:00:00.000Z".to_string()),
            ends_at: None,
        };
        assert_eq!(
            schedule
                .next_after("2026-07-31T23:00:00.000Z")
                .expect("schedule"),
            Some("2026-08-01T01:00:00.000Z".to_string())
        );
    }

    #[test]
    fn cron_skips_nonexistent_dst_local_time_without_panicking() {
        let schedule = TaskSchedule {
            kind: AgentTaskScheduleKind::Cron,
            cron_expression: Some("0 30 2 * * *".to_string()),
            timezone: "America/New_York".to_string(),
            scheduled_at: None,
            starts_at: None,
            ends_at: None,
        };
        let next = schedule
            .next_after("2026-03-08T06:59:59.000Z")
            .expect("schedule")
            .expect("next occurrence");
        assert_eq!(next, "2026-03-09T06:30:00.000Z");
    }
}
