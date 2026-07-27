#![cfg(feature = "postgres-sync")]

use std::ffi::OsString;
use std::sync::{Arc, Barrier, Condvar, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sdkwork_agent_kernel::{AgentManifest, KernelErrorKind, PolicySubject};
use sdkwork_database_config::claw_database::postgres_url_with_search_path;
use sdkwork_intelligence_agents_service::{
    AgentBusinessIdGenerator, AgentCompositionSlotKind, AgentCompositionTargetModule,
    AgentImplementationKind, AgentItemDriveRefInput, AgentItemFeedbackRating,
    AgentItemResourceRole, AgentProjectDriveAccessMode, AgentProjectVisibility,
    AgentProviderBindingCommand, AgentRepository, AgentSessionEntrySurface, AgentSessionItemKind,
    AgentSessionItemRecord, AgentSessionItemStatus, AgentSessionKind, AgentTurnMode,
    AgentTurnRecord, AgentTurnStatus, AgentVisibility, AgentsService, CancelTurnCommand,
    CreateAgentCommand, CreateProjectCommand, CreateProjectCompositionSlotCommand,
    CreateSessionCommand, CreateSessionItemCommand, CreateSessionRuntimeBindingCommand,
    CreateTurnCommand, DeleteProjectCompositionSlotCommand, GetAgentCommand,
    IamGatedPolicyProvider, InMemoryAgentAuditSink, ItemFeedbackListQuery, ListItemFeedbackCommand,
    ListProjectCompositionSlotsCommand, ListProjectsCommand, ListSessionItemsCommand,
    ListSessionUserStatesCommand, ProjectCompositionSlotListQuery, ProjectListQuery,
    ResourceUserStateListQuery, SessionItemListQuery, SqlAgentAuditSink, SqlAgentRepository,
    SyncPostgresAdapter, TurnExecutionInput, TurnExecutionOutput, TurnExecutor, TurnListQuery,
    UpdateItemFeedbackCommand, UpdateProjectCompositionSlotCommand, UpdateSessionUserStateCommand,
    AUDIT_SINK_NODE_ID, RUNTIME_MODE_INFERENCE_ERROR,
};

struct FailingTurnExecutor;

type LiveAgentsService = AgentsService<
    SqlAgentRepository<SyncPostgresAdapter>,
    InMemoryAgentAuditSink,
    IamGatedPolicyProvider,
>;

#[derive(Debug, Default)]
struct BlockingTurnState {
    started: bool,
    released: bool,
}

#[derive(Debug, Default)]
struct BlockingTurnExecutor {
    state: Mutex<BlockingTurnState>,
    changed: Condvar,
}

impl BlockingTurnExecutor {
    fn wait_until_started(&self) {
        let state = self
            .state
            .lock()
            .expect("blocking executor state should lock");
        let (state, timeout) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(10), |state| !state.started)
            .expect("blocking executor start wait should not be poisoned");
        assert!(state.started, "turn provider did not start before timeout");
        assert!(!timeout.timed_out(), "turn provider start wait timed out");
    }

    fn release(&self) {
        let mut state = self
            .state
            .lock()
            .expect("blocking executor state should lock");
        state.released = true;
        self.changed.notify_all();
    }
}

const AGENTS_DATABASE_SCHEMA_ENV: &str = "SDKWORK_AGENTS_DATABASE_SCHEMA";
const CANONICAL_DATABASE_SCHEMA_ENV: &str = "SDKWORK_DATABASE_SCHEMA";
const CLAW_DATABASE_SCHEMA_ENV: &str = "SDKWORK_CLAW_DATABASE_SCHEMA";
const DATABASE_SCHEMA_FALLBACK_PUBLIC_ENV: &str = "SDKWORK_DATABASE_SCHEMA_FALLBACK_PUBLIC";
const DATABASE_AUTO_MIGRATE_ENV: &str = "SDKWORK_AGENTS_DATABASE_AUTO_MIGRATE";

impl TurnExecutor for FailingTurnExecutor {
    fn complete(&self, _input: &TurnExecutionInput) -> TurnExecutionOutput {
        TurnExecutionOutput {
            content: "provider detail must not be persisted".to_string(),
            model_id: None,
            provider_id: None,
            input_tokens: 0,
            output_tokens: 0,
            runtime_mode: RUNTIME_MODE_INFERENCE_ERROR,
            stream_deltas: Vec::new(),
        }
    }
}

impl TurnExecutor for BlockingTurnExecutor {
    fn complete(&self, input: &TurnExecutionInput) -> TurnExecutionOutput {
        let mut state = self
            .state
            .lock()
            .expect("blocking executor state should lock");
        state.started = true;
        self.changed.notify_all();
        state = self
            .changed
            .wait_while(state, |state| !state.released)
            .expect("blocking executor release wait should not be poisoned");
        drop(state);
        TurnExecutionOutput {
            content: "completion that must lose the cancellation race".to_string(),
            model_id: input.model_id.clone(),
            provider_id: input.provider_id.clone(),
            input_tokens: 3,
            output_tokens: 5,
            runtime_mode: "postgres-live-blocking",
            stream_deltas: Vec::new(),
        }
    }
}

struct IsolatedPostgresSchema {
    admin: SyncPostgresAdapter,
    schema: String,
    _environment: Vec<EnvironmentVariableGuard>,
}

struct EnvironmentVariableGuard {
    key: &'static str,
    previous_value: Option<OsString>,
}

impl EnvironmentVariableGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous_value = std::env::var_os(key);
        std::env::set_var(key, value);
        Self {
            key,
            previous_value,
        }
    }
}

