use serde::{Deserialize, Serialize};

use crate::agent_turn::{AgentTurnRecord, AgentTurnStatus};
use crate::domain::{
    AgentInteractionRecord, AgentResourceUserStateRecord, AgentSessionRecord,
    AgentSessionRuntimeBindingRecord, AgentSessionRuntimeBindingStatus, AgentSessionStatus,
};
use crate::validation::parse_rfc3339_datetime;
use sdkwork_agent_kernel::{
    KernelError, KernelResult, SessionActivityEvidenceKind as KernelActivityEvidenceKind,
    SessionActivityFreshness as KernelActivityFreshness,
    SessionActivityInteractionHint as KernelActivityInteractionHint, SessionActivitySnapshot,
    SessionActivityState as KernelActivityState,
};
use sdkwork_utils_rust::{
    encoding::{base64url_decode, base64url_encode},
    sha256_hash,
};
use time::OffsetDateTime;

const SESSION_ACTIVITY_CURSOR_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionActivitySource {
    Session,
    Turn,
    Interaction,
    RuntimeBinding,
    UserState,
}

impl SessionActivitySource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Turn => "turn",
            Self::Interaction => "interaction",
            Self::RuntimeBinding => "runtime_binding",
            Self::UserState => "user_state",
        }
    }

    pub(crate) fn from_code(value: &str) -> KernelResult<Self> {
        match value {
            "session" => Ok(Self::Session),
            "turn" => Ok(Self::Turn),
            "interaction" => Ok(Self::Interaction),
            "runtime_binding" => Ok(Self::RuntimeBinding),
            "user_state" => Ok(Self::UserState),
            _ => Err(KernelError::Internal {
                message: format!("stored session activity source is invalid: {value}"),
            }),
        }
    }

    pub(crate) fn precedence(self) -> u8 {
        match self {
            Self::Session => 0,
            Self::RuntimeBinding => 1,
            Self::Turn => 2,
            Self::Interaction => 3,
            Self::UserState => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPresentationPhase {
    Ready,
    Queued,
    Running,
    Waiting,
    AwaitingInput,
    Completed,
    Failed,
    Cancelled,
    Idle,
    Closed,
    Archived,
    Deleted,
    Unknown,
}

impl SessionPresentationPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::AwaitingInput => "awaiting_input",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Idle => "idle",
            Self::Closed => "closed",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProviderActivityState {
    Idle,
    Working,
    Waiting,
    Failed,
}

impl SessionProviderActivityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
        }
    }
}

impl From<KernelActivityState> for SessionProviderActivityState {
    fn from(value: KernelActivityState) -> Self {
        match value {
            KernelActivityState::Idle => Self::Idle,
            KernelActivityState::Working => Self::Working,
            KernelActivityState::Waiting => Self::Waiting,
            KernelActivityState::Failed => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProviderActivityFreshness {
    Fresh,
    Stale,
    Unsupported,
    Unavailable,
}

impl SessionProviderActivityFreshness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::Unsupported => "unsupported",
            Self::Unavailable => "unavailable",
        }
    }
}

impl From<KernelActivityFreshness> for SessionProviderActivityFreshness {
    fn from(value: KernelActivityFreshness) -> Self {
        match value {
            KernelActivityFreshness::Fresh => Self::Fresh,
            KernelActivityFreshness::Stale => Self::Stale,
            KernelActivityFreshness::Unsupported => Self::Unsupported,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProviderActivityEvidenceKind {
    ProviderStatus,
    ProviderEvent,
    ProviderLock,
    ProviderProcess,
}

impl SessionProviderActivityEvidenceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProviderStatus => "provider_status",
            Self::ProviderEvent => "provider_event",
            Self::ProviderLock => "provider_lock",
            Self::ProviderProcess => "provider_process",
        }
    }
}

