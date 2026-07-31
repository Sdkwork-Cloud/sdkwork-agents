use super::{
    format_utc_seconds, reject_secret_material, validate_optional_bounded, validate_runtime_token,
    AgentsService,
};
use crate::agent_turn::AgentTurnMode;
use crate::agent_turn_input_queue::{
    AgentTurnInputQueueDriveRef, AgentTurnInputQueueEntry, AgentTurnInputQueueStatus,
    TurnInputQueueClaimOutcome, TurnInputQueueClaimRequest, TurnInputQueueListQuery,
    TurnInputQueueReorderEntry, MAX_TURN_INPUT_QUEUE_DRIVE_REFS,
};
use crate::ports::{
    offset_paginated_result, AgentAuditSink, AgentRepository, PaginatedResult,
    MAX_TURN_INPUT_CONTENT_BYTES,
};
use crate::validation::{
    default_plain_text_if_blank, require_non_blank, validate_requested_at, validate_standard_id,
};
use sdkwork_agent_kernel::{KernelError, KernelResult, PolicyProvider, PolicySubject};
use sdkwork_utils_rust::{sha256_hash, trim};
use std::collections::HashSet;
use time::{Duration, OffsetDateTime};

const MAX_ATTACHMENT_NAMES: usize = 64;
const MAX_ATTACHMENT_NAME_BYTES: usize = 256;
const MAX_QUEUE_CLAIM_SECONDS: u32 = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTurnInputQueueEntryCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub queue_entry_id: Option<String>,
    pub content: String,
    pub display_text: String,
    pub content_type: String,
    pub attachment_names: Vec<String>,
    pub drive_refs: Vec<AgentTurnInputQueueDriveRef>,
    pub turn_mode: AgentTurnMode,
    pub runtime_binding_id: Option<String>,
    pub requested_model_id: Option<String>,
    pub access_mode_id: Option<String>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListTurnInputQueueEntriesCommand {
    pub query: TurnInputQueueListQuery,
    pub path_agent_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateTurnInputQueueEntryCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub queue_entry_id: String,
    pub content: String,
    pub display_text: String,
    pub content_type: String,
    pub attachment_names: Vec<String>,
    pub drive_refs: Vec<AgentTurnInputQueueDriveRef>,
    pub turn_mode: AgentTurnMode,
    pub runtime_binding_id: Option<String>,
    pub requested_model_id: Option<String>,
    pub access_mode_id: Option<String>,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveTurnInputQueueEntryCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub queue_entry_id: String,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearTurnInputQueueEntriesCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReorderTurnInputQueueEntriesCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub entries: Vec<TurnInputQueueReorderEntry>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimNextTurnInputQueueEntryCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub claim_owner: String,
    pub lease_seconds: u32,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimNextTurnInputQueueEntryResult {
    Claimed {
        entry: AgentTurnInputQueueEntry,
        claim_token: String,
    },
    Busy(AgentTurnInputQueueEntry),
    Blocked(AgentTurnInputQueueEntry),
    ActiveTurn,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailTurnInputQueueEntryCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub queue_entry_id: String,
    pub expected_version: u64,
    pub fencing_token: u64,
    pub claim_token: String,
    pub error_code: String,
    pub error_detail: Option<String>,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryTurnInputQueueEntryCommand {
    pub tenant_id: u64,
    pub organization_id: u64,
    pub path_agent_id: String,
    pub session_id: String,
    pub queue_entry_id: String,
    pub expected_version: u64,
    pub owner_scope: Option<u64>,
    pub requested_by: PolicySubject,
    pub requested_at: String,
}

fn normalize_attachment_names(values: Vec<String>) -> KernelResult<Vec<String>> {
    if values.len() > MAX_ATTACHMENT_NAMES {
        return Err(KernelError::validation("attachmentNames exceeds 64 items"));
    }
    let mut seen = HashSet::with_capacity(values.len());
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let value = trim(&value).to_string();
        if value.is_empty() || value.len() > MAX_ATTACHMENT_NAME_BYTES {
            return Err(KernelError::validation(
                "attachmentNames contains an invalid name",
            ));
        }
        if seen.insert(value.clone()) {
            normalized.push(value);
        }
    }
    Ok(normalized)
}

fn normalize_queue_drive_refs(
    values: Vec<AgentTurnInputQueueDriveRef>,
) -> KernelResult<Vec<AgentTurnInputQueueDriveRef>> {
    if values.len() > MAX_TURN_INPUT_QUEUE_DRIVE_REFS {
        return Err(KernelError::validation("driveRefs exceeds 64 items"));
    }
    let mut seen = HashSet::with_capacity(values.len());
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        let drive_space_id = trim(&value.drive_space_id).to_string();
        let drive_node_id = trim(&value.drive_node_id).to_string();
        if drive_space_id.is_empty()
            || drive_space_id.len() > 128
            || drive_node_id.is_empty()
            || drive_node_id.len() > 128
        {
            return Err(KernelError::validation(
                "driveRefs contains an invalid reference",
            ));
        }
        let key = (
            value.resource_role.as_str(),
            drive_space_id.clone(),
            drive_node_id.clone(),
        );
        if seen.insert(key) {
            normalized.push(AgentTurnInputQueueDriveRef {
                resource_role: value.resource_role,
                drive_space_id,
                drive_node_id,
            });
        }
    }
    Ok(normalized)
}