impl Drop for EnvironmentVariableGuard {
    fn drop(&mut self) {
        match &self.previous_value {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

impl Drop for IsolatedPostgresSchema {
    fn drop(&mut self) {
        let drop_schema = format!("DROP SCHEMA IF EXISTS {} CASCADE", self.schema);
        let pool = self.admin.pool();
        if let Err(error) = pool.run(sqlx::query(&drop_schema).execute(pool.pool())) {
            eprintln!(
                "failed to drop isolated Agents schema {}: {error}",
                self.schema
            );
        }
    }
}

fn database_environment_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn create_isolated_schema(
    base_url: &str,
    suffix: u128,
) -> (IsolatedPostgresSchema, String, SyncPostgresAdapter) {
    let admin =
        SyncPostgresAdapter::connect(base_url).expect("postgres admin adapter should connect");
    let schema = format!("agents_live_{suffix}");
    let create_schema = format!("CREATE SCHEMA {schema}");
    admin
        .pool()
        .run(sqlx::query(&create_schema).execute(admin.pool().pool()))
        .expect("isolated postgres schema should be created");

    let environment = vec![
        EnvironmentVariableGuard::set(AGENTS_DATABASE_SCHEMA_ENV, &schema),
        EnvironmentVariableGuard::set(CANONICAL_DATABASE_SCHEMA_ENV, &schema),
        EnvironmentVariableGuard::set(CLAW_DATABASE_SCHEMA_ENV, &schema),
        EnvironmentVariableGuard::set(DATABASE_SCHEMA_FALLBACK_PUBLIC_ENV, "false"),
        EnvironmentVariableGuard::set(DATABASE_AUTO_MIGRATE_ENV, "false"),
    ];
    let isolated_schema = IsolatedPostgresSchema {
        admin,
        schema,
        _environment: environment,
    };
    let isolated_url = postgres_url_with_search_path(base_url, "SDKWORK_AGENTS");
    let isolated = SyncPostgresAdapter::connect(&isolated_url)
        .expect("isolated postgres adapter should connect");

    (isolated_schema, isolated_url, isolated)
}

fn bootstrap_isolated_schema(base_url: &str, suffix: u128) -> (IsolatedPostgresSchema, String) {
    let (isolated_schema, isolated_url, isolated) = create_isolated_schema(base_url, suffix);
    let database_pool = isolated.pool().database_pool().clone();
    isolated
        .pool()
        .block_on(sdkwork_agents_database_host::bootstrap_agents_database(
            database_pool,
        ))
        .expect("agents database lifecycle should bootstrap the isolated schema");
    assert_eq!(
        authoritative_agent_table_count(&isolated, &isolated_schema.schema),
        20,
        "greenfield init must materialize exactly 20 authoritative Agents baseline tables without enabling auto-migrate",
    );

    (isolated_schema, isolated_url)
}

fn authoritative_agent_table_count(adapter: &SyncPostgresAdapter, schema: &str) -> i64 {
    adapter
        .pool()
        .run(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM information_schema.tables \
                 WHERE table_schema = $1 \
                 AND table_type = 'BASE TABLE' \
                 AND table_name LIKE 'ai_agent%'",
            )
            .bind(schema)
            .fetch_one(adapter.pool().pool()),
        )
        .expect("authoritative Agents table count should be queryable")
}

fn subject() -> PolicySubject {
    PolicySubject::new("user.700", "700001").with_role("ai.agents.manage")
}

fn numeric_subject() -> PolicySubject {
    PolicySubject::new("700", "700001").with_role("ai.agents.manage")
}

fn manifest(agent_id: &str) -> AgentManifest {
    AgentManifest {
        schema_version: "1.0.0".to_string(),
        manifest_type: "agent".to_string(),
        agent_id: agent_id.to_string(),
        name: "postgres-user-state".to_string(),
        display_name: "Postgres User State".to_string(),
        description: "live contract fixture".to_string(),
        version: "0.1.0".to_string(),
        domain: "intelligence".to_string(),
        required_capabilities: vec!["model.chat".to_string()],
        optional_capabilities: Vec::new(),
        required_capability_requirements: Vec::new(),
        optional_capability_requirements: Vec::new(),
        event_families: vec!["agent.lifecycle".to_string()],
        owner_name: "sdkwork".to_string(),
        status: "active".to_string(),
    }
}

fn request_item_for_turn(id: u64, turn: &AgentTurnRecord) -> AgentSessionItemRecord {
    AgentSessionItemRecord {
        id,
        item_id: turn.request_item_id.clone(),
        tenant_id: turn.tenant_id,
        organization_id: turn.organization_id,
        session_id: turn.session_id.clone(),
        kind: AgentSessionItemKind::UserInput,
        content: Some("PostgreSQL lifecycle test request".to_string()),
        content_type: "text/plain".to_string(),
        status: AgentSessionItemStatus::Completed,
        sequence: 0,
        input_tokens: 0,
        output_tokens: 0,
        model_id: None,
        provider_id: None,
        tool_name: None,
        tool_call_id: None,
        tool_arguments_json: None,
        tool_result_json: None,
        parent_item_id: None,
        turn_id: Some(turn.turn_id.clone()),
        created_by: turn.owner_user_id,
        version: 0,
        created_at: turn.created_at.clone(),
        updated_at: turn.updated_at.clone(),
        completed_at: Some(turn.created_at.clone()),
        redacted_at: None,
        redacted_by: None,
        retention_until: None,
    }
}

fn create_live_turn_service(
    database_url: &str,
    suffix: u128,
    turn_executor: Option<Arc<dyn TurnExecutor>>,
) -> (Arc<LiveAgentsService>, String, String, String) {
    let service = AgentsService::new(
        SqlAgentRepository::new(
            SyncPostgresAdapter::connect(database_url)
                .expect("live turn service adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-concurrency.postgres-live"),
    );
    let service = match turn_executor {
        Some(turn_executor) => service.with_turn_executor(turn_executor),
        None => service,
    };
    let agent_id = format!("agent.concurrency.{suffix}");
    service
        .create_agent(CreateAgentCommand {
            agent_id: agent_id.clone(),
            tenant_id: 700_001,
            organization_id: 0,
            owner_user_id: 700,
            code: format!("concurrency-{suffix}"),
            display_name: "Postgres Turn Concurrency".to_string(),
            description: None,
            manifest: manifest(&agent_id),
            visibility: AgentVisibility::Private,
            tags: Vec::new(),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:00Z".to_string(),
        })
        .expect("live concurrency agent should be created");
    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 700_001,
            agent_id: agent_id.clone(),
            binding_id: format!("binding.concurrency.{suffix}"),
            provider_id: format!("provider.concurrency.{suffix}"),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: format!("profile.concurrency.{suffix}"),
            capabilities: Vec::new(),
            make_default: true,
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:01Z".to_string(),
        })
        .expect("live concurrency provider binding should be created");
    (
        Arc::new(service),
        agent_id,
        provider_binding.binding_id,
        provider_binding.provider_id,
    )
}

fn create_live_turn_session(
    service: &LiveAgentsService,
    agent_id: &str,
    provider_binding_id: &str,
    provider_id: &str,
    suffix: u128,
    label: &str,
) -> (String, String, String) {
    let session_id = format!("session.{label}.{suffix}");
    let runtime_binding_id = format!("runtime_binding.{label}.{suffix}");
    let model_id = format!("model.{label}.{suffix}");
    service
        .create_session(CreateSessionCommand {
            tenant_id: 700_001,
            organization_id: 0,
            agent_id: agent_id.to_string(),
            owner_user_id: 700,
            session_id: session_id.clone(),
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: None,
            idempotency_key: None,
            payload_hash: None,
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:02Z".to_string(),
        })
        .expect("live concurrency session should be created");
    service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 700_001,
            organization_id: 0,
            path_agent_id: agent_id.to_string(),
            session_id: session_id.clone(),
            runtime_binding_id: Some(runtime_binding_id.clone()),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: provider_binding_id.to_string(),
            model_id: model_id.clone(),
            provider_id: provider_id.to_string(),
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-20T00:00:03Z".to_string(),
        })
        .expect("live concurrency runtime binding should be created");
    (session_id, runtime_binding_id, model_id)
}

