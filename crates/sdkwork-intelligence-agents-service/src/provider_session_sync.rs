use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use sdkwork_agent_kernel::{
    AgentMessage, AgentMessageRole, AgentPart, AgentPartKind, KernelError, KernelErrorKind,
    KernelResult, PolicySubject, SessionKind,
};
use sdkwork_agents_runtime_facade::{
    AgentsSessionActor, AgentsSessionEntrySurface, AgentsSessionFacade, AgentsSessionKind,
    AgentsSessionRuntimeBindingDescriptor, ProviderSessionInventoryItem,
    ProviderSessionInventorySelector, ResolveAgentsSessionRequest, RuntimeFacadeError,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::application::{
    GetSessionCommand, ListSessionRuntimeBindingsCommand,
    ReconcileProviderSessionHistoryItemCommand,
};
use crate::domain::{AgentSessionItemKind, AgentSessionItemStatus};
use crate::http::{HttpAgentsSessionFacade, HttpService};
use crate::ports::{PaginationParams, SessionRuntimeBindingListQuery};
use crate::project::AgentProjectRecord;
use crate::runtime_facade_bridge::shared_code_engine_host;

const PROVIDER_SESSION_TITLE_MAX_BYTES: usize = 512;
const PROVIDER_SESSION_RECONCILIATION_MAX_ITEMS: usize = 10_000;
const PROVIDER_SESSION_RECONCILIATION_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderSessionSynchronizationIssueDisposition {
    Skipped,
    Failed,
}

impl ProviderSessionSynchronizationIssueDisposition {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Skipped => "skipped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderSessionSynchronizationIssue {
    pub(crate) code: &'static str,
    pub(crate) count: usize,
    pub(crate) disposition: ProviderSessionSynchronizationIssueDisposition,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ProviderSessionSynchronizationResult {
    pub(crate) failed_session_count: usize,
    pub(crate) issues: Vec<ProviderSessionSynchronizationIssue>,
    pub(crate) skipped_session_count: usize,
    pub(crate) synchronized_session_count: usize,
}

impl ProviderSessionSynchronizationResult {
    fn record_issue(
        &mut self,
        code: &'static str,
        disposition: ProviderSessionSynchronizationIssueDisposition,
        count: usize,
    ) {
        if count == 0 {
            return;
        }
        if let Some(issue) = self
            .issues
            .iter_mut()
            .find(|issue| issue.code == code && issue.disposition == disposition)
        {
            issue.count += count;
        } else {
            self.issues.push(ProviderSessionSynchronizationIssue {
                code,
                count,
                disposition,
            });
        }
        match disposition {
            ProviderSessionSynchronizationIssueDisposition::Skipped => {
                self.skipped_session_count += count;
            }
            ProviderSessionSynchronizationIssueDisposition::Failed => {
                self.failed_session_count += count;
            }
        }
    }

    fn record_skipped(&mut self, code: &'static str) {
        self.record_issue(
            code,
            ProviderSessionSynchronizationIssueDisposition::Skipped,
            1,
        );
    }

    fn record_failed(&mut self, code: &'static str) {
        self.record_issue(
            code,
            ProviderSessionSynchronizationIssueDisposition::Failed,
            1,
        );
    }
}

pub(crate) fn synchronize_project_provider_sessions(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    let exact_cwd = std::env::current_dir()
        .ok()
        .filter(|cwd| {
            cwd.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|basename| basename.eq_ignore_ascii_case(&project.name))
        })
        .map(|cwd| cwd.to_string_lossy().into_owned());
    synchronize_project_provider_sessions_with_selector(
        service,
        project,
        subject,
        exact_cwd,
        Some(project.name.clone()),
        None,
    )
}

pub(crate) fn synchronize_project_provider_sessions_at_cwd(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    exact_cwd: Option<String>,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    synchronize_project_provider_sessions_with_selector(
        service,
        project,
        subject,
        exact_cwd,
        Some(project.name.clone()),
        None,
    )
}

pub(crate) fn synchronize_project_provider_sessions_with_selector(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    exact_cwd: Option<String>,
    unique_basename: Option<String>,
    directory_fingerprint: Option<String>,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    let Some(host) = shared_code_engine_host() else {
        return Ok(ProviderSessionSynchronizationResult::default());
    };
    let inventory = host
        .discover_provider_sessions(&ProviderSessionInventorySelector {
            directory_fingerprint,
            exact_cwd,
            unique_basename,
        })
        .map_err(runtime_facade_error)?;
    if inventory.is_empty() {
        return Ok(ProviderSessionSynchronizationResult::default());
    }

    synchronize_provider_session_inventory(service, project, subject, inventory)
}

pub(crate) fn synchronize_provider_session_transcript(
    service: &HttpService,
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    agent_id: String,
    session_id: String,
    subject: PolicySubject,
) -> KernelResult<usize> {
    let Some(engine_key) = agent_id.strip_prefix("agent.intelligence.") else {
        return Ok(0);
    };
    if sdkwork_agents_runtime_facade::code_engine_agent_id(engine_key) != Some(agent_id.as_str())
        || !session_id.starts_with(&format!("session.provider.{engine_key}."))
    {
        return Ok(0);
    }
    let session = service.get_session(GetSessionCommand {
        tenant_id,
        organization_id,
        path_agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        owner_scope: Some(owner_user_id),
        requested_by: subject.clone(),
    })?;
    let binding_page =
        service.list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
            query: SessionRuntimeBindingListQuery::for_session(
                tenant_id,
                organization_id,
                session_id.clone(),
            )
            .current_only()
            .with_pagination(PaginationParams::default().with_page_size(20)),
            path_agent_id: agent_id.clone(),
            owner_scope: Some(owner_user_id),
            requested_by: subject.clone(),
        })?;
    let Some(binding) = binding_page.items.into_iter().find(|binding| {
        binding.is_current
            && binding.status.as_str() == "active"
            && binding.transport_kind == "provider-session-history"
            && sdkwork_agents_runtime_facade::code_engine_binding_id(engine_key)
                == Some(binding.provider_binding_id.as_str())
    }) else {
        return Ok(0);
    };
    let Some(provider_session_id) = binding
        .provider_session_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(0);
    };
    let Some(host) = shared_code_engine_host() else {
        return Ok(0);
    };
    let messages = host
        .load_provider_session_messages(engine_key, provider_session_id)
        .map_err(runtime_facade_error)?;
    let mut synchronized = 0;
    let mut tool_calls = HashMap::<String, (String, Option<String>)>::new();
    for message in messages {
        let requested_at = message
            .created_at
            .clone()
            .unwrap_or_else(|| session.updated_at.clone());
        for mut item in provider_session_history_items(engine_key, &message) {
            let item_key = stable_provider_session_item_key(
                engine_key,
                provider_session_id,
                item.provider_item_key.as_str(),
            );
            let item_id = format!("item.provider.{engine_key}.{item_key}");
            if item.kind == AgentSessionItemKind::ToolResult {
                if let Some((parent_item_id, tool_name)) = item
                    .tool_call_id
                    .as_ref()
                    .and_then(|tool_call_id| tool_calls.get(tool_call_id))
                {
                    item.parent_item_id = Some(parent_item_id.clone());
                    if item.tool_name.as_deref().is_none_or(|name| name == "tool") {
                        item.tool_name = tool_name.clone();
                    }
                }
            }
            service.reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    tenant_id,
                    organization_id,
                    session_id: session_id.clone(),
                    item_id: item_id.clone(),
                    kind: item.kind,
                    content: item.content,
                    content_type: item.content_type,
                    status: item.status,
                    model_id: Some(binding.model_id.clone()),
                    provider_id: Some(binding.provider_id.clone()),
                    tool_name: item.tool_name.clone(),
                    tool_call_id: item.tool_call_id.clone(),
                    tool_arguments_json: item.tool_arguments_json,
                    tool_result_json: item.tool_result_json,
                    parent_item_id: item.parent_item_id,
                    requested_by: subject.clone(),
                    requested_at: requested_at.clone(),
                },
                engine_key,
            )?;
            if item.kind == AgentSessionItemKind::ToolCall {
                if let Some(tool_call_id) = item.tool_call_id {
                    tool_calls.insert(tool_call_id, (item_id, item.tool_name));
                }
            }
            synchronized += 1;
        }
    }
    Ok(synchronized)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderSessionHistoryItem {
    provider_item_key: String,
    kind: AgentSessionItemKind,
    content: Option<String>,
    content_type: String,
    status: AgentSessionItemStatus,
    tool_name: Option<String>,
    tool_call_id: Option<String>,
    tool_arguments_json: Option<String>,
    tool_result_json: Option<String>,
    parent_item_id: Option<String>,
}

