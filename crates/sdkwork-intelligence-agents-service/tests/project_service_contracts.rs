use std::sync::{Arc, Mutex};

use sdkwork_agent_kernel::{KernelErrorKind, KernelEvent, KernelResult, PolicySubject};
use sdkwork_intelligence_agents_service::{
    extract_event_context, AgentAuditSink, AgentCompositionSlotKind, AgentCompositionTargetModule,
    AgentProjectDriveAccessMode, AgentProjectStatus, AgentProjectVisibility, AgentsService,
    AuditEventListQuery, CreateProjectCommand, CreateProjectCompositionSlotCommand,
    CreateWorkspaceCommand, DeleteProjectCompositionSlotCommand, EnsureDefaultWorkspaceCommand,
    GetProjectCommand, GetWorkspaceCommand, IamGatedPolicyProvider, ImportProjectCommand,
    InMemoryAgentRepository, ListProjectCompositionSlotsCommand, ListProjectsCommand,
    PaginatedResult, ProjectCompositionSlotListQuery, ProjectListQuery, ProjectMutationCommand,
    UpdateProjectCommand, UpdateProjectCompositionSlotCommand, UpdateWorkspaceCommand,
    WorkspaceMutationCommand,
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

type TestService = AgentsService<InMemoryAgentRepository, AuditSink, IamGatedPolicyProvider>;
type RecordedEvents = Arc<Mutex<Vec<KernelEvent>>>;

fn subject() -> PolicySubject {
    PolicySubject::new("user.30", "10").with_role("ai.agents.manage")
}

fn read_subject() -> PolicySubject {
    PolicySubject::new("user.30", "10").with_role("ai.agents.read")
}

fn app_user_subject() -> PolicySubject {
    PolicySubject::new("user.30", "10")
        .with_role("ai.agents.read")
        .with_role("ai.agents.use")
}

fn service() -> (TestService, RecordedEvents) {
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
fn workspace_lifecycle_is_owner_scoped_versioned_and_protects_default_and_projects() {
    let (service, events) = service();
    let default_workspace = service
        .ensure_default_workspace(EnsureDefaultWorkspaceCommand {
            tenant_id: 10,
            organization_id: 20,
            owner_user_id: 30,
            default_name: Some("Default Workspace".to_string()),
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:00:00Z".to_string(),
        })
        .unwrap();
    assert!(default_workspace.is_default);

    let created = service
        .create_workspace(CreateWorkspaceCommand {
            tenant_id: 10,
            organization_id: 20,
            owner_user_id: 30,
            name: "Client Work".to_string(),
            description: Some("Workspace lifecycle".to_string()),
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:01:00Z".to_string(),
        })
        .unwrap();
    assert!(!created.is_default);
    assert!(created.workspace_id.starts_with("workspace."));

    let retrieved = service
        .get_workspace(GetWorkspaceCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: created.workspace_id.clone(),
            owner_user_id: 30,
            requested_by: read_subject(),
        })
        .unwrap();
    assert_eq!(retrieved.name, "Client Work");
    assert!(service
        .get_workspace(GetWorkspaceCommand {
            owner_user_id: 31,
            ..GetWorkspaceCommand {
                tenant_id: 10,
                organization_id: 20,
                workspace_id: created.workspace_id.clone(),
                owner_user_id: 30,
                requested_by: read_subject(),
            }
        })
        .is_err());

    let updated = service
        .update_workspace(UpdateWorkspaceCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: created.workspace_id.clone(),
            owner_user_id: 30,
            expected_version: Some(0),
            name: Some("Client Workspace".to_string()),
            description: None,
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:02:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(updated.version, 1);

    assert!(service
        .archive_workspace(WorkspaceMutationCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: default_workspace.workspace_id,
            owner_user_id: 30,
            expected_version: Some(0),
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:03:00Z".to_string(),
        })
        .is_err());

    service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.workspace.guard".to_string(),
            workspace_id: Some(created.workspace_id.clone()),
            owner_user_id: 30,
            name: "Guarded project".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
            default_agent_id: None,
            default_model_id: None,
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:03:00Z".to_string(),
        })
        .unwrap();
    assert!(service
        .archive_workspace(WorkspaceMutationCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: created.workspace_id.clone(),
            owner_user_id: 30,
            expected_version: Some(1),
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:04:00Z".to_string(),
        })
        .is_err());

    service
        .delete_project(ProjectMutationCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.workspace.guard".to_string(),
            owner_scope: Some(30),
            expected_version: Some(0),
            requested_user_id: 30,
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:05:00Z".to_string(),
        })
        .unwrap();
    let archived = service
        .archive_workspace(WorkspaceMutationCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: created.workspace_id.clone(),
            owner_user_id: 30,
            expected_version: Some(1),
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:06:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(archived.version, 2);
    service
        .delete_workspace(WorkspaceMutationCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: created.workspace_id,
            owner_user_id: 30,
            expected_version: Some(2),
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:07:00Z".to_string(),
        })
        .unwrap();

    let recorded_events = events.lock().unwrap();
    let event_types = recorded_events
        .iter()
        .map(|event| event.event_type.as_str())
        .filter(|event_type| event_type.contains("workspace"))
        .collect::<Vec<_>>();
    assert!(event_types.contains(&"agent.business.workspace.created"));
    assert!(event_types.contains(&"agent.business.workspace.updated"));
    assert!(event_types.contains(&"agent.business.workspace.archived"));
    assert!(event_types.contains(&"agent.business.workspace.deleted"));
}