struct NormalizedQueuePayload {
    content: String,
    display_text: String,
    content_type: String,
    attachment_names: Vec<String>,
    drive_refs: Vec<AgentTurnInputQueueDriveRef>,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    access_mode_id: Option<String>,
    payload_hash: String,
}

struct QueuePayloadInput {
    content: String,
    display_text: String,
    content_type: String,
    attachment_names: Vec<String>,
    drive_refs: Vec<AgentTurnInputQueueDriveRef>,
    runtime_binding_id: Option<String>,
    requested_model_id: Option<String>,
    access_mode_id: Option<String>,
    turn_mode: AgentTurnMode,
}

fn normalize_queue_payload(input: QueuePayloadInput) -> KernelResult<NormalizedQueuePayload> {
    let content = trim(&input.content).to_string();
    require_non_blank(&content, "content")?;
    if content.len() > MAX_TURN_INPUT_CONTENT_BYTES {
        return Err(KernelError::validation(format!(
            "content exceeds maximum size of {MAX_TURN_INPUT_CONTENT_BYTES} bytes"
        )));
    }
    reject_secret_material(&content, "content")?;
    let display_text = trim(&input.display_text).to_string();
    if display_text.len() > MAX_TURN_INPUT_CONTENT_BYTES {
        return Err(KernelError::validation(
            "displayText exceeds the Turn input limit",
        ));
    }
    let attachment_names = normalize_attachment_names(input.attachment_names)?;
    let drive_refs = normalize_queue_drive_refs(input.drive_refs)?;
    validate_optional_bounded(&input.runtime_binding_id, "runtimeBindingId", 128)?;
    validate_optional_bounded(&input.requested_model_id, "requestedModelId", 128)?;
    validate_optional_bounded(&input.access_mode_id, "accessModeId", 64)?;
    let drive_ref_hash_input = drive_refs
        .iter()
        .map(|value| {
            serde_json::json!({
                "resourceRole": value.resource_role.as_str(),
                "driveSpaceId": value.drive_space_id,
                "driveNodeId": value.drive_node_id,
            })
        })
        .collect::<Vec<_>>();
    let hash_input = serde_json::to_vec(&serde_json::json!({
        "content": content,
        "contentType": default_plain_text_if_blank(&input.content_type),
        "turnMode": input.turn_mode.as_str(),
        "runtimeBindingId": input.runtime_binding_id,
        "requestedModelId": input.requested_model_id,
        "accessModeId": input.access_mode_id,
        "driveRefs": drive_ref_hash_input,
    }))
    .map_err(|error| KernelError::Internal {
        message: format!("failed to encode queued Turn payload: {error}"),
    })?;
    Ok(NormalizedQueuePayload {
        content,
        display_text,
        content_type: default_plain_text_if_blank(&input.content_type),
        attachment_names,
        drive_refs,
        runtime_binding_id: input.runtime_binding_id,
        requested_model_id: input.requested_model_id,
        access_mode_id: input.access_mode_id,
        payload_hash: sha256_hash(&hash_input),
    })
}