fn provider_session_history_items(
    engine_key: &str,
    message: &AgentMessage,
) -> Vec<ProviderSessionHistoryItem> {
    let mut legacy_text_item_available = true;
    message
        .parts
        .iter()
        .filter_map(|part| {
            let content_type = provider_part_content_type(engine_key, part);
            let kind = provider_session_item_kind(message.role, part.kind, content_type)?;
            let uses_legacy_message_id = legacy_text_item_available
                && part.kind == AgentPartKind::Text
                && !matches!(kind, AgentSessionItemKind::Reasoning);
            if uses_legacy_message_id {
                legacy_text_item_available = false;
            }
            let provider_item_key = if uses_legacy_message_id {
                message.message_id.clone()
            } else {
                format!("{}\u{0}{}", message.message_id, part.part_id)
            };
            let tool_call_id = part.tool_call_id.clone().or_else(|| {
                provider_part_metadata(engine_key, part, "tool_call_id").map(str::to_string)
            });
            let has_result = provider_part_metadata(engine_key, part, "has_result") == Some("true");
            let status = provider_session_item_status(engine_key, part, kind, has_result);
            let provider_json = part.json.clone();
            let tool_payload = if kind == AgentSessionItemKind::ToolCall
                || kind == AgentSessionItemKind::ToolResult
            {
                provider_json.or_else(|| {
                    Some(
                        serde_json::json!({
                            "id": tool_call_id.as_deref(),
                            "type": kind.as_str(),
                            "name": part.name.as_deref(),
                            "output": part.text.as_deref(),
                        })
                        .to_string(),
                    )
                })
            } else {
                None
            };
            let content = match kind {
                AgentSessionItemKind::ToolCall | AgentSessionItemKind::ToolResult => None,
                AgentSessionItemKind::ArtifactReference | AgentSessionItemKind::StatusNotice => {
                    part.json
                        .clone()
                        .or_else(|| part.content_ref.clone())
                        .or_else(|| part.artifact_id.clone())
                        .or_else(|| part.policy_decision_id.clone())
                        .or_else(|| part.text.clone())
                }
                _ => part.text.clone().or_else(|| part.json.clone()),
            }
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
            let item_content_type = if matches!(
                kind,
                AgentSessionItemKind::ToolCall
                    | AgentSessionItemKind::ToolResult
                    | AgentSessionItemKind::StatusNotice
            ) || part.json.is_some()
            {
                "application/json".to_string()
            } else {
                part.mime_type
                    .clone()
                    .unwrap_or_else(|| "text/plain".to_string())
            };
            Some(ProviderSessionHistoryItem {
                provider_item_key,
                kind,
                content,
                content_type: item_content_type,
                status,
                tool_name: part.name.clone(),
                tool_call_id,
                tool_arguments_json: (kind == AgentSessionItemKind::ToolCall)
                    .then(|| tool_payload.clone())
                    .flatten(),
                tool_result_json: ((kind == AgentSessionItemKind::ToolResult) || has_result)
                    .then_some(tool_payload)
                    .flatten(),
                parent_item_id: None,
            })
        })
        .collect()
}

