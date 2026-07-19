use std::sync::{Arc, Mutex};

use sdkwork_agent_kernel::{KernelEvent, KernelResult, PolicySubject};
use sdkwork_intelligence_agents_service::{
    extract_event_context, AgentAuditSink, AgentCompositionSlotKind, AgentCompositionTargetModule,
    AgentProjectDriveAccessMode, AgentProjectStatus, AgentProjectVisibility, AgentsService,
    AuditEventListQuery, CreateProjectCommand, CreateProjectCompositionSlotCommand,
    DeleteProjectCompositionSlotCommand, GetProjectCommand, IamGatedPolicyProvider,
    InMemoryAgentRepository, ListProjectCompositionSlotsCommand, ListProjectsCommand,
    PaginatedResult, ProjectCompositionSlotListQuery, ProjectListQuery, ProjectMutationCommand,
    UpdateProjectCommand, UpdateProjectCompositionSlotCommand,
};

#[derive(Clone, Default)]
struct AuditSink(Arc<Mutex<Vec<KernelEvent>>>);

impl AgentAuditSink for AuditSink {
    fn record(&self, event: KernelEvent) -> KernelResult<()> {
        self.0.lock().unwrap().push(event);
        Ok(())
    }

    fn list_events(
        &self,
        _query: &AuditEventListQuery,
    ) -> KernelResult<PaginatedResult<KernelEvent>> {
        Ok(PaginatedResult::empty())
    }
}

fn subject() -> PolicySubject {
    PolicySubject::new("user.30", "10").with_role("ai.agents.manage")
}

fn service() -> (
    AgentsService<InMemoryAgentRepository, AuditSink, IamGatedPolicyProvider>,
    Arc<Mutex<Vec<KernelEvent>>>,
) {
    let events = Arc::new(Mutex::new(Vec::new()));
    (
        AgentsService::new(
            InMemoryAgentRepository::new(),
            AuditSink(events.clone()),
            IamGatedPolicyProvider::new("policy.project.test"),
        ),
        events,
    )
}