struct LiveTurnTarget<'a> {
    agent_id: &'a str,
    session_id: &'a str,
    runtime_binding_id: &'a str,
    model_id: &'a str,
}

fn live_turn_command(
    target: LiveTurnTarget<'_>,
    turn_id: impl Into<String>,
    idempotency_key: impl Into<String>,
    payload_hash: impl Into<String>,
    content: impl Into<String>,
) -> CreateTurnCommand {
    CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: target.agent_id.to_string(),
        session_id: target.session_id.to_string(),
        turn_id: Some(turn_id.into()),
        content: content.into(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(target.runtime_binding_id.to_string()),
        requested_model_id: Some(target.model_id.to_string()),
        idempotency_key: idempotency_key.into(),
        payload_hash: payload_hash.into(),
        client_request_id: None,
        drive_refs: Vec::new(),
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-20T00:00:04Z".to_string(),
        prefer_stream: false,
    }
}

fn read_session_aggregate(
    database_url: &str,
    session_id: &str,
) -> (
    sdkwork_intelligence_agents_service::AgentSessionRecord,
    Vec<AgentSessionItemRecord>,
    Vec<AgentTurnRecord>,
) {
    let repository = SqlAgentRepository::new(
        SyncPostgresAdapter::connect(database_url)
            .expect("live aggregate inspection adapter should connect"),
    );
    let session = repository
        .get_session(700_001, 0, session_id)
        .expect("live session should be queryable")
        .expect("live session should exist");
    let items = repository
        .list_session_items(&SessionItemListQuery::for_session(
            700_001,
            0,
            session_id.to_string(),
        ))
        .expect("live session items should be queryable");
    let turns = repository
        .list_turns(&TurnListQuery::for_session(
            700_001,
            0,
            session_id.to_string(),
        ))
        .expect("live turns should be queryable");
    (session, items, turns)
}

#[test]
#[ignore = "requires SDKWORK_AGENTS_TEST_POSTGRES_URL with schema create/drop permission"]
fn postgres_partial_schema_fails_closed_without_auto_migrate_authorization() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url = std::env::var("SDKWORK_AGENTS_TEST_POSTGRES_URL")
        .expect("SDKWORK_AGENTS_TEST_POSTGRES_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let (isolated_schema, _database_url, isolated) =
        create_isolated_schema(&base_database_url, suffix);
    isolated
        .pool()
        .run(
            sqlx::query("CREATE TABLE ai_agent_outbox_event (id BIGINT PRIMARY KEY)")
                .execute(isolated.pool().pool()),
        )
        .expect("partial schema completion anchor should be created");

    let bootstrap =
        isolated
            .pool()
            .block_on(sdkwork_agents_database_host::bootstrap_agents_database(
                isolated.pool().database_pool().clone(),
            ));
    let error = match bootstrap {
        Ok(_) => panic!("partial schema must not bootstrap when auto-migrate is disabled"),
        Err(error) => error,
    };

    assert!(error.contains("agents database schema is incomplete"));
    assert!(error.contains("missing table: ai_agent"));
    assert_eq!(
        authoritative_agent_table_count(&isolated, &isolated_schema.schema),
        1,
        "init must preserve an anchored partial schema for drift rejection instead of replaying the greenfield baseline",
    );
}