fn provider_session_item_kind(
    role: AgentMessageRole,
    part_kind: AgentPartKind,
    content_type: Option<&str>,
) -> Option<AgentSessionItemKind> {
    if matches!(content_type, Some("reasoning" | "thinking")) {
        return Some(AgentSessionItemKind::Reasoning);
    }
    if content_type.is_some_and(|value| {
        matches!(
            value,
            "advisor_tool_result"
                | "bash_code_execution_tool_result"
                | "code_execution_tool_result"
                | "function_call_output"
                | "custom_tool_call_output"
                | "mcp_tool_result"
                | "text_editor_code_execution_tool_result"
                | "tool_result"
                | "tool_search_tool_result"
                | "web_fetch_tool_result"
                | "web_search_tool_result"
        )
    }) {
        return Some(AgentSessionItemKind::ToolResult);
    }
    if matches!(
        content_type,
        Some(
            "compaction"
                | "context_compacted"
                | "queue-operation"
                | "step-start"
                | "step-finish"
                | "task_complete"
                | "task_started"
        )
    ) {
        return Some(AgentSessionItemKind::StatusNotice);
    }
    if content_type.is_some_and(|value| {
        matches!(
            value,
            "attachment" | "input_image" | "image" | "document" | "file"
        )
    }) {
        return Some(AgentSessionItemKind::ArtifactReference);
    }

    match part_kind {
        AgentPartKind::Text => match role {
            AgentMessageRole::User => Some(AgentSessionItemKind::UserInput),
            AgentMessageRole::Agent | AgentMessageRole::Model => {
                Some(AgentSessionItemKind::AssistantOutput)
            }
            AgentMessageRole::System | AgentMessageRole::Policy => {
                Some(AgentSessionItemKind::SystemInstruction)
            }
            AgentMessageRole::Tool => Some(AgentSessionItemKind::ToolResult),
            AgentMessageRole::Adapter => Some(AgentSessionItemKind::StatusNotice),
        },
        AgentPartKind::ToolCallRef => Some(AgentSessionItemKind::ToolCall),
        AgentPartKind::Error => Some(AgentSessionItemKind::ErrorNotice),
        AgentPartKind::PolicyDecisionRef => Some(AgentSessionItemKind::StatusNotice),
        AgentPartKind::Json => match role {
            AgentMessageRole::Tool => Some(AgentSessionItemKind::ToolResult),
            _ => Some(AgentSessionItemKind::ArtifactReference),
        },
        AgentPartKind::BinaryRef
        | AgentPartKind::FileRef
        | AgentPartKind::ArtifactRef
        | AgentPartKind::ImageRef
        | AgentPartKind::AudioRef
        | AgentPartKind::VideoRef => Some(AgentSessionItemKind::ArtifactReference),
    }
}

fn provider_session_item_status(
    engine_key: &str,
    part: &AgentPart,
    kind: AgentSessionItemKind,
    has_result: bool,
) -> AgentSessionItemStatus {
    match provider_part_metadata(engine_key, part, "status") {
        Some("pending" | "queued" | "running" | "in_progress") => AgentSessionItemStatus::Pending,
        Some("failed" | "error") => AgentSessionItemStatus::Failed,
        Some("cancelled" | "canceled" | "aborted") => AgentSessionItemStatus::Cancelled,
        Some("completed" | "complete" | "success" | "succeeded") => {
            AgentSessionItemStatus::Completed
        }
        _ if kind == AgentSessionItemKind::ToolCall && !has_result => {
            AgentSessionItemStatus::Pending
        }
        _ => AgentSessionItemStatus::Completed,
    }
}

fn provider_part_content_type<'a>(engine_key: &str, part: &'a AgentPart) -> Option<&'a str> {
    provider_part_metadata(engine_key, part, "content_type")
}

fn provider_part_metadata<'a>(
    engine_key: &str,
    part: &'a AgentPart,
    field_name: &str,
) -> Option<&'a str> {
    let namespace = match engine_key {
        "claude-code" => "claude",
        other => other,
    };
    part.metadata_value(format!("{namespace}.{field_name}").as_str())
}

fn synchronize_provider_session_inventory(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    inventory: Vec<ProviderSessionInventoryItem>,
) -> KernelResult<ProviderSessionSynchronizationResult> {
    let facade =
        HttpAgentsSessionFacade::for_provider_session_history_reconciliation(service.clone());
    let actor = AgentsSessionActor {
        subject_id: subject.subject_id.clone(),
        roles: subject.roles.clone(),
    };
    let started_at = Instant::now();
    let inventory_len = inventory.len();
    let mut result = ProviderSessionSynchronizationResult::default();
    let mut seen_provider_sessions = HashSet::new();
    for (index, item) in inventory.into_iter().enumerate() {
        if index >= PROVIDER_SESSION_RECONCILIATION_MAX_ITEMS {
            result.record_issue(
                "inventory_item_limit_exceeded",
                ProviderSessionSynchronizationIssueDisposition::Failed,
                inventory_len - index,
            );
            break;
        }
        if started_at.elapsed() >= PROVIDER_SESSION_RECONCILIATION_TIMEOUT {
            result.record_issue(
                "synchronization_time_budget_exceeded",
                ProviderSessionSynchronizationIssueDisposition::Failed,
                inventory_len - index,
            );
            break;
        }
        if item.session.kind == SessionKind::Subagent || item.session.parent_session_id.is_some() {
            result.record_skipped("non_root_session");
            continue;
        }
        let provider_id = item.provider_id.trim().to_string();
        let provider_session_id = item.session.session_id.trim().to_string();
        if provider_id.is_empty() || provider_session_id.is_empty() {
            result.record_failed("invalid_provider_session_identity");
            continue;
        }
        let provider_binding_id = item.binding_id.trim().to_string();
        if provider_binding_id.is_empty() {
            result.record_failed("invalid_runtime_binding_identity");
            continue;
        }
        if !seen_provider_sessions.insert((
            provider_binding_id.clone(),
            provider_id.clone(),
            provider_session_id.clone(),
        )) {
            result.record_skipped("duplicate_provider_session");
            continue;
        }
        let requested_at = match provider_session_requested_at(&item, project) {
            Ok(requested_at) => requested_at,
            Err(_) => {
                result.record_failed("invalid_synchronization_timestamp");
                continue;
            }
        };
        if let Err(error) = service.ensure_code_engine_runtime_identity(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
            &item.engine_key,
            &item.agent_id,
            &provider_binding_id,
            &provider_id,
            subject.clone(),
            &requested_at,
        ) {
            if is_fatal_provider_session_synchronization_error(&error) {
                return Err(error);
            }
            record_provider_session_reconciliation_failure(
                project,
                &item.engine_key,
                "runtime_identity_reconciliation_failed",
                &error,
            );
            result.record_failed("runtime_identity_reconciliation_failed");
            continue;
        }
        let stable_key = stable_provider_session_key(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
            &item.engine_key,
            &provider_binding_id,
            &provider_id,
            &provider_session_id,
        );
        let session_id = format!("session.provider.{}.{}", item.engine_key, stable_key);
        let runtime_binding_id = format!(
            "runtime_binding.provider.{}.{}",
            item.engine_key, stable_key
        );
        let title = provider_session_title(item.session.title.as_deref(), &item.engine_key);
        let model_id = item
            .session
            .model
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| item.default_model_id.clone());
        let reconciliation = facade.resolve_or_create_session(ResolveAgentsSessionRequest {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            owner_user_id: project.owner_user_id,
            agent_id: item.agent_id.clone(),
            session_id,
            project_id: Some(project.project_id.clone()),
            session_kind: AgentsSessionKind::Coding,
            entry_surface: AgentsSessionEntrySurface::Pc,
            source_module: Some("birdcoder".to_string()),
            source_context_kind: Some("provider_session".to_string()),
            source_context_id: Some(project.project_id.clone()),
            parent_session_id: None,
            forked_from_turn_id: None,
            title,
            idempotency_key: format!("provider-session:{}:{}", item.engine_key, stable_key),
            payload_hash: format!("provider-session-v1:{}:{}", item.engine_key, stable_key),
            runtime_binding: Some(AgentsSessionRuntimeBindingDescriptor {
                runtime_binding_id,
                runtime_location_id: None,
                host_mode: "server".to_string(),
                transport_kind: "provider-session-history".to_string(),
                provider_binding_id,
                model_id,
                provider_id,
                provider_session_id: Some(provider_session_id.clone()),
                provider_session_tree_id: Some(provider_session_id),
                provider_parent_session_id: item.session.parent_session_id.clone(),
                provider_forked_from_session_id: item.session.forked_from_id.clone(),
            }),
            actor: actor.clone(),
            requested_at,
        });
        if let Err(error) = reconciliation {
            let error = runtime_facade_error(error);
            if is_fatal_provider_session_synchronization_error(&error) {
                return Err(error);
            }
            record_provider_session_reconciliation_failure(
                project,
                &item.engine_key,
                "session_reconciliation_failed",
                &error,
            );
            result.record_failed("session_reconciliation_failed");
            continue;
        }
        result.synchronized_session_count += 1;
    }
    Ok(result)
}