impl From<KernelActivityEvidenceKind> for SessionProviderActivityEvidenceKind {
    fn from(value: KernelActivityEvidenceKind) -> Self {
        match value {
            KernelActivityEvidenceKind::ProviderStatus => Self::ProviderStatus,
            KernelActivityEvidenceKind::ProviderEvent => Self::ProviderEvent,
            KernelActivityEvidenceKind::ProviderLock => Self::ProviderLock,
            KernelActivityEvidenceKind::ProviderProcess => Self::ProviderProcess,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionProviderActivityInteractionHint {
    ApprovalRequired,
    UserInputRequired,
}

impl SessionProviderActivityInteractionHint {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApprovalRequired => "approval_required",
            Self::UserInputRequired => "user_input_required",
        }
    }
}

impl From<KernelActivityInteractionHint> for SessionProviderActivityInteractionHint {
    fn from(value: KernelActivityInteractionHint) -> Self {
        match value {
            KernelActivityInteractionHint::ApprovalRequired => Self::ApprovalRequired,
            KernelActivityInteractionHint::UserInputRequired => Self::UserInputRequired,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionProviderActivityObservation {
    pub provider_session_id: String,
    pub state: Option<SessionProviderActivityState>,
    pub freshness: SessionProviderActivityFreshness,
    pub evidence_kind: Option<SessionProviderActivityEvidenceKind>,
    pub interaction_hint: Option<SessionProviderActivityInteractionHint>,
    pub observed_at: Option<String>,
    pub fresh_until: Option<String>,
}

impl SessionProviderActivityObservation {
    pub fn unavailable(provider_session_id: impl Into<String>) -> Self {
        Self {
            provider_session_id: provider_session_id.into(),
            state: None,
            freshness: SessionProviderActivityFreshness::Unavailable,
            evidence_kind: None,
            interaction_hint: None,
            observed_at: None,
            fresh_until: None,
        }
    }

    pub(crate) fn from_provider_snapshot(
        expected_provider_session_id: &str,
        snapshot: SessionActivitySnapshot,
    ) -> Self {
        if snapshot.provider_session_id != expected_provider_session_id {
            return Self::unavailable(expected_provider_session_id);
        }
        Self::from(snapshot)
    }

    fn is_authoritative(&self) -> bool {
        if self.freshness != SessionProviderActivityFreshness::Fresh
            || self.state.is_none()
            || self.evidence_kind.is_none()
        {
            return false;
        }
        let (Some(observed_at), Some(fresh_until)) =
            (self.observed_at.as_deref(), self.fresh_until.as_deref())
        else {
            return false;
        };
        let Ok(observed_at) = parse_rfc3339_datetime(observed_at, "provider activity observedAt")
        else {
            return false;
        };
        let Ok(fresh_until) = parse_rfc3339_datetime(fresh_until, "provider activity freshUntil")
        else {
            return false;
        };
        fresh_until > observed_at && fresh_until > OffsetDateTime::now_utc()
    }
}

impl From<SessionActivitySnapshot> for SessionProviderActivityObservation {
    fn from(value: SessionActivitySnapshot) -> Self {
        let mut observation = Self {
            provider_session_id: value.provider_session_id,
            state: value.state.map(Into::into),
            freshness: value.freshness.into(),
            evidence_kind: value.evidence_kind.map(Into::into),
            interaction_hint: value.interaction_hint.map(Into::into),
            observed_at: value.observed_at,
            fresh_until: value.fresh_until,
        };
        if observation.freshness == SessionProviderActivityFreshness::Fresh
            && !observation.is_authoritative()
        {
            observation.freshness = SessionProviderActivityFreshness::Stale;
        }
        observation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionProviderIdentity {
    pub runtime_binding_id: Option<String>,
    pub provider_binding_id: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub provider_session_id: Option<String>,
    pub provider_session_tree_id: Option<String>,
    pub provider_parent_session_id: Option<String>,
    pub provider_forked_from_session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionActivityFreshness {
    pub activity_at: String,
    pub source: SessionActivitySource,
    pub observed_at: Option<String>,
    pub fresh_until: Option<String>,
    pub session_version: u64,
    pub latest_turn_version: Option<u64>,
    pub latest_interaction_id: Option<String>,
    pub latest_interaction_version: Option<u64>,
    pub latest_runtime_binding_id: Option<String>,
    pub latest_runtime_binding_version: Option<u64>,
    pub pending_interaction_version: Option<u64>,
    pub current_runtime_binding_version: Option<u64>,
    pub user_state_version: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionActivitySummaryRecord {
    pub session: AgentSessionRecord,
    pub latest_turn: Option<AgentTurnRecord>,
    pub pending_interaction: Option<AgentInteractionRecord>,
    pub current_runtime_binding: Option<AgentSessionRuntimeBindingRecord>,
    pub latest_runtime_binding: Option<AgentSessionRuntimeBindingRecord>,
    pub user_state: Option<AgentResourceUserStateRecord>,
    pub provider_identity: SessionProviderIdentity,
    pub freshness: SessionActivityFreshness,
    pub provider_activity: Option<SessionProviderActivityObservation>,
    pub presentation_phase: SessionPresentationPhase,
}

impl SessionActivitySummaryRecord {
    pub(crate) fn from_parts(
        session: AgentSessionRecord,
        latest_turn: Option<AgentTurnRecord>,
        pending_interaction: Option<AgentInteractionRecord>,
        current_runtime_binding: Option<AgentSessionRuntimeBindingRecord>,
        latest_runtime_binding: Option<AgentSessionRuntimeBindingRecord>,
        user_state: Option<AgentResourceUserStateRecord>,
        latest_interaction_component: Option<(String, u64)>,
        activity_at: String,
        activity_source: SessionActivitySource,
    ) -> Self {
        let provider_identity =
            provider_identity(current_runtime_binding.as_ref(), latest_turn.as_ref());
        let (observed_at, fresh_until) = effective_activity_timing(
            latest_turn.as_ref(),
            pending_interaction.as_ref(),
            &activity_at,
        );
        let presentation_phase = presentation_phase(
            &session,
            latest_turn.as_ref(),
            pending_interaction.as_ref(),
            current_runtime_binding.as_ref(),
            latest_runtime_binding.as_ref(),
            fresh_until.as_deref(),
        );
        let freshness = SessionActivityFreshness {
            activity_at,
            source: activity_source,
            observed_at,
            fresh_until,
            session_version: session.version,
            latest_turn_version: latest_turn.as_ref().map(|turn| turn.version),
            latest_interaction_id: latest_interaction_component
                .as_ref()
                .map(|(identity, _)| identity.clone()),
            latest_interaction_version: latest_interaction_component
                .as_ref()
                .map(|(_, version)| *version),
            latest_runtime_binding_id: latest_runtime_binding
                .as_ref()
                .map(|binding| binding.runtime_binding_id.clone()),
            latest_runtime_binding_version: latest_runtime_binding
                .as_ref()
                .map(|binding| binding.version),
            pending_interaction_version: pending_interaction
                .as_ref()
                .map(|interaction| interaction.version),
            current_runtime_binding_version: current_runtime_binding
                .as_ref()
                .map(|binding| binding.version),
            user_state_version: user_state.as_ref().map(|state| state.version),
        };
        Self {
            session,
            latest_turn,
            pending_interaction,
            current_runtime_binding,
            latest_runtime_binding,
            user_state,
            provider_identity,
            freshness,
            provider_activity: None,
            presentation_phase,
        }
    }

    pub(crate) fn with_provider_activity(
        mut self,
        observation: SessionProviderActivityObservation,
    ) -> Self {
        if let Some(provider_phase) =
            authoritative_provider_phase(self.presentation_phase, &observation)
        {
            self.freshness.observed_at = observation.observed_at.clone();
            self.freshness.fresh_until = observation.fresh_until.clone();
            self.presentation_phase = provider_phase;
        }
        self.provider_activity = Some(observation);
        self
    }
}

fn authoritative_provider_phase(
    persisted_phase: SessionPresentationPhase,
    observation: &SessionProviderActivityObservation,
) -> Option<SessionPresentationPhase> {
    if !observation.is_authoritative() {
        return None;
    }
    let provider_phase = match observation.state {
        Some(SessionProviderActivityState::Idle) => SessionPresentationPhase::Idle,
        Some(SessionProviderActivityState::Working) => SessionPresentationPhase::Running,
        Some(SessionProviderActivityState::Waiting) if observation.interaction_hint.is_some() => {
            SessionPresentationPhase::AwaitingInput
        }
        Some(SessionProviderActivityState::Waiting) => SessionPresentationPhase::Waiting,
        Some(SessionProviderActivityState::Failed) => SessionPresentationPhase::Failed,
        None => return None,
    };
    match persisted_phase {
        SessionPresentationPhase::Unknown => Some(provider_phase),
        SessionPresentationPhase::Ready
        | SessionPresentationPhase::Idle
        | SessionPresentationPhase::Completed
            if provider_phase != SessionPresentationPhase::Idle =>
        {
            Some(provider_phase)
        }
        _ => None,
    }
}

fn provider_identity(
    binding: Option<&AgentSessionRuntimeBindingRecord>,
    turn: Option<&AgentTurnRecord>,
) -> SessionProviderIdentity {
    SessionProviderIdentity {
        runtime_binding_id: binding
            .map(|value| value.runtime_binding_id.clone())
            .or_else(|| turn.and_then(|value| value.runtime_binding_id.clone())),
        provider_binding_id: binding
            .map(|value| value.provider_binding_id.clone())
            .or_else(|| turn.and_then(|value| value.provider_binding_id.clone())),
        provider_id: binding
            .map(|value| value.provider_id.clone())
            .or_else(|| turn.and_then(|value| value.provider_id.clone())),
        model_id: binding
            .map(|value| value.model_id.clone())
            .or_else(|| turn.and_then(|value| value.model_id.clone())),
        provider_session_id: binding.and_then(|value| value.provider_session_id.clone()),
        provider_session_tree_id: binding.and_then(|value| value.provider_session_tree_id.clone()),
        provider_parent_session_id: binding
            .and_then(|value| value.provider_parent_session_id.clone()),
        provider_forked_from_session_id: binding
            .and_then(|value| value.provider_forked_from_session_id.clone()),
    }
}

fn presentation_phase(
    session: &AgentSessionRecord,
    latest_turn: Option<&AgentTurnRecord>,
    pending_interaction: Option<&AgentInteractionRecord>,
    current_runtime_binding: Option<&AgentSessionRuntimeBindingRecord>,
    latest_runtime_binding: Option<&AgentSessionRuntimeBindingRecord>,
    fresh_until: Option<&str>,
) -> SessionPresentationPhase {
    if session.deleted_at.is_some() {
        return SessionPresentationPhase::Deleted;
    }
    match session.status {
        AgentSessionStatus::Archived => return SessionPresentationPhase::Archived,
        AgentSessionStatus::Closed => return SessionPresentationPhase::Closed,
        AgentSessionStatus::Active | AgentSessionStatus::Idle => {}
    }
    if pending_interaction.is_some() {
        return SessionPresentationPhase::AwaitingInput;
    }
    if latest_binding_failure_is_effective(
        current_runtime_binding.is_some(),
        latest_runtime_binding.map(|binding| binding.status),
        latest_runtime_binding.map(|binding| binding.updated_at.as_str()),
        latest_turn.map(|turn| turn.updated_at.as_str()),
    ) {
        return SessionPresentationPhase::Failed;
    }
    if latest_turn.is_some_and(|turn| {
        matches!(
            turn.status,
            AgentTurnStatus::Requested | AgentTurnStatus::Running
        ) && fresh_until.is_some_and(timestamp_is_expired)
    }) {
        return SessionPresentationPhase::Unknown;
    }
    match latest_turn.map(|turn| turn.status) {
        Some(AgentTurnStatus::Requested) => SessionPresentationPhase::Queued,
        Some(AgentTurnStatus::Running) => SessionPresentationPhase::Running,
        Some(AgentTurnStatus::Completed) => SessionPresentationPhase::Completed,
        Some(AgentTurnStatus::Failed) => SessionPresentationPhase::Failed,
        Some(AgentTurnStatus::Cancelled) => SessionPresentationPhase::Cancelled,
        None if current_runtime_binding
            .and_then(|binding| binding.provider_session_id.as_deref())
            .is_some() =>
        {
            SessionPresentationPhase::Unknown
        }
        None if session.status == AgentSessionStatus::Idle => SessionPresentationPhase::Idle,
        None => SessionPresentationPhase::Ready,
    }
}

fn latest_binding_failure_is_effective(
    has_current_binding: bool,
    latest_binding_status: Option<AgentSessionRuntimeBindingStatus>,
    latest_binding_updated_at: Option<&str>,
    latest_turn_updated_at: Option<&str>,
) -> bool {
    if has_current_binding
        || latest_binding_status != Some(AgentSessionRuntimeBindingStatus::Failed)
    {
        return false;
    }
    let Some(binding_updated_at) = latest_binding_updated_at else {
        return false;
    };
    let Some(turn_updated_at) = latest_turn_updated_at else {
        return true;
    };
    let Ok(binding_updated_at) =
        parse_rfc3339_datetime(binding_updated_at, "latest RuntimeBinding updatedAt")
    else {
        return false;
    };
    let Ok(turn_updated_at) = parse_rfc3339_datetime(turn_updated_at, "latest Turn updatedAt")
    else {
        return false;
    };
    binding_updated_at >= turn_updated_at
}

fn effective_activity_timing(
    latest_turn: Option<&AgentTurnRecord>,
    pending_interaction: Option<&AgentInteractionRecord>,
    activity_at: &str,
) -> (Option<String>, Option<String>) {
    if let Some(interaction) = pending_interaction {
        return (Some(interaction.updated_at.clone()), None);
    }
    if let Some(turn) = latest_turn {
        let fresh_until = match turn.status {
            AgentTurnStatus::Requested | AgentTurnStatus::Running => turn.lease_expires_at.clone(),
            AgentTurnStatus::Completed | AgentTurnStatus::Failed | AgentTurnStatus::Cancelled => {
                None
            }
        };
        return (Some(turn.updated_at.clone()), fresh_until);
    }
    (Some(activity_at.to_string()), None)
}

fn timestamp_is_expired(value: &str) -> bool {
    parse_rfc3339_datetime(value, "activity freshUntil")
        .map(|fresh_until| fresh_until <= OffsetDateTime::now_utc())
        .unwrap_or(true)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionActivityCursor {
    pub activity_at: String,
    pub session_internal_id: u64,
    pub scope_fingerprint: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SessionActivityCursorPayload {
    version: u8,
    activity_at: String,
    session_internal_id: String,
    scope_fingerprint: String,
}

pub(crate) fn encode_session_activity_cursor(
    cursor: &SessionActivityCursor,
) -> KernelResult<String> {
    let payload = SessionActivityCursorPayload {
        version: SESSION_ACTIVITY_CURSOR_VERSION,
        activity_at: cursor.activity_at.clone(),
        session_internal_id: cursor.session_internal_id.to_string(),
        scope_fingerprint: cursor.scope_fingerprint.clone(),
    };
    let json = serde_json::to_vec(&payload).map_err(|error| KernelError::Internal {
        message: format!("failed to encode session activity cursor: {error}"),
    })?;
    Ok(base64url_encode(&json))
}

pub(crate) fn decode_session_activity_cursor(value: &str) -> KernelResult<SessionActivityCursor> {
    let decoded = base64url_decode(value)
        .ok_or_else(|| KernelError::validation("cursor is not a valid opaque token"))?;
    let payload: SessionActivityCursorPayload = serde_json::from_slice(&decoded)
        .map_err(|_| KernelError::validation("cursor is not a valid opaque token"))?;
    if payload.version != SESSION_ACTIVITY_CURSOR_VERSION {
        return Err(KernelError::validation("cursor version is not supported"));
    }
    parse_rfc3339_datetime(&payload.activity_at, "cursor activityAt")?;
    let session_internal_id = payload
        .session_internal_id
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| KernelError::validation("cursor is not a valid opaque token"))?;
    Ok(SessionActivityCursor {
        activity_at: payload.activity_at,
        session_internal_id,
        scope_fingerprint: payload.scope_fingerprint,
    })
}

pub(crate) fn session_activity_scope_fingerprint(
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    workspace_id: Option<&str>,
    project_id: Option<&str>,
    agent_id: Option<&str>,
) -> String {
    let scope = serde_json::json!({
        "version": 1,
        "tenantId": tenant_id.to_string(),
        "organizationId": organization_id.to_string(),
        "ownerUserId": owner_user_id.to_string(),
        "workspaceId": workspace_id,
        "projectId": project_id,
        "agentId": agent_id,
    });
    sha256_hash(scope.to_string().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_activity_cursor_is_opaque_and_round_trips() {
        let cursor = SessionActivityCursor {
            activity_at: "2026-07-27T09:00:00Z".to_string(),
            session_internal_id: 42,
            scope_fingerprint: "scope-fingerprint".to_string(),
        };
        let encoded = encode_session_activity_cursor(&cursor).expect("encode cursor");
        assert!(!encoded.contains("2026-07-27"));
        assert_eq!(decode_session_activity_cursor(&encoded).unwrap(), cursor);
        assert!(decode_session_activity_cursor("42").is_err());
    }

    fn fresh_provider_observation(
        state: SessionProviderActivityState,
        interaction_hint: Option<SessionProviderActivityInteractionHint>,
    ) -> SessionProviderActivityObservation {
        SessionProviderActivityObservation {
            provider_session_id: "provider.activity.test".to_string(),
            state: Some(state),
            freshness: SessionProviderActivityFreshness::Fresh,
            evidence_kind: Some(SessionProviderActivityEvidenceKind::ProviderEvent),
            interaction_hint,
            observed_at: Some("2099-07-27T09:00:00Z".to_string()),
            fresh_until: Some("2099-07-27T09:05:00Z".to_string()),
        }
    }

    #[test]
    fn fresh_provider_activity_overrides_settled_non_terminal_phases() {
        assert_eq!(
            authoritative_provider_phase(
                SessionPresentationPhase::Completed,
                &fresh_provider_observation(SessionProviderActivityState::Working, None),
            ),
            Some(SessionPresentationPhase::Running)
        );
        assert_eq!(
            authoritative_provider_phase(
                SessionPresentationPhase::Ready,
                &fresh_provider_observation(
                    SessionProviderActivityState::Waiting,
                    Some(SessionProviderActivityInteractionHint::UserInputRequired),
                ),
            ),
            Some(SessionPresentationPhase::AwaitingInput)
        );
        assert_eq!(
            authoritative_provider_phase(
                SessionPresentationPhase::Idle,
                &fresh_provider_observation(SessionProviderActivityState::Failed, None),
            ),
            Some(SessionPresentationPhase::Failed)
        );
    }

    #[test]
    fn provider_activity_does_not_override_higher_priority_persisted_phases() {
        let working = fresh_provider_observation(SessionProviderActivityState::Working, None);
        for phase in [
            SessionPresentationPhase::AwaitingInput,
            SessionPresentationPhase::Queued,
            SessionPresentationPhase::Running,
            SessionPresentationPhase::Failed,
            SessionPresentationPhase::Closed,
            SessionPresentationPhase::Archived,
            SessionPresentationPhase::Deleted,
        ] {
            assert_eq!(
                authoritative_provider_phase(phase, &working),
                None,
                "{phase:?}"
            );
        }

        let mut expired = working;
        expired.fresh_until = Some("2000-01-01T00:00:00Z".to_string());
        assert_eq!(
            authoritative_provider_phase(SessionPresentationPhase::Ready, &expired),
            None
        );

        let mut unbounded = fresh_provider_observation(SessionProviderActivityState::Working, None);
        unbounded.observed_at = None;
        unbounded.fresh_until = None;
        assert_eq!(
            authoritative_provider_phase(SessionPresentationPhase::Ready, &unbounded),
            None
        );
    }

    #[test]
    fn binding_failure_and_turn_phase_are_ordered_by_evidence_time() {
        assert!(!latest_binding_failure_is_effective(
            false,
            Some(AgentSessionRuntimeBindingStatus::Failed),
            Some("2026-07-27T09:00:00Z"),
            Some("2026-07-27T09:01:00Z"),
        ));
        assert!(latest_binding_failure_is_effective(
            false,
            Some(AgentSessionRuntimeBindingStatus::Failed),
            Some("2026-07-27T09:02:00Z"),
            Some("2026-07-27T09:01:00Z"),
        ));
        assert!(!latest_binding_failure_is_effective(
            false,
            Some(AgentSessionRuntimeBindingStatus::Deactivated),
            Some("2026-07-27T09:02:00Z"),
            Some("2026-07-27T09:01:00Z"),
        ));
        assert!(!latest_binding_failure_is_effective(
            true,
            Some(AgentSessionRuntimeBindingStatus::Failed),
            Some("2026-07-27T09:02:00Z"),
            Some("2026-07-27T09:01:00Z"),
        ));
    }

    #[test]
    fn provider_snapshot_validation_fails_closed_at_the_service_boundary() {
        let valid = SessionActivitySnapshot {
            provider_session_id: "provider.expected".to_string(),
            state: Some(KernelActivityState::Working),
            freshness: KernelActivityFreshness::Fresh,
            evidence_kind: Some(KernelActivityEvidenceKind::ProviderEvent),
            interaction_hint: None,
            observed_at: Some("2099-07-27T09:00:00Z".to_string()),
            fresh_until: Some("2099-07-27T09:00:30Z".to_string()),
        };
        assert_eq!(
            SessionProviderActivityObservation::from_provider_snapshot(
                "provider.expected",
                valid.clone(),
            )
            .freshness,
            SessionProviderActivityFreshness::Fresh
        );
        assert_eq!(
            SessionProviderActivityObservation::from_provider_snapshot("provider.other", valid)
                .freshness,
            SessionProviderActivityFreshness::Unavailable
        );

        let incomplete = SessionActivitySnapshot {
            provider_session_id: "provider.expected".to_string(),
            state: Some(KernelActivityState::Working),
            freshness: KernelActivityFreshness::Fresh,
            evidence_kind: None,
            interaction_hint: None,
            observed_at: None,
            fresh_until: None,
        };
        assert_eq!(
            SessionProviderActivityObservation::from_provider_snapshot(
                "provider.expected",
                incomplete,
            )
            .freshness,
            SessionProviderActivityFreshness::Stale
        );
    }
}