#[test]
fn read_only_subject_can_list_projects_and_code_engines() {
    let (service, _) = service();
    service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.read-only".to_string(),
            workspace_id: None,
            owner_user_id: 30,
            name: "Read only".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
            default_agent_id: None,
            default_model_id: None,
            requested_by: subject(),
            requested_at: "2026-07-25T00:00:00Z".to_string(),
        })
        .unwrap();

    let projects = service
        .list_projects(ListProjectsCommand {
            query: ProjectListQuery::for_organization(10, 20).for_owner(30),
            requested_by: read_subject(),
        })
        .unwrap();
    assert_eq!(projects.items.len(), 1);

    service
        .list_code_engine_catalog(read_subject())
        .expect("read-only subject should list the code-engine catalog");
}

#[test]
fn app_user_can_create_and_update_owned_project() {
    let (service, _) = service();
    let created = service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.app-user".to_string(),
            workspace_id: None,
            owner_user_id: 30,
            name: "App user project".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
            default_agent_id: None,
            default_model_id: None,
            requested_by: subject(),
            requested_at: "2026-07-25T00:00:00Z".to_string(),
        })
        .expect("app user should create an owned project");

    let updated = service
        .update_project(UpdateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: created.project_id,
            owner_scope: Some(30),
            expected_version: Some(0),
            name: Some("Updated app user project".to_string()),
            description: None,
            visibility: None,
            drive_access_mode: None,
            default_agent_id: None,
            default_model_id: None,
            requested_user_id: 30,
            requested_by: app_user_subject(),
            requested_at: "2026-07-25T00:01:00Z".to_string(),
        })
        .expect("app user should update an owned project");

    assert_eq!(updated.name, "Updated app user project");
}

#[test]
fn drive_import_is_inserted_once_and_reused_by_source_identity() {
    let (service, _) = service();
    service
        .ensure_default_workspace(EnsureDefaultWorkspaceCommand {
            tenant_id: 10,
            organization_id: 20,
            owner_user_id: 30,
            default_name: None,
            requested_by: subject(),
            requested_at: "2026-07-25T00:00:00Z".to_string(),
        })
        .unwrap();
    let command = ImportProjectCommand {
        tenant_id: 10,
        organization_id: 20,
        workspace_id: "workspace.default.30".to_string(),
        project_id: "project.drive-import".to_string(),
        owner_user_id: 30,
        name: "Drive import".to_string(),
        description: None,
        source_kind: "drive_sandbox".to_string(),
        source_ref: "drive://space.alpha/root.alpha".to_string(),
        drive_space_id: "space.alpha".to_string(),
        drive_root_entry_id: "root.alpha".to_string(),
        drive_logical_path: "/sandbox".to_string(),
        requested_by: app_user_subject(),
        requested_at: "2026-07-25T00:00:00Z".to_string(),
    };

    let imported = service.import_project(command.clone()).unwrap();
    let imported_again = service
        .import_project(ImportProjectCommand {
            project_id: "project.drive-import-retry".to_string(),
            name: "Ignored retry name".to_string(),
            ..command
        })
        .unwrap();

    assert_eq!(imported.project_id, imported_again.project_id);
    assert_eq!(imported.version, 0);
    assert_eq!(
        imported.import_source_kind.as_deref(),
        Some("drive_sandbox")
    );
    let projects = service
        .list_projects(ListProjectsCommand {
            query: ProjectListQuery::for_organization(10, 20)
                .for_owner(30)
                .for_workspace("workspace.default.30"),
            requested_by: app_user_subject(),
        })
        .unwrap();
    assert_eq!(projects.items.len(), 1);
}