fn is_fatal_provider_session_synchronization_error(error: &KernelError) -> bool {
    matches!(
        error.kind(),
        KernelErrorKind::PermissionRequired
            | KernelErrorKind::PolicyDenied
            | KernelErrorKind::SecurityViolation
    )
}

fn record_provider_session_reconciliation_failure(
    project: &AgentProjectRecord,
    engine_key: &str,
    issue_code: &'static str,
    error: &KernelError,
) {
    tracing::warn!(
        target: "sdkwork.agents.provider_session_sync",
        project_id = %project.project_id,
        engine_key = %engine_key,
        issue_code,
        error_code = error.code(),
        error_kind = error.kind().as_str(),
        "provider session inventory item reconciliation failed"
    );
}

fn provider_session_requested_at(
    item: &ProviderSessionInventoryItem,
    project: &AgentProjectRecord,
) -> KernelResult<String> {
    [
        item.session.updated_at.as_deref(),
        item.session.created_at.as_deref(),
        Some(project.updated_at.as_str()),
        Some(project.created_at.as_str()),
    ]
    .into_iter()
    .flatten()
    .find_map(normalize_provider_session_timestamp)
    .ok_or_else(|| {
        KernelError::validation("provider session inventory has no valid synchronization timestamp")
    })
}

fn normalize_provider_session_timestamp(value: &str) -> Option<String> {
    let value = value.trim();
    if OffsetDateTime::parse(value, &Rfc3339).is_ok() {
        return Some(value.to_string());
    }

    let (date, time) = value.split_once(' ')?;
    let mut candidate = format!("{date}T{time}");
    let offset_index = candidate
        .char_indices()
        .skip(11)
        .filter_map(|(index, character)| matches!(character, '+' | '-').then_some(index))
        .last()?;
    let offset = &candidate[offset_index..];
    if offset.len() == 3
        && offset[1..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        candidate.push_str(":00");
    }

    OffsetDateTime::parse(&candidate, &Rfc3339)
        .ok()
        .map(|_| candidate)
}

fn provider_session_title(value: Option<&str>, engine_key: &str) -> String {
    let compact = value
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let value = if compact.is_empty() {
        format!("{engine_key} session")
    } else {
        compact
    };
    if value.len() <= PROVIDER_SESSION_TITLE_MAX_BYTES {
        return value;
    }

    let mut end = PROVIDER_SESSION_TITLE_MAX_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].trim_end().to_string()
}

fn stable_provider_session_key(
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    engine_key: &str,
    provider_binding_id: &str,
    provider_id: &str,
    provider_session_id: &str,
) -> String {
    let digest = sdkwork_utils_rust::sha256_hash(
        format!(
            "provider-session-v3\u{0}{tenant_id}\u{0}{organization_id}\u{0}{owner_user_id}\u{0}{engine_key}\u{0}{provider_binding_id}\u{0}{provider_id}\u{0}{provider_session_id}"
        )
        .as_bytes(),
    );
    digest[..32].to_string()
}

fn stable_provider_session_item_key(
    engine_key: &str,
    provider_session_id: &str,
    provider_message_id: &str,
) -> String {
    let digest = sdkwork_utils_rust::sha256_hash(
        format!(
            "provider-session-item-v1\u{0}{engine_key}\u{0}{provider_session_id}\u{0}{provider_message_id}"
        )
        .as_bytes(),
    );
    digest[..32].to_string()
}

