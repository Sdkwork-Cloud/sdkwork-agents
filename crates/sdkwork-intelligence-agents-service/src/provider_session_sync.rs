use std::sync::Arc;

use sdkwork_agent_kernel::{
    AgentMessageRole, AgentPartKind, KernelError, KernelResult, PolicySubject,
};
use sdkwork_agents_runtime_facade::{
    AgentsSessionActor, AgentsSessionEntrySurface, AgentsSessionFacade, AgentsSessionKind,
    AgentsSessionRuntimeBindingDescriptor, ProviderSessionInventoryItem,
    ProviderSessionInventorySelector, ResolveAgentsSessionRequest,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::application::{
    CreateSessionItemCommand, GetSessionCommand, ListSessionRuntimeBindingsCommand,
};
use crate::domain::AgentSessionItemKind;
use crate::http::{HttpAgentsSessionFacade, HttpService};
use crate::ports::{PaginationParams, SessionRuntimeBindingListQuery};
use crate::project::AgentProjectRecord;
use crate::runtime_facade_bridge::shared_code_engine_host;

const PROVIDER_SESSION_TITLE_MAX_BYTES: usize = 512;

pub(crate) fn synchronize_project_provider_sessions(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
) -> KernelResult<usize> {
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
) -> KernelResult<usize> {
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
) -> KernelResult<usize> {
    let Some(host) = shared_code_engine_host() else {
        return Ok(0);
    };
    let inventory = host
        .discover_provider_sessions(&ProviderSessionInventorySelector {
            directory_fingerprint,
            exact_cwd,
            unique_basename,
        })
        .map_err(runtime_facade_error)?;
    if inventory.is_empty() {
        return Ok(0);
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
    for message in messages {
        let kind = match message.role {
            AgentMessageRole::User => AgentSessionItemKind::UserInput,
            AgentMessageRole::Agent | AgentMessageRole::Model => {
                AgentSessionItemKind::AssistantOutput
            }
            _ => continue,
        };
        let content = message
            .parts
            .iter()
            .filter(|part| part.kind == AgentPartKind::Text)
            .filter_map(|part| part.text.as_deref())
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        if content.is_empty() {
            continue;
        }
        let item_key = stable_provider_session_item_key(
            engine_key,
            provider_session_id,
            message.message_id.as_str(),
        );
        service.reconcile_provider_session_history_session_item(
            CreateSessionItemCommand {
                tenant_id,
                organization_id,
                session_id: session_id.clone(),
                item_id: format!("item.provider.{engine_key}.{item_key}"),
                kind,
                content,
                content_type: "text/plain".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                model_id: Some(binding.model_id.clone()),
                provider_id: Some(binding.provider_id.clone()),
                parent_item_id: None,
                requested_by: subject.clone(),
                requested_at: message
                    .created_at
                    .unwrap_or_else(|| session.updated_at.clone()),
            },
            engine_key,
        )?;
        synchronized += 1;
    }
    Ok(synchronized)
}

fn synchronize_provider_session_inventory(
    service: Arc<HttpService>,
    project: &AgentProjectRecord,
    subject: PolicySubject,
    inventory: Vec<ProviderSessionInventoryItem>,
) -> KernelResult<usize> {
    let facade = HttpAgentsSessionFacade::for_provider_session_history_reconciliation(service.clone());
    let actor = AgentsSessionActor {
        subject_id: subject.subject_id.clone(),
        roles: subject.roles.clone(),
    };
    let mut synchronized = 0;
    for item in inventory {
        let requested_at = provider_session_requested_at(&item, project)?;
        service.ensure_code_engine_runtime_identity(
            project.tenant_id,
            project.organization_id,
            project.owner_user_id,
            &item.engine_key,
            &item.agent_id,
            &item.binding_id,
            &item.provider_id,
            subject.clone(),
            &requested_at,
        )?;
        let stable_key = stable_provider_session_key(&item.engine_key, &item.session.session_id);
        let session_id = format!("session.provider.{}.{}", item.engine_key, stable_key);
        let runtime_binding_id =
            format!("runtime_binding.provider.{}.{}", item.engine_key, stable_key);
        let title = provider_session_title(item.session.title.as_deref(), &item.engine_key);
        let model_id = item
            .session
            .model
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| item.default_model_id.clone());
        facade
            .resolve_or_create_session(ResolveAgentsSessionRequest {
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
                    provider_binding_id: item.binding_id.clone(),
                    model_id,
                    provider_id: item.provider_id.clone(),
                    provider_session_id: Some(item.session.session_id.clone()),
                    provider_session_tree_id: None,
                    provider_parent_session_id: item.session.parent_session_id.clone(),
                    provider_forked_from_session_id: item.session.forked_from_id.clone(),
                }),
                actor: actor.clone(),
                requested_at,
            })
            .map_err(runtime_facade_error)?;
        synchronized += 1;
    }
    Ok(synchronized)
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

fn stable_provider_session_key(engine_key: &str, provider_session_id: &str) -> String {
    let digest = sdkwork_utils_rust::sha256_hash(
        format!("provider-session-v1\u{0}{engine_key}\u{0}{provider_session_id}").as_bytes(),
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

fn runtime_facade_error(error: impl std::fmt::Display) -> KernelError {
    KernelError::Internal {
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{
        CreateProjectCommand, ListSessionItemsCommand, ListSessionRuntimeBindingsCommand,
        ListSessionsCommand,
    };
    use crate::http::AgentHttpState;
    use crate::infrastructure::{
        IamGatedPolicyProvider, InMemoryAgentAuditSink, InMemoryAgentRepository,
    };
    use crate::ports::{
        PaginationParams, SessionItemListQuery, SessionListQuery, SessionRuntimeBindingListQuery,
    };
    use crate::{AgentProjectDriveAccessMode, AgentProjectVisibility};
    use sdkwork_agent_kernel::AgentSession;
    use sdkwork_agents_runtime_facade::CodeEngineCatalogEngine;

    #[test]
    fn provider_session_ids_are_stable_and_provider_scoped() {
        let first = stable_provider_session_key("codex", "provider-1");
        assert_eq!(first, stable_provider_session_key("codex", "provider-1"));
        assert_ne!(first, stable_provider_session_key("opencode", "provider-1"));
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
        assert_eq!(provider_session_title(Some("   "), "codex"), "codex session");

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
        assert_eq!(synchronized, 227);
        assert_eq!(
            synchronize_provider_session_inventory(
                state.service.clone(),
                &project,
                subject.clone(),
                inventory,
            )
            .expect("idempotent provider inventory replay"),
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
            assert_eq!(worker.join().expect("refresh worker").expect("refresh"), 3);
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
            .expect("PostgreSQL project time fallback"),
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
            vec![inventory_item(engine, "provider-session-transcript-1".to_string(), 1)],
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
        let command = CreateSessionItemCommand {
            tenant_id: project.tenant_id,
            organization_id: project.organization_id,
            session_id: session.session_id.clone(),
            item_id: item_id.clone(),
            kind: AgentSessionItemKind::UserInput,
            content: "provider user message".to_string(),
            content_type: "text/plain".to_string(),
            input_tokens: 0,
            output_tokens: 0,
            model_id: Some(engine.models[0].model_id.clone()),
            provider_id: Some(engine.models[0].provider_id.clone()),
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
        assert_eq!(items.total_count, Some(1));
        assert_eq!(items.items[0].item_id, item_id);
        assert_eq!(
            items.items[0].content.as_deref(),
            Some("provider user message")
        );
    }
}