#[test]
fn project_name_is_unique_within_workspace_across_create_import_and_rename() {
    let (service, _) = service();
    service
        .ensure_default_workspace(EnsureDefaultWorkspaceCommand {
            tenant_id: 10,
            organization_id: 20,
            owner_user_id: 30,
            default_name: None,
            requested_by: subject(),
            requested_at: "2026-07-26T00:00:00Z".to_string(),
        })
        .unwrap();

    let imported = service
        .import_project(ImportProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: "workspace.default.30".to_string(),
            project_id: "project.folder-alpha".to_string(),
            owner_user_id: 30,
            name: "Shared Folder".to_string(),
            description: None,
            source_kind: "drive_sandbox".to_string(),
            source_ref: "drive://space.alpha/root.alpha".to_string(),
            drive_space_id: "space.alpha".to_string(),
            drive_root_entry_id: "root.alpha".to_string(),
            drive_logical_path: "/shared-folder".to_string(),
            requested_by: app_user_subject(),
            requested_at: "2026-07-26T00:01:00Z".to_string(),
        })
        .unwrap();
    let imported_from_another_source = service
        .import_project(ImportProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            workspace_id: "workspace.default.30".to_string(),
            project_id: "project.folder-beta".to_string(),
            owner_user_id: 30,
            name: "  shared folder  ".to_string(),
            description: None,
            source_kind: "drive_sandbox".to_string(),
            source_ref: "drive://space.beta/root.beta".to_string(),
            drive_space_id: "space.beta".to_string(),
            drive_root_entry_id: "root.beta".to_string(),
            drive_logical_path: "/shared-folder".to_string(),
            requested_by: app_user_subject(),
            requested_at: "2026-07-26T00:02:00Z".to_string(),
        })
        .unwrap();
    assert_eq!(imported_from_another_source.project_id, imported.project_id);

    let create_error = service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.duplicate-create".to_string(),
            workspace_id: Some("workspace.default.30".to_string()),
            owner_user_id: 30,
            name: "SHARED FOLDER".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
            default_agent_id: None,
            default_model_id: None,
            requested_by: app_user_subject(),
            requested_at: "2026-07-26T00:03:00Z".to_string(),
        })
        .unwrap_err();
    assert_eq!(create_error.kind(), KernelErrorKind::Conflict);

    let other = service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.other".to_string(),
            workspace_id: Some("workspace.default.30".to_string()),
            owner_user_id: 30,
            name: "Other".to_string(),
            description: None,
            visibility: AgentProjectVisibility::Private,
            drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
            default_agent_id: None,
            default_model_id: None,
            requested_by: app_user_subject(),
            requested_at: "2026-07-26T00:04:00Z".to_string(),
        })
        .unwrap();
    let rename_error = service
        .update_project(UpdateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: other.project_id,
            owner_scope: Some(30),
            expected_version: Some(other.version),
            name: Some("shared folder".to_string()),
            description: None,
            visibility: None,
            drive_access_mode: None,
            default_agent_id: None,
            default_model_id: None,
            requested_user_id: 30,
            requested_by: app_user_subject(),
            requested_at: "2026-07-26T00:05:00Z".to_string(),
        })
        .unwrap_err();
    assert_eq!(rename_error.kind(), KernelErrorKind::Conflict);

    let projects = service
        .list_projects(ListProjectsCommand {
            query: ProjectListQuery::for_organization(10, 20)
                .for_owner(30)
                .for_workspace("workspace.default.30"),
            requested_by: app_user_subject(),
        })
        .unwrap();
    assert_eq!(projects.items.len(), 2);
}

#[test]
fn project_lifecycle_is_scoped_versioned_and_audited() {
    let (service, events) = service();
    let created = service
        .create_project(CreateProjectCommand {
            tenant_id: 10,
            organization_id: 20,
            project_id: "project.alpha".to_string(),
            workspace_id: None,
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
        .filter(|action| action.starts_with("project_"))
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
    assert!(events
        .iter()
        .filter(|event| {
            extract_event_context(&event.payload, "audit_action")
                .is_some_and(|action| action.starts_with("project_"))
        })
        .all(|event| {
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
        workspace_id: None,
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
            workspace_id: None,
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
            slot_id: "slot.documents".to_string(),
            slot_kind: AgentCompositionSlotKind::Document,
            target_module: AgentCompositionTargetModule::Documents,
            target_ref: "document.project.specification".to_string(),
            target_version_ref: Some("document.version.1".to_string()),
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