pub(crate) fn runtime_facade_error(error: RuntimeFacadeError) -> KernelError {
    match error {
        RuntimeFacadeError::InvalidInput(message)
        | RuntimeFacadeError::EngineMismatch {
            slot_engine: message,
            ..
        } => KernelError::validation(message),
        RuntimeFacadeError::UnsupportedEngine { engine_key }
        | RuntimeFacadeError::UnsupportedLiveInteraction { engine_key, .. } => {
            KernelError::validation(format!("unsupported engineId \"{engine_key}\""))
        }
        RuntimeFacadeError::BlankPrompt => KernelError::validation("prompt must not be blank"),
        RuntimeFacadeError::EngineUnavailable { engine_key, .. } => {
            KernelError::ProviderUnavailable {
                provider_id: engine_key,
            }
        }
        RuntimeFacadeError::Kernel(message) | RuntimeFacadeError::Handler(message) => {
            KernelError::Internal { message }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{
        CreateProjectCommand, ListSessionActivitySummariesCommand, ListSessionItemsCommand,
        ListSessionRuntimeBindingsCommand, ListSessionsCommand, UpdateSessionCommand,
    };
    use crate::http::AgentHttpState;
    use crate::infrastructure::{
        IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use crate::ports::{
        PaginationParams, SessionActivitySummaryListQuery, SessionItemListQuery, SessionListQuery,
        SessionRuntimeBindingListQuery,
    };
    use crate::{AgentProjectDriveAccessMode, AgentProjectVisibility};
    use sdkwork_agent_kernel::{AgentSession, SessionKind};
    use sdkwork_agents_runtime_facade::CodeEngineCatalogEngine;

    #[test]
    fn provider_session_ids_are_stable_and_runtime_scoped() {
        let first = stable_provider_session_key(
            100001,
            0,
            42,
            "codex",
            "binding.agent-provider.codex",
            "provider.model.codex",
            "provider-1",
        );
        assert_eq!(
            first,
            stable_provider_session_key(
                100001,
                0,
                42,
                "codex",
                "binding.agent-provider.codex",
                "provider.model.codex",
                "provider-1",
            )
        );
        assert_ne!(
            first,
            stable_provider_session_key(
                100001,
                0,
                42,
                "opencode",
                "binding.agent-provider.opencode",
                "provider.model.opencode",
                "provider-1",
            )
        );
        assert_ne!(
            first,
            stable_provider_session_key(
                100001,
                0,
                43,
                "codex",
                "binding.agent-provider.codex",
                "provider.model.codex",
                "provider-1",
            )
        );
    }

    #[test]
    fn projects_provider_parts_without_flattening_native_tool_payloads() {
        let native_tool = serde_json::json!({
            "type": "tool",
            "callID": "call-1",
            "tool": "mcp__docs__search",
            "state": {
                "status": "completed",
                "input": { "q": "session items" },
                "output": "found"
            }
        });
        let native_tool_json = native_tool.to_string();
        let mut tool_part = AgentPart::tool_call_ref("part-tool", "call-1")
            .with_name("mcp__docs__search")
            .from_provider("opencode")
            .with_metadata("opencode.content_type", "tool")
            .with_metadata("opencode.status", "completed")
            .with_metadata("opencode.has_result", "true");
        tool_part.json = Some(native_tool_json.clone());
        let message = AgentMessage::new(
            "message-1",
            AgentMessageRole::Agent,
            vec![
                AgentPart::text("part-reasoning", "inspect the code")
                    .from_provider("opencode")
                    .with_metadata("opencode.content_type", "reasoning"),
                tool_part,
                AgentPart::text("part-text", "done")
                    .from_provider("opencode")
                    .with_metadata("opencode.content_type", "text"),
                AgentPart::json(
                    "part-step",
                    serde_json::json!({ "type": "step-finish" }).to_string(),
                )
                .from_provider("opencode")
                .with_metadata("opencode.content_type", "step-finish"),
            ],
        );

        let items = provider_session_history_items("opencode", &message);
        assert_eq!(
            items.iter().map(|item| item.kind).collect::<Vec<_>>(),
            vec![
                AgentSessionItemKind::Reasoning,
                AgentSessionItemKind::ToolCall,
                AgentSessionItemKind::AssistantOutput,
                AgentSessionItemKind::StatusNotice,
            ]
        );
        assert_eq!(items[1].status, AgentSessionItemStatus::Completed);
        assert_eq!(
            items[1].tool_arguments_json.as_deref(),
            Some(native_tool_json.as_str())
        );
        assert_eq!(items[1].tool_result_json, items[1].tool_arguments_json);
        assert_eq!(items[2].provider_item_key, "message-1");
        assert_eq!(items[3].content_type, "application/json");
    }

    #[test]
    fn normalizes_postgres_project_timestamp_for_provider_session_fallback() {
        assert_eq!(
            normalize_provider_session_timestamp("2026-07-27 03:11:00+00").as_deref(),
            Some("2026-07-27T03:11:00+00:00")
        );
        assert_eq!(
            normalize_provider_session_timestamp("2026-07-27 03:11:00.123456-07").as_deref(),
            Some("2026-07-27T03:11:00.123456-07:00")
        );
        assert_eq!(
            normalize_provider_session_timestamp("2026-07-27T03:11:00Z").as_deref(),
            Some("2026-07-27T03:11:00Z")
        );
        assert!(normalize_provider_session_timestamp("not-a-timestamp").is_none());
    }

    #[test]
    fn normalizes_provider_session_titles_to_the_service_limit() {
        assert_eq!(
            provider_session_title(Some("  first\n\tsecond  "), "codex"),
            "first second"
        );
        assert_eq!(
            provider_session_title(Some("   "), "codex"),
            "codex session"
        );

        let long_ascii = "a".repeat(PROVIDER_SESSION_TITLE_MAX_BYTES + 100);
        let ascii_title = provider_session_title(Some(&long_ascii), "codex");
        assert_eq!(ascii_title.len(), PROVIDER_SESSION_TITLE_MAX_BYTES);

        let long_unicode = "\u{4f1a}".repeat(200);
        let unicode_title = provider_session_title(Some(&long_unicode), "codex");
        assert!(unicode_title.len() <= PROVIDER_SESSION_TITLE_MAX_BYTES);
        assert!(unicode_title.is_char_boundary(unicode_title.len()));
    }

    fn test_project(state: &AgentHttpState) -> AgentProjectRecord {
        state
            .service
            .create_project(CreateProjectCommand {
                tenant_id: 100_001,
                organization_id: 0,
                project_id: "project.provider-session-inventory".to_string(),
                workspace_id: None,
                owner_user_id: 100,
                name: "provider-session-inventory".to_string(),
                description: None,
                visibility: AgentProjectVisibility::Private,
                drive_access_mode: AgentProjectDriveAccessMode::Disabled,
                default_agent_id: None,
                default_model_id: None,
                requested_by: PolicySubject {
                    subject_id: "100".to_string(),
                    tenant_id: "100001".to_string(),
                    roles: vec![
                        "ai.agents.manage".to_string(),
                        "ai.agents.read".to_string(),
                        "ai.agents.use".to_string(),
                    ],
                },
                requested_at: "2026-07-26T00:00:00Z".to_string(),
            })
            .expect("test project")
    }

    fn read_subject() -> PolicySubject {
        PolicySubject {
            subject_id: "100".to_string(),
            tenant_id: "100001".to_string(),
            roles: vec!["ai.agents.read".to_string()],
        }
    }

    fn inventory_item(
        engine: &CodeEngineCatalogEngine,
        provider_session_id: String,
        ordinal: usize,
    ) -> ProviderSessionInventoryItem {
        let default_model = engine.models.first().expect("engine default model");
        let timestamp = format!("2026-07-26T00:{:02}:00Z", ordinal % 60);
        let mut session = AgentSession::new(provider_session_id)
            .with_title(format!("{} provider session {ordinal}", engine.engine_key))
            .with_model(default_model.model_id.clone())
            .with_cwd(r"E:\sdkwork-space\sdkwork-birdcoder");
        session.created_at = Some(timestamp.clone());
        session.updated_at = Some(timestamp);
        ProviderSessionInventoryItem {
            engine_key: engine.engine_key.clone(),
            agent_id: engine.agent_id.clone(),
            binding_id: engine.binding_id.clone(),
            provider_id: default_model.provider_id.clone(),
            default_model_id: default_model.model_id.clone(),
            session,
        }
    }

    #[test]
    fn synchronizes_complete_multi_provider_inventory_across_session_pages() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let subject = read_subject();
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let engine = |key: &str| {
            catalog
                .engines
                .iter()
                .find(|engine| engine.engine_key == key)
                .unwrap_or_else(|| panic!("missing {key} engine"))
        };
        let mut inventory = (0..225)
            .map(|index| inventory_item(engine("codex"), format!("codex-{index}"), index))
            .collect::<Vec<_>>();
        inventory.push(inventory_item(
            engine("claude-code"),
            "claude-code-1".to_string(),
            225,
        ));
        inventory.push(inventory_item(
            engine("opencode"),
            "opencode-1".to_string(),
            226,
        ));

        let synchronized = synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            subject.clone(),
            inventory.clone(),
        )
        .expect("complete provider inventory sync");
        assert_eq!(synchronized.synchronized_session_count, 227);
        assert_eq!(
            synchronize_provider_session_inventory(
                state.service.clone(),
                &project,
                subject.clone(),
                inventory,
            )
            .expect("idempotent provider inventory replay")
            .synchronized_session_count,
            227,
        );

        let list_page = |page| {
            state
                .service
                .list_sessions(ListSessionsCommand {
                    query: SessionListQuery::for_tenant(project.tenant_id)
                        .for_organization(project.organization_id)
                        .for_owner(project.owner_user_id)
                        .for_project(project.project_id.clone())
                        .with_pagination(
                            PaginationParams::default()
                                .with_page_size(200)
                                .with_page(page),
                        ),
                    requested_by: subject.clone(),
                })
                .expect("provider session page")
        };
        let first_page = list_page(1);
        let second_page = list_page(2);
        assert_eq!(first_page.items.len(), 200);
        assert_eq!(first_page.total_count, Some(227));
        assert!(first_page.has_more);
        assert_eq!(second_page.items.len(), 27);
        assert_eq!(second_page.total_count, Some(227));
        assert!(!second_page.has_more);

        let activity_query = SessionActivitySummaryListQuery::for_owner(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
        )
        .for_project(project.project_id.clone())
        .with_page_size(200);
        let first_activity_page = state
            .service
            .list_session_activity_summaries(ListSessionActivitySummariesCommand {
                query: activity_query.clone(),
                requested_by: subject.clone(),
            })
            .expect("first synchronized provider Session activity page");
        assert_eq!(first_activity_page.items.len(), 200);
        assert!(first_activity_page.has_more);
        let activity_cursor = crate::session_activity::decode_session_activity_cursor(
            first_activity_page
                .next_page_token
                .as_deref()
                .expect("provider Session activity cursor"),
        )
        .expect("decode provider Session activity cursor");
        let second_activity_page = state
            .service
            .list_session_activity_summaries(ListSessionActivitySummariesCommand {
                query: activity_query.after(activity_cursor),
                requested_by: subject.clone(),
            })
            .expect("second synchronized provider Session activity page");
        assert_eq!(second_activity_page.items.len(), 27);
        assert!(!second_activity_page.has_more);

        let synchronized_session_ids = first_page
            .items
            .iter()
            .chain(second_page.items.iter())
            .map(|session| session.session_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let activity_session_ids = first_activity_page
            .items
            .iter()
            .chain(second_activity_page.items.iter())
            .map(|summary| summary.session.session_id.as_str())
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(activity_session_ids, synchronized_session_ids);

        for engine_key in ["codex", "claude-code", "opencode"] {
            let session = first_page
                .items
                .iter()
                .chain(second_page.items.iter())
                .find(|session| session.agent_id == engine(engine_key).agent_id)
                .unwrap_or_else(|| panic!("missing synchronized {engine_key} session"));
            let bindings = state
                .service
                .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                    query: SessionRuntimeBindingListQuery::for_session(
                        project.tenant_id,
                        project.organization_id,
                        session.session_id.clone(),
                    ),
                    path_agent_id: session.agent_id.clone(),
                    owner_scope: Some(project.owner_user_id),
                    requested_by: subject.clone(),
                })
                .expect("provider Session runtime binding");
            let binding = bindings
                .items
                .first()
                .expect("current provider Session binding");
            assert_eq!(binding.provider_binding_id, engine(engine_key).binding_id);
            assert_eq!(
                binding.provider_id,
                engine(engine_key).models[0].provider_id
            );
            assert!(binding.provider_session_id.is_some());
        }
    }

    #[test]
    fn concurrent_provider_session_inventory_refreshes_are_idempotent() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let inventory = ["codex", "claude-code", "opencode"]
            .into_iter()
            .enumerate()
            .map(|(index, key)| {
                let engine = catalog
                    .engines
                    .iter()
                    .find(|engine| engine.engine_key == key)
                    .unwrap_or_else(|| panic!("missing {key} engine"));
                inventory_item(engine, format!("{key}-concurrent"), index)
            })
            .collect::<Vec<_>>();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let workers = (0..2)
            .map(|_| {
                let service = state.service.clone();
                let project = project.clone();
                let inventory = inventory.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    synchronize_provider_session_inventory(
                        service,
                        &project,
                        read_subject(),
                        inventory,
                    )
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            assert_eq!(
                worker
                    .join()
                    .expect("refresh worker")
                    .expect("refresh")
                    .synchronized_session_count,
                3
            );
        }

        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("concurrent inventory result");
        assert_eq!(sessions.total_count, Some(3));
    }

    #[test]
    fn provider_inventory_deduplicates_normalized_identity_and_skips_subagents() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let root = inventory_item(engine, " provider-session-1 ".to_string(), 1);
        let mut duplicate = root.clone();
        duplicate.session.session_id = "provider-session-1".to_string();
        duplicate.session.title = Some("Duplicate title must not create a row".to_string());
        let mut subagent = inventory_item(engine, "provider-subagent-1".to_string(), 2);
        subagent.session.kind = SessionKind::Subagent;
        subagent.session.parent_session_id = Some("provider-session-1".to_string());

        let synchronization = synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![root, duplicate, subagent],
        )
        .expect("normalized provider inventory sync");
        assert_eq!(synchronization.synchronized_session_count, 1);
        assert_eq!(synchronization.skipped_session_count, 2);
        assert_eq!(synchronization.failed_session_count, 0);

        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id.clone()),
                requested_by: read_subject(),
            })
            .expect("deduplicated provider sessions");
        assert_eq!(sessions.total_count, Some(1));
        let session = sessions.items.first().expect("root provider session");
        let bindings = state
            .service
            .list_session_runtime_bindings(ListSessionRuntimeBindingsCommand {
                query: SessionRuntimeBindingListQuery::for_session(
                    project.tenant_id,
                    project.organization_id,
                    session.session_id.clone(),
                ),
                path_agent_id: session.agent_id.clone(),
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("normalized provider binding");
        let binding = bindings.items.first().expect("provider binding");
        assert_eq!(
            binding.provider_session_id.as_deref(),
            Some("provider-session-1")
        );
        assert_eq!(
            binding.provider_session_tree_id.as_deref(),
            Some("provider-session-1")
        );
    }

    #[test]
    fn provider_inventory_reports_invalid_items_without_aborting_valid_reconciliation() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let valid = inventory_item(engine, "provider-session-valid".to_string(), 1);
        let mut invalid = inventory_item(engine, "provider-session-invalid".to_string(), 2);
        invalid.provider_id = " ".to_string();

        let synchronization = synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![invalid, valid],
        )
        .expect("dirty inventory item must not abort valid reconciliation");

        assert_eq!(synchronization.synchronized_session_count, 1);
        assert_eq!(synchronization.skipped_session_count, 0);
        assert_eq!(synchronization.failed_session_count, 1);
        assert_eq!(
            synchronization.issues,
            vec![ProviderSessionSynchronizationIssue {
                code: "invalid_provider_session_identity",
                count: 1,
                disposition: ProviderSessionSynchronizationIssueDisposition::Failed,
            }]
        );
    }

    #[test]
    fn repeated_provider_session_inventory_sync_updates_the_provider_title() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let mut item = inventory_item(engine, "codex-renamed".to_string(), 1);
        item.session.title = Some("Initial provider title".to_string());
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item.clone()],
        )
        .expect("initial provider inventory sync");

        item.session.title = Some("Renamed provider title".to_string());
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item],
        )
        .expect("renamed provider inventory sync");

        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("renamed provider session");
        assert_eq!(sessions.total_count, Some(1));
        assert_eq!(
            sessions.items[0].title.as_deref(),
            Some("Renamed provider title")
        );
    }

    #[test]
    fn provider_inventory_never_overwrites_a_user_renamed_session_title() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let mut item = inventory_item(engine, "codex-user-title".to_string(), 1);
        item.session.title = Some("Provider title".to_string());
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item.clone()],
        )
        .expect("initial provider inventory sync");

        let session = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id.clone()),
                requested_by: read_subject(),
            })
            .expect("provider session")
            .items
            .into_iter()
            .next()
            .expect("one provider session");
        let renamed = state
            .service
            .update_session(UpdateSessionCommand {
                tenant_id: project.tenant_id,
                organization_id: project.organization_id,
                path_agent_id: session.agent_id.clone(),
                session_id: session.session_id.clone(),
                title: Some("User-owned title".to_string()),
                project_id: None,
                expected_version: Some(session.version),
                owner_scope: Some(project.owner_user_id),
                requested_by: PolicySubject {
                    subject_id: "100".to_string(),
                    tenant_id: "100001".to_string(),
                    roles: vec!["ai.agents.use".to_string()],
                },
                requested_at: "2026-07-27T12:00:00Z".to_string(),
            })
            .expect("user rename");
        assert_eq!(
            renamed.title_source,
            crate::domain::AgentSessionTitleSource::User
        );

        item.session.title = Some("Provider title after user rename".to_string());
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![item],
        )
        .expect("provider inventory refresh");

        let refreshed = state
            .service
            .get_session(crate::application::GetSessionCommand {
                tenant_id: project.tenant_id,
                organization_id: project.organization_id,
                path_agent_id: session.agent_id,
                session_id: session.session_id,
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("refreshed provider session");
        assert_eq!(refreshed.title.as_deref(), Some("User-owned title"));
        assert_eq!(
            refreshed.title_source,
            crate::domain::AgentSessionTitleSource::User
        );
    }

    #[test]
    fn synchronizes_inventory_without_provider_timestamp_from_postgres_project_time() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let mut project = test_project(&state);
        project.created_at = "2026-07-27 03:10:00+00".to_string();
        project.updated_at = "2026-07-27 03:11:00+00".to_string();
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        let mut item = inventory_item(engine, "codex-without-time".to_string(), 0);
        item.session.created_at = None;
        item.session.updated_at = None;

        assert_eq!(
            synchronize_provider_session_inventory(
                state.service.clone(),
                &project,
                read_subject(),
                vec![item],
            )
            .expect("PostgreSQL project time fallback")
            .synchronized_session_count,
            1
        );
        let sessions = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("synchronized Session list");
        assert_eq!(sessions.total_count, Some(1));
        assert_eq!(sessions.items[0].created_at, "2026-07-27T03:11:00+00:00");
    }

    #[test]
    fn provider_session_transcript_items_are_idempotent_and_readable() {
        let state = AgentHttpState::new(
            InMemoryAgentRepository::new(),
            InMemoryAgentAuditSink::default(),
            IamGatedPolicyProvider::default(),
        );
        let project = test_project(&state);
        let catalog = shared_code_engine_host()
            .expect("code engine host")
            .catalog();
        let engine = catalog
            .engines
            .iter()
            .find(|engine| engine.engine_key == "codex")
            .expect("codex engine");
        synchronize_provider_session_inventory(
            state.service.clone(),
            &project,
            read_subject(),
            vec![inventory_item(
                engine,
                "provider-session-transcript-1".to_string(),
                1,
            )],
        )
        .expect("provider inventory sync");
        let session = state
            .service
            .list_sessions(ListSessionsCommand {
                query: SessionListQuery::for_tenant(project.tenant_id)
                    .for_organization(project.organization_id)
                    .for_owner(project.owner_user_id)
                    .for_project(project.project_id),
                requested_by: read_subject(),
            })
            .expect("provider sessions")
            .items
            .into_iter()
            .next()
            .expect("provider session");
        let item_id = format!(
            "item.provider.codex.{}",
            stable_provider_session_item_key("codex", "provider-session-transcript-1", "message-1")
        );
        let command = ReconcileProviderSessionHistoryItemCommand {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            session_id: session.session_id.clone(),
            item_id: item_id.clone(),
            kind: AgentSessionItemKind::UserInput,
            content: Some("provider user message".to_string()),
            content_type: "text/plain".to_string(),
            status: AgentSessionItemStatus::Completed,
            model_id: Some(engine.models[0].model_id.clone()),
            provider_id: Some(engine.models[0].provider_id.clone()),
            tool_name: None,
            tool_call_id: None,
            tool_arguments_json: None,
            tool_result_json: None,
            parent_item_id: None,
            requested_by: read_subject(),
            requested_at: "2026-07-26T00:01:00Z".to_string(),
        };
        state
            .service
            .reconcile_provider_session_history_session_item(command.clone(), "codex")
            .expect("provider transcript item");
        state
            .service
            .reconcile_provider_session_history_session_item(command, "codex")
            .expect("idempotent provider transcript replay");
        let tool_item_id = format!(
            "item.provider.codex.{}",
            stable_provider_session_item_key(
                "codex",
                "provider-session-transcript-1",
                "tool-message-1\u{0}tool-part-1"
            )
        );
        let pending_tool = serde_json::json!({
            "type": "function_call",
            "id": "provider-tool-item-1",
            "call_id": "provider-tool-call-1",
            "name": "shell_command",
            "arguments": "{\"command\":\"cargo test\"}"
        })
        .to_string();
        let tool_command = ReconcileProviderSessionHistoryItemCommand {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            session_id: session.session_id.clone(),
            item_id: tool_item_id.clone(),
            kind: AgentSessionItemKind::ToolCall,
            content: None,
            content_type: "application/json".to_string(),
            status: AgentSessionItemStatus::Pending,
            model_id: Some(engine.models[0].model_id.clone()),
            provider_id: Some(engine.models[0].provider_id.clone()),
            tool_name: Some("shell_command".to_string()),
            tool_call_id: Some("provider-tool-call-1".to_string()),
            tool_arguments_json: Some(pending_tool),
            tool_result_json: None,
            parent_item_id: None,
            requested_by: read_subject(),
            requested_at: "2026-07-26T00:02:00Z".to_string(),
        };
        state
            .service
            .reconcile_provider_session_history_session_item(tool_command.clone(), "codex")
            .expect("pending provider tool item");
        let completed_tool = serde_json::json!({
            "type": "function_call",
            "id": "provider-tool-item-1",
            "call_id": "provider-tool-call-1",
            "name": "shell_command",
            "arguments": "{\"command\":\"cargo test\"}",
            "output": "ok",
            "status": "completed"
        })
        .to_string();
        let completed = state
            .service
            .reconcile_provider_session_history_session_item(
                ReconcileProviderSessionHistoryItemCommand {
                    status: AgentSessionItemStatus::Completed,
                    tool_result_json: Some(completed_tool.clone()),
                    requested_at: "2026-07-26T00:03:00Z".to_string(),
                    ..tool_command
                },
                "codex",
            )
            .expect("completed provider tool item");
        assert_eq!(completed.version, 2);
        assert_eq!(completed.status, AgentSessionItemStatus::Completed);
        assert_eq!(
            completed.tool_result_json.as_deref(),
            Some(completed_tool.as_str())
        );
        let items = state
            .service
            .list_session_items(ListSessionItemsCommand {
                query: SessionItemListQuery::for_session(
                    project.tenant_id,
                    project.organization_id,
                    session.session_id,
                ),
                path_agent_id: session.agent_id,
                owner_scope: Some(project.owner_user_id),
                requested_by: read_subject(),
            })
            .expect("provider transcript items");
        assert_eq!(items.total_count, Some(2));
        let user_item = items
            .items
            .iter()
            .find(|item| item.item_id == item_id)
            .expect("provider user item");
        assert_eq!(user_item.content.as_deref(), Some("provider user message"));
        let tool_item = items
            .items
            .iter()
            .find(|item| item.item_id == tool_item_id)
            .expect("provider tool item");
        assert_eq!(tool_item.status, AgentSessionItemStatus::Completed);
        assert_eq!(tool_item.version, 2);
    }
}