#[test]
#[ignore = "requires SDKWORK_AGENTS_TEST_POSTGRES_URL with schema create/drop permission"]
fn postgres_resource_user_state_round_trip_and_stale_write_rollback() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url = std::env::var("SDKWORK_AGENTS_TEST_POSTGRES_URL")
        .expect("SDKWORK_AGENTS_TEST_POSTGRES_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let (_isolated_schema, database_url) = bootstrap_isolated_schema(&base_database_url, suffix);
    let repository = SqlAgentRepository::new(
        SyncPostgresAdapter::connect(&database_url).expect("postgres adapter should connect"),
    );
    let service = AgentsService::new(
        repository,
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.user-state.postgres-live"),
    );
    let agent_id = format!("agent.live.{suffix}");
    let session_id = format!("session.live.{suffix}");

    service
        .create_agent(CreateAgentCommand {
            agent_id: agent_id.clone(),
            tenant_id: 700_001,
            organization_id: 0,
            owner_user_id: 700,
            code: format!("live-{suffix}"),
            display_name: "Postgres User State".to_string(),
            description: None,
            manifest: manifest(&agent_id),
            visibility: AgentVisibility::Private,
            tags: Vec::new(),
            default_code_task_intent: None,
            implementation_provider_id: None,
            implementation_kind: None,
            implementation_type: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:00Z".to_string(),
        })
        .unwrap();
    let created_agent = service
        .get_agent(GetAgentCommand {
            tenant_id: 700_001,
            agent_id: agent_id.clone(),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(created_agent.agent_id, agent_id);
    let project_id = format!("project.live.{suffix}");
    service
        .create_project(CreateProjectCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            workspace_id: None,
            owner_user_id: 700,
            name: "Postgres composition project".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::ExplicitResources,
            default_agent_id: Some(agent_id.clone()),
            default_model_id: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:10Z".to_string(),
        })
        .unwrap();
    let projects = service
        .list_projects(ListProjectsCommand {
            query: ProjectListQuery::for_organization(700_001, 0).for_owner(700),
            requested_by: subject(),
        })
        .expect("live projects should be listable after project creation");
    assert!(
        projects
            .items
            .iter()
            .any(|project| project.project_id == project_id),
        "the newly created project should be visible in its owner-scoped project list"
    );
    let project_slot = service
        .create_project_composition_slot(CreateProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: "slot.project.instructions".to_string(),
            slot_kind: AgentCompositionSlotKind::Prompt,
            target_module: AgentCompositionTargetModule::Prompts,
            target_ref: format!("prompt.live.{suffix}"),
            target_version_ref: Some("version.1".to_string()),
            priority: -1000,
            enabled: true,
            policy_json: "{\"role\":\"system\"}".to_string(),
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:20Z".to_string(),
        })
        .unwrap();
    let mut slot_query =
        ProjectCompositionSlotListQuery::for_project(700_001, 0, project_id.clone());
    slot_query.slot_kind = Some(AgentCompositionSlotKind::Prompt);
    slot_query.enabled = Some(true);
    let project_slots = service
        .list_project_composition_slots(ListProjectCompositionSlotsCommand {
            query: slot_query,
            owner_scope: Some(700),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(project_slots.items.len(), 1);

    let stale_slot_update =
        service.update_project_composition_slot(UpdateProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: project_slot.slot_id.clone(),
            expected_version: Some(9),
            slot_kind: None,
            target_module: None,
            target_ref: None,
            target_version_ref: None,
            priority: None,
            enabled: Some(false),
            policy_json: None,
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:30Z".to_string(),
        });
    assert!(stale_slot_update.is_err());
    let updated_slot = service
        .update_project_composition_slot(UpdateProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: project_slot.slot_id.clone(),
            expected_version: Some(0),
            slot_kind: None,
            target_module: None,
            target_ref: None,
            target_version_ref: None,
            priority: None,
            enabled: Some(false),
            policy_json: None,
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:40Z".to_string(),
        })
        .unwrap();
    assert_eq!(updated_slot.version, 1);
    service
        .delete_project_composition_slot(DeleteProjectCompositionSlotCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: project_id.clone(),
            slot_id: project_slot.slot_id,
            expected_version: Some(1),
            owner_scope: Some(700),
            requested_user_id: 700,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:50Z".to_string(),
        })
        .unwrap();
    service
        .create_session(CreateSessionCommand {
            tenant_id: 700_001,
            organization_id: 0,
            agent_id: agent_id.clone(),
            owner_user_id: 700,
            session_id: session_id.clone(),
            project_id: None,
            session_kind: AgentSessionKind::Assistant,
            entry_surface: AgentSessionEntrySurface::Api,
            source_module: None,
            source_context_kind: None,
            source_context_id: None,
            parent_session_id: None,
            forked_from_turn_id: None,
            title: None,
            idempotency_key: None,
            payload_hash: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:01:00Z".to_string(),
        })
        .unwrap();

    let provider_binding = service
        .add_provider_binding(AgentProviderBindingCommand {
            tenant_id: 700_001,
            agent_id: agent_id.clone(),
            binding_id: format!("binding.live.{suffix}"),
            provider_id: format!("provider.live.{suffix}"),
            implementation_kind: AgentImplementationKind::ManifestOnly,
            configuration_profile_id: format!("profile.live.{suffix}"),
            capabilities: Vec::new(),
            make_default: true,
            requested_by: subject(),
            requested_at: "2026-07-19T00:01:10Z".to_string(),
        })
        .unwrap();
    let runtime_binding = service
        .create_session_runtime_binding(CreateSessionRuntimeBindingCommand {
            tenant_id: 700_001,
            organization_id: 0,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            runtime_binding_id: Some(format!("runtime_binding.live.{suffix}")),
            runtime_location_id: None,
            host_mode: "managed".to_string(),
            transport_kind: "in_process".to_string(),
            provider_binding_id: provider_binding.binding_id,
            model_id: format!("model.live.{suffix}"),
            provider_id: provider_binding.provider_id,
            provider_session_id: None,
            provider_session_tree_id: None,
            provider_parent_session_id: None,
            provider_forked_from_session_id: None,
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-19T00:01:20Z".to_string(),
        })
        .unwrap();

    let created = service
        .update_session_user_state(UpdateSessionUserStateCommand {
            tenant_id: 700_001,
            organization_id: 0,
            user_id: 700,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            pinned: Some(true),
            hidden: None,
            mark_opened: true,
            last_read_item_sequence: Some(0),
            custom_title: None,
            expected_version: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(created.version, 0);

    let states = service
        .list_session_user_states(ListSessionUserStatesCommand {
            query: ResourceUserStateListQuery::for_user_sessions(700_001, 0, 700).pinned_only(),
            path_agent_id: agent_id.clone(),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(states.items.len(), 1);
    assert_eq!(states.items[0].resource_id, session_id);

    let item_id = format!("item.live.{suffix}");
    service
        .create_session_item(CreateSessionItemCommand {
            tenant_id: 700_001,
            organization_id: 0,
            session_id: session_id.clone(),
            item_id: item_id.clone(),
            kind: AgentSessionItemKind::AssistantOutput,
            content: "Live answer".to_string(),
            content_type: "text/plain".to_string(),
            input_tokens: 0,
            output_tokens: 2,
            model_id: None,
            provider_id: None,
            parent_item_id: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:01Z".to_string(),
        })
        .unwrap();
    let feedback = service
        .update_item_feedback(UpdateItemFeedbackCommand {
            tenant_id: 700_001,
            organization_id: 0,
            user_id: 700,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            item_id: item_id.clone(),
            rating: Some(AgentItemFeedbackRating::Up),
            reason_code: None,
            comment: None,
            expected_version: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:02Z".to_string(),
        })
        .unwrap();
    assert_eq!(feedback.version, 0);
    let feedback_page = service
        .list_item_feedback(ListItemFeedbackCommand {
            query: ItemFeedbackListQuery::for_user_session(700_001, 0, 700, session_id.clone()),
            path_agent_id: agent_id.clone(),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(feedback_page.items.len(), 1);
    assert_eq!(feedback_page.items[0].item_id, item_id);

    let drive_ref = AgentItemDriveRefInput {
        resource_role: AgentItemResourceRole::Image,
        drive_space_id: format!("space-live-{suffix}"),
        drive_node_id: format!("node-live-{suffix}"),
    };
    let turn = service
        .execute_turn(CreateTurnCommand {
            tenant_id: 700_001,
            organization_id: 0,
            agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            turn_id: Some(format!("turn.live.{suffix}")),
            content: "Describe this image".to_string(),
            content_type: "image/png".to_string(),
            turn_mode: AgentTurnMode::Interactive,
            runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
            requested_model_id: Some(runtime_binding.model_id.clone()),
            idempotency_key: format!("live-drive-{suffix}"),
            payload_hash: format!("sha256:live-drive-{suffix}"),
            client_request_id: Some(format!("request-live-drive-{suffix}")),
            drive_refs: vec![drive_ref.clone()],
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:10Z".to_string(),
            prefer_stream: false,
        })
        .unwrap();
    assert_eq!(turn.session.item_count, 3);
    assert_eq!(turn.user_item_drive_refs.len(), 1);
    assert_eq!(
        turn.user_item_drive_refs[0].drive_node_id,
        drive_ref.drive_node_id
    );

    let page = service
        .list_session_items_with_drive_refs(ListSessionItemsCommand {
            query: SessionItemListQuery::for_session(700_001, 0, session_id.clone()),
            path_agent_id: agent_id.clone(),
            owner_scope: Some(700),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(page.items.len(), 3);
    assert_eq!(
        page.items
            .iter()
            .find(|item| item.item.item_id == turn.user_input_item.item_id)
            .unwrap()
            .drive_refs
            .len(),
        1
    );

    let mut invalid = drive_ref.clone();
    invalid.drive_node_id.clear();
    let invalid_result = service.execute_turn(CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        turn_id: None,
        content: "Invalid attachment".to_string(),
        content_type: "image/png".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        requested_model_id: Some(runtime_binding.model_id.clone()),
        idempotency_key: format!("live-drive-invalid-{suffix}"),
        payload_hash: format!("sha256:live-drive-invalid-{suffix}"),
        client_request_id: None,
        drive_refs: vec![invalid],
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-19T00:02:11Z".to_string(),
        prefer_stream: false,
    });
    assert!(invalid_result.is_err());

    let duplicate_result = service.execute_turn(CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        turn_id: None,
        content: "Duplicate attachment".to_string(),
        content_type: "image/png".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        requested_model_id: Some(runtime_binding.model_id.clone()),
        idempotency_key: format!("live-drive-duplicate-{suffix}"),
        payload_hash: format!("sha256:live-drive-duplicate-{suffix}"),
        client_request_id: None,
        drive_refs: vec![drive_ref.clone(), drive_ref],
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-19T00:02:12Z".to_string(),
        prefer_stream: false,
    });
    assert!(duplicate_result.is_err());
    let page_after_rejections = service
        .list_session_items_with_drive_refs(ListSessionItemsCommand {
            query: SessionItemListQuery::for_session(700_001, 0, session_id.clone()),
            path_agent_id: agent_id.clone(),
            owner_scope: Some(700),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(page_after_rejections.items.len(), 3);

    let failed_idempotency_key = format!("live-turn-failed-{suffix}");
    let failure_service = AgentsService::new(
        SqlAgentRepository::new(
            SyncPostgresAdapter::connect(&database_url).expect("failure adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-failure.postgres-live"),
    )
    .with_turn_executor(Arc::new(FailingTurnExecutor));
    let failed_result = failure_service.execute_turn(CreateTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        agent_id: agent_id.clone(),
        session_id: session_id.clone(),
        turn_id: None,
        content: "Persist this provider failure".to_string(),
        content_type: "text/plain".to_string(),
        turn_mode: AgentTurnMode::Interactive,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        requested_model_id: Some(runtime_binding.model_id.clone()),
        idempotency_key: failed_idempotency_key.clone(),
        payload_hash: format!("sha256:live-turn-failed-{suffix}"),
        client_request_id: None,
        drive_refs: Vec::new(),
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-19T00:02:20Z".to_string(),
        prefer_stream: false,
    });
    assert!(failed_result.is_err());
    let lifecycle_repository = SqlAgentRepository::new(
        SyncPostgresAdapter::connect(&database_url).expect("lifecycle adapter should connect"),
    );
    let failed_turn = lifecycle_repository
        .get_turn_by_idempotency(700_001, 0, 700, &failed_idempotency_key)
        .unwrap()
        .unwrap();
    assert_eq!(failed_turn.status, AgentTurnStatus::Failed);
    assert_eq!(failed_turn.version, 2);
    assert_eq!(
        failed_turn.error_code.as_deref(),
        Some("turn_inference_failed")
    );
    assert_eq!(
        failed_turn.error_detail.as_deref(),
        Some("managed turn inference failed")
    );

    let base_id = u64::try_from(suffix).unwrap();
    let cancellable_turn_id = format!("turn.cancel.{suffix}");
    let cancellable_turn = AgentTurnRecord {
        id: base_id.saturating_add(10_000),
        turn_id: cancellable_turn_id.clone(),
        tenant_id: 700_001,
        organization_id: 0,
        session_id: session_id.clone(),
        agent_id: agent_id.clone(),
        owner_user_id: 700,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        client_request_id: None,
        idempotency_key: format!("live-cancel-{suffix}"),
        payload_hash: format!("payload-cancel-{suffix}"),
        request_item_id: format!("item.cancel.{suffix}"),
        response_item_id: None,
        turn_mode: AgentTurnMode::Interactive,
        status: AgentTurnStatus::Requested,
        requested_model_id: None,
        provider_binding_id: None,
        model_id: None,
        provider_id: None,
        input_tokens: 0,
        output_tokens: 0,
        cached_tokens: 0,
        finish_reason: None,
        error_code: None,
        error_detail: None,
        trace_id: None,
        attempt_count: 0,
        max_attempts: 3,
        next_retry_at: None,
        available_at: "2026-07-18T00:00:00Z".to_string(),
        lease_owner: None,
        lease_token: None,
        lease_expires_at: None,
        fencing_token: 0,
        version: 0,
        created_at: "2026-07-18T00:00:00Z".to_string(),
        updated_at: "2026-07-18T00:00:00Z".to_string(),
        started_at: None,
        completed_at: None,
        cancel_requested_at: None,
        cancelled_at: None,
        retention_until: None,
    };
    lifecycle_repository
        .insert_turn_request(
            cancellable_turn.clone(),
            request_item_for_turn(base_id.saturating_add(10_001), &cancellable_turn),
            Vec::new(),
        )
        .unwrap();
    let cancellation_service = AgentsService::new(
        SqlAgentRepository::new(
            SyncPostgresAdapter::connect(&database_url)
                .expect("cancellation adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-cancel.postgres-live"),
    );
    let cancelled = cancellation_service
        .cancel_turn(CancelTurnCommand {
            tenant_id: 700_001,
            organization_id: 0,
            path_agent_id: agent_id.clone(),
            session_id: session_id.clone(),
            turn_id: cancellable_turn_id,
            expected_version: Some(0),
            owner_scope: Some(700),
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:30Z".to_string(),
        })
        .unwrap();
    assert_eq!(cancelled.status, AgentTurnStatus::Cancelled);
    assert_eq!(cancelled.version, 1);
    assert!(cancelled.cancelled_at.is_some());

    let stale_turn_id = format!("turn.stale.{suffix}");
    let stale_turn = AgentTurnRecord {
        id: base_id.saturating_add(20_000),
        turn_id: stale_turn_id.clone(),
        tenant_id: 700_001,
        organization_id: 0,
        session_id: session_id.clone(),
        agent_id: agent_id.clone(),
        owner_user_id: 700,
        runtime_binding_id: Some(runtime_binding.runtime_binding_id.clone()),
        client_request_id: None,
        idempotency_key: format!("live-stale-{suffix}"),
        payload_hash: format!("payload-stale-{suffix}"),
        request_item_id: format!("item.stale.{suffix}"),
        response_item_id: None,
        turn_mode: AgentTurnMode::Interactive,
        status: AgentTurnStatus::Requested,
        requested_model_id: None,
        provider_binding_id: None,
        model_id: None,
        provider_id: None,
        input_tokens: 0,
        output_tokens: 0,
        cached_tokens: 0,
        finish_reason: None,
        error_code: None,
        error_detail: None,
        trace_id: None,
        attempt_count: 0,
        max_attempts: 3,
        next_retry_at: None,
        available_at: "2026-07-18T00:00:00Z".to_string(),
        lease_owner: None,
        lease_token: None,
        lease_expires_at: None,
        fencing_token: 0,
        version: 0,
        created_at: "2026-07-18T00:00:00Z".to_string(),
        updated_at: "2026-07-18T00:00:00Z".to_string(),
        started_at: None,
        completed_at: None,
        cancel_requested_at: None,
        cancelled_at: None,
        retention_until: None,
    };
    lifecycle_repository
        .insert_turn_request(
            stale_turn.clone(),
            request_item_for_turn(base_id.saturating_add(20_001), &stale_turn),
            Vec::new(),
        )
        .unwrap();
    let reconciliation_service = AgentsService::new(
        SqlAgentRepository::new(
            SyncPostgresAdapter::connect(&database_url)
                .expect("reconciliation adapter should connect"),
        ),
        InMemoryAgentAuditSink::default(),
        IamGatedPolicyProvider::new("policy.agents.turn-reconcile.postgres-live"),
    );
    let reconciliation = reconciliation_service
        .reconcile_stale_turns("2026-07-19T00:00:00Z", "2026-07-19T00:02:40Z", 100)
        .unwrap();
    assert_eq!(reconciliation.failed.len(), 1);
    assert_eq!(reconciliation.failed[0].turn_id, stale_turn_id);
    assert_eq!(reconciliation.failed[0].status, AgentTurnStatus::Failed);

    let stale = service.update_session_user_state(UpdateSessionUserStateCommand {
        tenant_id: 700_001,
        organization_id: 0,
        user_id: 700,
        path_agent_id: agent_id,
        session_id,
        pinned: Some(false),
        hidden: None,
        mark_opened: false,
        last_read_item_sequence: None,
        custom_title: None,
        expected_version: Some(9),
        requested_by: subject(),
        requested_at: "2026-07-19T00:03:00Z".to_string(),
    });
    assert!(stale.is_err());
}

#[test]
#[ignore = "requires SDKWORK_AGENTS_TEST_POSTGRES_URL with schema create/drop permission"]
fn postgres_project_create_persists_sql_audit_events() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url = std::env::var("SDKWORK_AGENTS_TEST_POSTGRES_URL")
        .expect("SDKWORK_AGENTS_TEST_POSTGRES_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let (_isolated_schema, database_url) = bootstrap_isolated_schema(&base_database_url, suffix);
    let repository_adapter =
        SyncPostgresAdapter::connect(&database_url).expect("postgres adapter should connect");
    let audit_adapter = SyncPostgresAdapter::with_pool_and_id_generator(
        repository_adapter.pool().clone(),
        AgentBusinessIdGenerator::with_node_id(AUDIT_SINK_NODE_ID)
            .expect("audit sink id generator should initialize"),
    );
    let service = AgentsService::new(
        SqlAgentRepository::new(repository_adapter),
        SqlAgentAuditSink::new_global(audit_adapter),
        IamGatedPolicyProvider::new("policy.agents.project-audit.postgres-live"),
    );

    let project = service
        .create_project(CreateProjectCommand {
            tenant_id: 700_001,
            organization_id: 0,
            project_id: format!("project.audit.{suffix}"),
            workspace_id: None,
            owner_user_id: 700,
            name: "Postgres audited project".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::ExplicitResources,
            default_agent_id: None,
            default_model_id: None,
            requested_by: numeric_subject(),
            requested_at: "2026-07-27T00:00:00Z".to_string(),
        })
        .expect("project creation should persist workspace and project audit events");

    assert_eq!(project.owner_user_id, 700);
}

#[test]
#[ignore = "requires SDKWORK_AGENTS_TEST_POSTGRES_URL with schema create/drop permission"]
fn postgres_turn_idempotency_and_session_sequences_are_concurrency_safe() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url = std::env::var("SDKWORK_AGENTS_TEST_POSTGRES_URL")
        .expect("SDKWORK_AGENTS_TEST_POSTGRES_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_schema, database_url) = bootstrap_isolated_schema(&base_database_url, suffix);
    let (service, agent_id, provider_binding_id, provider_id) =
        create_live_turn_service(&database_url, suffix, None);

    let (idempotent_session_id, idempotent_runtime_id, idempotent_model_id) =
        create_live_turn_session(
            service.as_ref(),
            &agent_id,
            &provider_binding_id,
            &provider_id,
            suffix,
            "idempotent",
        );
    let idempotent_command = live_turn_command(
        LiveTurnTarget {
            agent_id: &agent_id,
            session_id: &idempotent_session_id,
            runtime_binding_id: &idempotent_runtime_id,
            model_id: &idempotent_model_id,
        },
        format!("turn.idempotent.{suffix}"),
        format!("idempotency.concurrent.{suffix}"),
        format!("sha256:idempotency.concurrent.{suffix}"),
        "concurrent idempotent request",
    );
    let start = Arc::new(Barrier::new(3));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let service = Arc::clone(&service);
        let command = idempotent_command.clone();
        let start = Arc::clone(&start);
        workers.push(thread::spawn(move || {
            start.wait();
            service.execute_turn(command)
        }));
    }
    start.wait();
    let outcomes = workers
        .into_iter()
        .map(|worker| worker.join().expect("idempotency worker should not panic"))
        .collect::<Vec<_>>();
    assert!(outcomes.iter().any(Result::is_ok));
    assert!(outcomes.iter().all(|outcome| {
        outcome.is_ok()
            || outcome
                .as_ref()
                .is_err_and(|error| error.kind() == KernelErrorKind::Conflict)
    }));

    let (session, items, turns) = read_session_aggregate(&database_url, &idempotent_session_id);
    assert_eq!(session.item_count, 2);
    assert_eq!(session.last_item_sequence, 2);
    assert_eq!(items.len(), 2);
    assert_eq!(turns.len(), 1);
    assert_eq!(
        items.iter().map(|item| item.sequence).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(turns[0].status, AgentTurnStatus::Completed);

    service
        .execute_turn(idempotent_command.clone())
        .expect("same idempotency key and payload should replay the completed turn");
    let mut conflicting_payload = idempotent_command;
    conflicting_payload.payload_hash = format!("sha256:different.{suffix}");
    let conflict = service
        .execute_turn(conflicting_payload)
        .expect_err("same idempotency key with a different payload must conflict");
    assert_eq!(conflict.kind(), KernelErrorKind::Conflict);
    let (session, items, turns) = read_session_aggregate(&database_url, &idempotent_session_id);
    assert_eq!((session.item_count, session.last_item_sequence), (2, 2));
    assert_eq!((items.len(), turns.len()), (2, 1));

    let (scope_a_session_id, scope_a_runtime_id, scope_a_model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "scope_a",
    );
    let (scope_b_session_id, scope_b_runtime_id, scope_b_model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "scope_b",
    );
    let shared_key = format!("idempotency.cross_session.{suffix}");
    let scope_commands = vec![
        live_turn_command(
            LiveTurnTarget {
                agent_id: &agent_id,
                session_id: &scope_a_session_id,
                runtime_binding_id: &scope_a_runtime_id,
                model_id: &scope_a_model_id,
            },
            format!("turn.scope_a.{suffix}"),
            shared_key.clone(),
            format!("sha256:scope_a.{suffix}"),
            "cross-session contender A",
        ),
        live_turn_command(
            LiveTurnTarget {
                agent_id: &agent_id,
                session_id: &scope_b_session_id,
                runtime_binding_id: &scope_b_runtime_id,
                model_id: &scope_b_model_id,
            },
            format!("turn.scope_b.{suffix}"),
            shared_key,
            format!("sha256:scope_b.{suffix}"),
            "cross-session contender B",
        ),
    ];
    let start = Arc::new(Barrier::new(3));
    let workers = scope_commands
        .into_iter()
        .map(|command| {
            let service = Arc::clone(&service);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                start.wait();
                service.execute_turn(command)
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    let outcomes = workers
        .into_iter()
        .map(|worker| worker.join().expect("scope worker should not panic"))
        .collect::<Vec<_>>();
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                outcome
                    .as_ref()
                    .is_err_and(|error| error.kind() == KernelErrorKind::Conflict)
            })
            .count(),
        1
    );
    let scope_a = read_session_aggregate(&database_url, &scope_a_session_id);
    let scope_b = read_session_aggregate(&database_url, &scope_b_session_id);
    assert_eq!(scope_a.0.item_count + scope_b.0.item_count, 2);
    assert_eq!(
        scope_a.0.last_item_sequence + scope_b.0.last_item_sequence,
        2
    );
    assert_eq!(scope_a.1.len() + scope_b.1.len(), 2);
    assert_eq!(scope_a.2.len() + scope_b.2.len(), 1);
    assert!(
        (scope_a.0.item_count == 2 && scope_b.0.item_count == 0)
            || (scope_a.0.item_count == 0 && scope_b.0.item_count == 2)
    );

    let (sequence_session_id, sequence_runtime_id, sequence_model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "sequence",
    );
    let sequence_commands = (0..2)
        .map(|index| {
            live_turn_command(
                LiveTurnTarget {
                    agent_id: &agent_id,
                    session_id: &sequence_session_id,
                    runtime_binding_id: &sequence_runtime_id,
                    model_id: &sequence_model_id,
                },
                format!("turn.sequence.{index}.{suffix}"),
                format!("idempotency.sequence.{index}.{suffix}"),
                format!("sha256:sequence.{index}.{suffix}"),
                format!("parallel sequence request {index}"),
            )
        })
        .collect::<Vec<_>>();
    let start = Arc::new(Barrier::new(3));
    let workers = sequence_commands
        .into_iter()
        .map(|command| {
            let service = Arc::clone(&service);
            let start = Arc::clone(&start);
            thread::spawn(move || {
                start.wait();
                service.execute_turn(command)
            })
        })
        .collect::<Vec<_>>();
    start.wait();
    for worker in workers {
        worker
            .join()
            .expect("sequence worker should not panic")
            .expect("different idempotency keys should both complete");
    }
    let (session, items, turns) = read_session_aggregate(&database_url, &sequence_session_id);
    assert_eq!((session.item_count, session.last_item_sequence), (4, 4));
    assert_eq!((items.len(), turns.len()), (4, 2));
    assert_eq!(
        items.iter().map(|item| item.sequence).collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

#[test]
#[ignore = "requires SDKWORK_AGENTS_TEST_POSTGRES_URL with schema create/drop permission"]
fn postgres_cancel_wins_completion_race_without_partial_response_state() {
    let _environment_lock = database_environment_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let base_database_url = std::env::var("SDKWORK_AGENTS_TEST_POSTGRES_URL")
        .expect("SDKWORK_AGENTS_TEST_POSTGRES_URL must be set");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the Unix epoch")
        .as_nanos();
    let (_isolated_schema, database_url) = bootstrap_isolated_schema(&base_database_url, suffix);
    let blocking_executor = Arc::new(BlockingTurnExecutor::default());
    let (service, agent_id, provider_binding_id, provider_id) =
        create_live_turn_service(&database_url, suffix, Some(blocking_executor.clone()));
    let (session_id, runtime_binding_id, model_id) = create_live_turn_session(
        service.as_ref(),
        &agent_id,
        &provider_binding_id,
        &provider_id,
        suffix,
        "cancel_race",
    );
    let turn_id = format!("turn.cancel_race.{suffix}");
    let command = live_turn_command(
        LiveTurnTarget {
            agent_id: &agent_id,
            session_id: &session_id,
            runtime_binding_id: &runtime_binding_id,
            model_id: &model_id,
        },
        turn_id.clone(),
        format!("idempotency.cancel_race.{suffix}"),
        format!("sha256:cancel_race.{suffix}"),
        "cancel this turn while its provider is running",
    );
    let execution_service = Arc::clone(&service);
    let execution = thread::spawn(move || execution_service.execute_turn(command));
    blocking_executor.wait_until_started();

    let cancellation = service.cancel_turn(CancelTurnCommand {
        tenant_id: 700_001,
        organization_id: 0,
        path_agent_id: agent_id,
        session_id: session_id.clone(),
        turn_id: turn_id.clone(),
        expected_version: Some(1),
        owner_scope: Some(700),
        requested_by: subject(),
        requested_at: "2026-07-20T00:00:05Z".to_string(),
    });
    blocking_executor.release();
    let completion = execution
        .join()
        .expect("completion worker should not panic")
        .expect_err("completion must lose after cancellation commits");
    assert_eq!(completion.kind(), KernelErrorKind::Conflict);
    let cancelled = cancellation.expect("running turn should be cancelled");
    assert_eq!(cancelled.status, AgentTurnStatus::Cancelled);
    assert_eq!(cancelled.version, 2);

    let (session, items, turns) = read_session_aggregate(&database_url, &session_id);
    assert_eq!((session.item_count, session.last_item_sequence), (1, 1));
    assert_eq!(
        (session.total_input_tokens, session.total_output_tokens),
        (0, 0)
    );
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].kind, AgentSessionItemKind::UserInput);
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].turn_id, turn_id);
    assert_eq!(turns[0].status, AgentTurnStatus::Cancelled);
    assert!(turns[0].response_item_id.is_none());
    assert_eq!((turns[0].input_tokens, turns[0].output_tokens), (0, 0));
}
