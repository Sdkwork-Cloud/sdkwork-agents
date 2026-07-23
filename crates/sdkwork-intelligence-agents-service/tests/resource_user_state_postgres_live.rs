#![cfg(feature = "postgres-sync")]

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sdkwork_agent_kernel::{AgentManifest, PolicySubject};
use sdkwork_intelligence_agents_service::{
    AgentCompositionSlotKind, AgentCompositionTargetModule, AgentImplementationKind,
    AgentItemDriveRefInput, AgentItemFeedbackRating, AgentItemResourceRole,
    AgentProjectDriveAccessMode, AgentProjectVisibility, AgentProviderBindingCommand,
    AgentRepository, AgentSessionEntrySurface, AgentSessionItemKind, AgentSessionKind,
    AgentTurnMode, AgentTurnRecord, AgentTurnStatus, AgentVisibility, AgentsService,
    CancelTurnCommand, CreateAgentCommand, CreateProjectCommand,
    CreateProjectCompositionSlotCommand, CreateSessionCommand, CreateSessionItemCommand,
    CreateSessionRuntimeBindingCommand, CreateTurnCommand, DeleteProjectCompositionSlotCommand,
    GetAgentCommand, IamGatedPolicyProvider, InMemoryAgentAuditSink, ItemFeedbackListQuery,
    ListItemFeedbackCommand, ListProjectCompositionSlotsCommand, ListSessionItemsCommand,
    ListSessionUserStatesCommand, ProjectCompositionSlotListQuery, ResourceUserStateListQuery,
    SessionItemListQuery, SqlAgentRepository, SyncPostgresAdapter, TurnExecutionInput,
    TurnExecutionOutput, TurnExecutor, UpdateItemFeedbackCommand,
    UpdateProjectCompositionSlotCommand, UpdateSessionUserStateCommand,
    RUNTIME_MODE_INFERENCE_ERROR,
};

struct FailingTurnExecutor;

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

struct IsolatedPostgresSchema {
    admin: SyncPostgresAdapter,
    schema: String,
    previous_schema: Option<String>,
}

impl Drop for IsolatedPostgresSchema {
    fn drop(&mut self) {
        let drop_schema = format!("DROP SCHEMA IF EXISTS {} CASCADE", self.schema);
        let pool = self.admin.pool();
        let _ = pool.run(sqlx::query(&drop_schema).execute(pool.pool()));
        match &self.previous_schema {
            Some(value) => std::env::set_var("SDKWORK_AGENTS_DATABASE_SCHEMA", value),
            None => std::env::remove_var("SDKWORK_AGENTS_DATABASE_SCHEMA"),
        }
    }
}

fn bootstrap_isolated_schema(base_url: &str, suffix: u128) -> (IsolatedPostgresSchema, String) {
    let admin =
        SyncPostgresAdapter::connect(base_url).expect("postgres admin adapter should connect");
    let schema = format!("agents_live_{suffix}");
    let create_schema = format!("CREATE SCHEMA {schema}");
    admin
        .pool()
        .run(sqlx::query(&create_schema).execute(admin.pool().pool()))
        .expect("isolated postgres schema should be created");

    let separator = if base_url.contains('?') { '&' } else { '?' };
    let isolated_url = format!("{base_url}{separator}options=-c%20search_path%3D{schema}%2Cpublic");
    let isolated = SyncPostgresAdapter::connect(&isolated_url)
        .expect("isolated postgres adapter should connect");
    let previous_schema = std::env::var("SDKWORK_AGENTS_DATABASE_SCHEMA").ok();
    std::env::set_var("SDKWORK_AGENTS_DATABASE_SCHEMA", &schema);
    let database_pool = isolated.pool().database_pool().clone();
    isolated
        .pool()
        .block_on(sdkwork_agents_database_host::bootstrap_agents_database(
            database_pool,
        ))
        .expect("agents database lifecycle should bootstrap the isolated schema");

    (
        IsolatedPostgresSchema {
            admin,
            schema,
            previous_schema,
        },
        isolated_url,
    )
}

fn subject() -> PolicySubject {
    PolicySubject::new("user.700", "700001").with_role("ai.agents.manage")
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

#[test]
#[ignore = "requires SDKWORK_AGENTS_TEST_POSTGRES_URL with schema create/drop permission"]
fn postgres_resource_user_state_round_trip_and_stale_write_rollback() {
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
            native_session_id: None,
            native_session_tree_id: None,
            native_parent_session_id: None,
            native_forked_from_session_id: None,
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
    lifecycle_repository
        .insert_turn_reservation(AgentTurnRecord {
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
        })
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
    lifecycle_repository
        .insert_turn_reservation(AgentTurnRecord {
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
        })
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