#[test]
fn project_lifecycle_is_scoped_versioned_and_audited() {
    let (service, events) = service();
    let created = service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.alpha".to_string(),
            owner_user_id: 30,
            name: "Alpha".to_string(),
            description: Some("Commercial project".to_string()),
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
            default_agent_id: None,
            default_model_id: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(created.version, 0);

    let updated = service
        .update_project(UpdateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: created.project_id.clone(),
            owner_scope: Some(30),
            expected_version: Some(0),
            name: Some("Alpha Commercial".to_string()),
            description: None,
            visibility: None,
            drive_access_mode: None,
            default_agent_id: None,
            default_model_id: None,
            requested_user_id: 30,
            requested_by: subject(),
            requested_at: "2026-07-19T01:00:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(updated.version, 1);

    let listed = service
        .list_projects(ListProjectsCommand {
            query: ProjectListQuery::for_organization(10, 20).for_owner(30),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(listed.items.len(), 1);

    let archived = service
        .archive_project(ProjectMutationCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: created.project_id.clone(),
            owner_scope: Some(30),
            expected_version: Some(1),
            requested_user_id: 30,
            requested_by: subject(),
            requested_at: "2026-07-19T02:00:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(archived.status, AgentProjectStatus::Archived);

    service
        .delete_project(ProjectMutationCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: created.project_id.clone(),
            owner_scope: Some(30),
            expected_version: Some(2),
            requested_user_id: 30,
            requested_by: subject(),
            requested_at: "2026-07-19T03:00:00Z".to_string(),
        })
        .unwrap();
    assert!(service
        .get_project(GetProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: created.project_id,
            owner_scope: Some(30),
            requested_by: subject(),
        })
        .is_err());

    let events = events.lock().unwrap();
    let actions = events
        .iter()
        .map(|event| extract_event_context(&event.payload, "audit_action").unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        actions,
        [
            "project_created",
            "project_updated",
            "project_archived",
            "project_deleted",
        ]
    );
    assert!(events.iter().all(|event| {
        extract_event_context(&event.payload, "aggregate_type").as_deref() == Some("project")
    }));
}

#[test]
fn shared_project_cannot_expose_owner_private_drive_library() {
    let (service, _) = service();
    let result = service.create_project(CreateProjectCommand {
        tenant_id: 10,
        organization_id: 20,
        project_id: "project.shared".to_string(),
        owner_user_id: 30,
        name: "Shared".to_string(),
        description: None,
        visibility: AgentProjectVisibility::Shared,
        drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
        default_agent_id: None,
        default_model_id: None,
        requested_by: subject(),
        requested_at: "2026-07-19T00:00:00Z".to_string(),
    });
    assert!(result.is_err());
}

#[test]
fn project_composition_slot_lifecycle_enforces_mapping_version_owner_and_audit() {
    let (service, events) = service();
    service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.composition".to_string(),
            owner_user_id: 30,
            name: "Composition".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::ExplicitResources,
            default_agent_id: None,
            default_model_id: None,
            requested_by: subject(),
            requested_at: "2026-07-19T00:00:00Z".to_string(),
        })
        .unwrap();

    let invalid = service.create_project_composition_slot(CreateProjectCompositionSlotCommand {
        tenant_id: 10,
        organization_id: 20,
        project_id: "project.composition".to_string(),
        slot_id: "slot.instructions".to_string(),
        slot_kind: AgentCompositionSlotKind::Prompt,
        target_module: AgentCompositionTargetModule::Memory,
        target_ref: "prompt.instructions".to_string(),
        target_version_ref: None,
        priority: 0,
        enabled: true,
        policy_json: "{}".to_string(),
        owner_scope: Some(30),
        requested_user_id: 30,
        requested_by: subject(),
        requested_at: "2026-07-19T00:01:00Z".to_string(),
    });
    assert!(invalid.is_err());

    let created = service
        .create_project_composition_slot(CreateProjectCompositionSlotCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.composition".to_string(),
            slot_id: "slot.instructions".to_string(),
            slot_kind: AgentCompositionSlotKind::Prompt,
            target_module: AgentCompositionTargetModule::Prompts,
            target_ref: "prompt.instructions".to_string(),
            target_version_ref: Some("version.1".to_string()),
            priority: 0,
            enabled: true,
            policy_json: "{\"mode\":\"system\"}".to_string(),
            owner_scope: Some(30),
            requested_user_id: 30,
            requested_by: subject(),
            requested_at: "2026-07-19T00:02:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(created.version, 0);

    assert!(service
        .update_project_composition_slot(UpdateProjectCompositionSlotCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.composition".to_string(),
            slot_id: created.slot_id.clone(),
            expected_version: None,
            slot_kind: None,
            target_module: None,
            target_ref: None,
            target_version_ref: None,
            priority: Some(10),
            enabled: None,
            policy_json: None,
            owner_scope: Some(30),
            requested_user_id: 30,
            requested_by: subject(),
            requested_at: "2026-07-19T00:03:00Z".to_string(),
        })
        .is_err());

    let updated = service
        .update_project_composition_slot(UpdateProjectCompositionSlotCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.composition".to_string(),
            slot_id: created.slot_id.clone(),
            expected_version: Some(0),
            slot_kind: None,
            target_module: None,
            target_ref: None,
            target_version_ref: None,
            priority: Some(10),
            enabled: Some(false),
            policy_json: None,
            owner_scope: Some(30),
            requested_user_id: 30,
            requested_by: subject(),
            requested_at: "2026-07-19T00:04:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(updated.version, 1);

    let listed = service
        .list_project_composition_slots(ListProjectCompositionSlotsCommand {
            query: ProjectCompositionSlotListQuery::for_project(10, 20, "project.composition"),
            owner_scope: Some(30),
            requested_by: subject(),
        })
        .unwrap();
    assert_eq!(listed.items.len(), 1);

    service
        .delete_project_composition_slot(DeleteProjectCompositionSlotCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.composition".to_string(),
            slot_id: created.slot_id,
            expected_version: Some(1),
            owner_scope: Some(30),
            requested_user_id: 30,
            requested_by: subject(),
            requested_at: "2026-07-19T00:05:00Z".to_string(),
        })
        .unwrap();

    let actions = events
        .lock()
        .unwrap()
        .iter()
        .filter_map(|event| extract_event_context(&event.payload, "audit_action"))
        .collect::<Vec<_>>();
    assert!(actions.contains(&"project_composition_slot_created".to_string()));
    assert!(actions.contains(&"project_composition_slot_updated".to_string()));
    assert!(actions.contains(&"project_composition_slot_deleted".to_string()));
}