impl<R, A, P> AgentsService<R, A, P>
where
    R: AgentRepository,
    A: AgentAuditSink,
    P: PolicyProvider,
{
    pub fn create_turn_input_queue_entry(
        &self,
        command: CreateTurnInputQueueEntryCommand,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        validate_requested_at(&command.requested_at)?;
        validate_standard_id(&command.path_agent_id, "agentId", Some("agent."))?;
        validate_standard_id(&command.session_id, "sessionId", Some("session."))?;
        self.authorize(
            "agent.business.turn.create",
            command.requested_by,
            format!(
                "agent.business.session.{}.turn-input-queue",
                command.session_id
            ),
            "turn.create",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        if !session.status.is_active() {
            return Err(KernelError::validation("session is not active"));
        }
        let normalized = normalize_queue_payload(QueuePayloadInput {
            content: command.content,
            display_text: command.display_text,
            content_type: command.content_type,
            attachment_names: command.attachment_names,
            drive_refs: command.drive_refs,
            runtime_binding_id: command.runtime_binding_id,
            requested_model_id: command.requested_model_id,
            access_mode_id: command.access_mode_id,
            turn_mode: command.turn_mode,
        })?;
        let id = self.repository.next_id()?;
        let queue_entry_id = command
            .queue_entry_id
            .unwrap_or_else(|| format!("queue-entry.{id}"));
        validate_standard_id(&queue_entry_id, "queueEntryId", Some("queue-entry."))?;
        let idempotency_key = format!("{queue_entry_id}.v0");
        self.repository
            .insert_turn_input_queue_entry(AgentTurnInputQueueEntry {
                id,
                queue_entry_id: queue_entry_id.clone(),
                tenant_id: command.tenant_id,
                organization_id: command.organization_id,
                session_id: command.session_id,
                agent_id: command.path_agent_id,
                owner_user_id: session.owner_user_id,
                content: normalized.content,
                display_text: normalized.display_text,
                content_type: normalized.content_type,
                attachment_names: normalized.attachment_names,
                drive_refs: normalized.drive_refs,
                turn_mode: command.turn_mode,
                runtime_binding_id: normalized.runtime_binding_id,
                requested_model_id: normalized.requested_model_id,
                access_mode_id: normalized.access_mode_id,
                idempotency_key,
                payload_hash: normalized.payload_hash,
                client_request_id: queue_entry_id,
                position: 0,
                status: AgentTurnInputQueueStatus::Queued,
                claim_owner: None,
                claim_token_hash: None,
                claim_expires_at: None,
                fencing_token: 0,
                error_code: None,
                error_detail: None,
                version: 0,
                created_at: command.requested_at.clone(),
                updated_at: command.requested_at,
                claimed_at: None,
                failed_at: None,
            })
    }

    pub fn list_turn_input_queue_entries(
        &self,
        command: ListTurnInputQueueEntriesCommand,
    ) -> KernelResult<PaginatedResult<AgentTurnInputQueueEntry>> {
        self.authorize(
            "agent.business.turn.list",
            command.requested_by,
            format!(
                "agent.business.session.{}.turn-input-queue",
                command.query.session_id
            ),
            "turn.list",
        )?;
        let session = self.load_session_for_nested_route(
            command.query.tenant_id,
            command.query.organization_id,
            &command.query.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        let total_count = self
            .repository
            .count_turn_input_queue_entries(&command.query, session.owner_user_id)?;
        let items = self
            .repository
            .list_turn_input_queue_entries(&command.query, session.owner_user_id)?;
        Ok(offset_paginated_result(
            items,
            &command.query.pagination,
            total_count,
        ))
    }

    pub fn update_turn_input_queue_entry(
        &self,
        command: UpdateTurnInputQueueEntryCommand,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        validate_requested_at(&command.requested_at)?;
        self.authorize(
            "agent.business.turn.create",
            command.requested_by,
            format!("agent.business.turn-input-queue.{}", command.queue_entry_id),
            "turn.create",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        validate_standard_id(
            &command.queue_entry_id,
            "queueEntryId",
            Some("queue-entry."),
        )?;
        let mut entry = self
            .repository
            .get_turn_input_queue_entry(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                session.owner_user_id,
                &command.queue_entry_id,
            )?
            .ok_or_else(|| KernelError::validation("queued Turn input not found"))?;
        if entry.status == AgentTurnInputQueueStatus::Executing {
            return Err(KernelError::conflict(
                "executing queued Turn input cannot be edited",
            ));
        }
        if entry.version != command.expected_version {
            return Err(KernelError::conflict("queued Turn input version mismatch"));
        }
        let normalized = normalize_queue_payload(QueuePayloadInput {
            content: command.content,
            display_text: command.display_text,
            content_type: command.content_type,
            attachment_names: command.attachment_names,
            drive_refs: command.drive_refs,
            runtime_binding_id: command.runtime_binding_id,
            requested_model_id: command.requested_model_id,
            access_mode_id: command.access_mode_id,
            turn_mode: command.turn_mode,
        })?;
        entry.content = normalized.content;
        entry.display_text = normalized.display_text;
        entry.content_type = normalized.content_type;
        entry.attachment_names = normalized.attachment_names;
        entry.drive_refs = normalized.drive_refs;
        entry.turn_mode = command.turn_mode;
        entry.runtime_binding_id = normalized.runtime_binding_id;
        entry.requested_model_id = normalized.requested_model_id;
        entry.access_mode_id = normalized.access_mode_id;
        entry.payload_hash = normalized.payload_hash;
        entry.version = entry.version.saturating_add(1);
        entry.idempotency_key = format!("{}.v{}", entry.queue_entry_id, entry.version);
        entry.client_request_id = entry.idempotency_key.clone();
        entry.status = AgentTurnInputQueueStatus::Queued;
        entry.error_code = None;
        entry.error_detail = None;
        entry.failed_at = None;
        entry.updated_at = command.requested_at;
        self.repository
            .update_turn_input_queue_entry(entry, command.expected_version)
    }

    pub fn remove_turn_input_queue_entry(
        &self,
        command: RemoveTurnInputQueueEntryCommand,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        self.authorize(
            "agent.business.turn.cancel",
            command.requested_by,
            format!("agent.business.turn-input-queue.{}", command.queue_entry_id),
            "turn.cancel",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        self.repository.remove_turn_input_queue_entry(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            session.owner_user_id,
            &command.queue_entry_id,
            command.expected_version,
        )
    }

    pub fn clear_turn_input_queue_entries(
        &self,
        command: ClearTurnInputQueueEntriesCommand,
    ) -> KernelResult<u64> {
        self.authorize(
            "agent.business.turn.cancel",
            command.requested_by,
            format!(
                "agent.business.session.{}.turn-input-queue",
                command.session_id
            ),
            "turn.cancel",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        self.repository.clear_turn_input_queue_entries(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            session.owner_user_id,
        )
    }

    pub fn reorder_turn_input_queue_entries(
        &self,
        command: ReorderTurnInputQueueEntriesCommand,
    ) -> KernelResult<Vec<AgentTurnInputQueueEntry>> {
        validate_requested_at(&command.requested_at)?;
        if command.entries.len() > crate::MAX_TURN_INPUT_QUEUE_ENTRIES_PER_SESSION {
            return Err(KernelError::validation("orderedEntries exceeds 32 items"));
        }
        let mut ids = HashSet::with_capacity(command.entries.len());
        if command
            .entries
            .iter()
            .any(|entry| !ids.insert(entry.queue_entry_id.as_str()))
        {
            return Err(KernelError::validation(
                "orderedEntries contains duplicate ids",
            ));
        }
        self.authorize(
            "agent.business.turn.create",
            command.requested_by,
            format!(
                "agent.business.session.{}.turn-input-queue",
                command.session_id
            ),
            "turn.create",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        self.repository.reorder_turn_input_queue_entries(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            session.owner_user_id,
            &command.entries,
            &command.requested_at,
        )
    }

    pub fn claim_next_turn_input_queue_entry(
        &self,
        command: ClaimNextTurnInputQueueEntryCommand,
    ) -> KernelResult<ClaimNextTurnInputQueueEntryResult> {
        validate_requested_at(&command.requested_at)?;
        validate_runtime_token(&command.claim_owner, "claimOwner", 128)?;
        if !(1..=MAX_QUEUE_CLAIM_SECONDS).contains(&command.lease_seconds) {
            return Err(KernelError::validation(
                "leaseSeconds must be between 1 and 300",
            ));
        }
        self.authorize(
            "agent.business.turn.create",
            command.requested_by,
            format!(
                "agent.business.session.{}.turn-input-queue",
                command.session_id
            ),
            "turn.create",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        let raw_claim_token = sdkwork_utils_rust::id::random_string(48);
        let claim_token_hash = sha256_hash(raw_claim_token.as_bytes());
        let now = OffsetDateTime::now_utc();
        let claim_expires_at =
            format_utc_seconds(now + Duration::seconds(i64::from(command.lease_seconds)));
        let outcome =
            self.repository
                .claim_next_turn_input_queue_entry(&TurnInputQueueClaimRequest {
                    tenant_id: command.tenant_id,
                    organization_id: command.organization_id,
                    session_id: command.session_id,
                    owner_user_id: session.owner_user_id,
                    claim_owner: command.claim_owner,
                    claim_token_hash,
                    claim_expires_at,
                    requested_at: format_utc_seconds(now),
                })?;
        Ok(match outcome {
            TurnInputQueueClaimOutcome::Claimed(entry) => {
                ClaimNextTurnInputQueueEntryResult::Claimed {
                    entry,
                    claim_token: raw_claim_token,
                }
            }
            TurnInputQueueClaimOutcome::Busy(entry) => {
                ClaimNextTurnInputQueueEntryResult::Busy(entry)
            }
            TurnInputQueueClaimOutcome::Blocked(entry) => {
                ClaimNextTurnInputQueueEntryResult::Blocked(entry)
            }
            TurnInputQueueClaimOutcome::ActiveTurn => {
                ClaimNextTurnInputQueueEntryResult::ActiveTurn
            }
            TurnInputQueueClaimOutcome::Empty => ClaimNextTurnInputQueueEntryResult::Empty,
        })
    }

    pub fn fail_turn_input_queue_entry(
        &self,
        command: FailTurnInputQueueEntryCommand,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        validate_requested_at(&command.requested_at)?;
        require_non_blank(&command.error_code, "errorCode")?;
        if command.error_code.len() > 128
            || command
                .error_detail
                .as_deref()
                .is_some_and(|v| v.len() > 1024)
        {
            return Err(KernelError::validation(
                "queue failure detail exceeds its limit",
            ));
        }
        if let Some(error_detail) = command.error_detail.as_deref() {
            reject_secret_material(error_detail, "errorDetail")?;
        }
        if !(32..=256).contains(&command.claim_token.len()) {
            return Err(KernelError::validation("claimToken is invalid"));
        }
        self.authorize(
            "agent.business.turn.create",
            command.requested_by,
            format!("agent.business.turn-input-queue.{}", command.queue_entry_id),
            "turn.create",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        self.repository.fail_turn_input_queue_entry(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            session.owner_user_id,
            &command.queue_entry_id,
            command.expected_version,
            command.fencing_token,
            &sha256_hash(command.claim_token.as_bytes()),
            &command.error_code,
            command.error_detail.as_deref(),
            &command.requested_at,
        )
    }

    pub fn retry_turn_input_queue_entry(
        &self,
        command: RetryTurnInputQueueEntryCommand,
    ) -> KernelResult<AgentTurnInputQueueEntry> {
        validate_requested_at(&command.requested_at)?;
        self.authorize(
            "agent.business.turn.create",
            command.requested_by,
            format!("agent.business.turn-input-queue.{}", command.queue_entry_id),
            "turn.create",
        )?;
        let session = self.load_session_for_nested_route(
            command.tenant_id,
            command.organization_id,
            &command.session_id,
            &command.path_agent_id,
            command.owner_scope,
        )?;
        let mut entry = self
            .repository
            .get_turn_input_queue_entry(
                command.tenant_id,
                command.organization_id,
                &command.session_id,
                session.owner_user_id,
                &command.queue_entry_id,
            )?
            .ok_or_else(|| KernelError::validation("queued Turn input not found"))?;
        if entry.status != AgentTurnInputQueueStatus::Failed {
            return Err(KernelError::conflict(
                "only failed queued Turn input can be retried",
            ));
        }
        if entry.version != command.expected_version {
            return Err(KernelError::conflict("queued Turn input version mismatch"));
        }
        entry.version = entry.version.saturating_add(1);
        entry.idempotency_key = format!("{}.v{}", entry.queue_entry_id, entry.version);
        entry.client_request_id = entry.idempotency_key.clone();
        entry.status = AgentTurnInputQueueStatus::Queued;
        entry.error_code = None;
        entry.error_detail = None;
        entry.failed_at = None;
        entry.updated_at = command.requested_at;
        self.repository
            .update_turn_input_queue_entry(entry, command.expected_version)
    }
}
