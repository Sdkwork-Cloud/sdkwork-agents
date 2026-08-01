use sdkwork_intelligence_agents_service::{
    AgentCompositionSlotKind, AgentCompositionTargetModule, AgentProjectCompositionSlotRecord,
    AgentProjectDriveAccessMode, AgentProjectRecord, AgentProjectStatus, AgentProjectVisibility,
    AgentRepository, AgentWorkspaceRecord, AgentWorkspaceStatus, InMemoryAgentRepository,
    PaginationParams, ProjectCompositionSlotListQuery, ProjectListQuery, SQL_COUNT_AGENT_PROJECTS,
    SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS, SQL_INSERT_AGENT_PROJECT,
    SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT, SQL_LIST_AGENT_PROJECTS,
    SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS, SQL_SELECT_AGENT_PROJECT,
    SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME, SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT,
    SQL_UPDATE_AGENT_PROJECT, SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT,
    SQL_UPDATE_AGENT_WORKSPACE,
};

fn project(
    id: u64,
    tenant_id: u64,
    organization_id: u64,
    owner_user_id: u64,
    project_id: &str,
) -> AgentProjectRecord {
    AgentProjectRecord {
        id,
        project_id: project_id.to_string(),
        workspace_id: format!("workspace.default.{owner_user_id}"),
        tenant_id,
        organization_id,
        owner_user_id,
        name: format!("Project {project_id}"),
        description: Some("Commercial chat project".to_string()),
        visibility: AgentProjectVisibility::Private,
        status: AgentProjectStatus::Active,
        drive_access_mode: AgentProjectDriveAccessMode::OwnerLibrary,
        default_agent_id: None,
        default_model_id: None,
        import_source_kind: None,
        import_source_ref: None,
        drive_space_id: None,
        drive_root_entry_id: None,
        drive_logical_path: None,
        created_by: owner_user_id,
        updated_by: owner_user_id,
        version: 0,
        created_at: "2026-07-19T00:00:00Z".to_string(),
        updated_at: "2026-07-19T00:00:00Z".to_string(),
        archived_at: None,
        archived_by: None,
        deleted_at: None,
        deleted_by: None,
        retention_until: None,
    }
}

fn project_slot(id: u64, slot_id: &str, priority: i32) -> AgentProjectCompositionSlotRecord {
    AgentProjectCompositionSlotRecord {
        id,
        tenant_id: 10,
        organization_id: 20,
        project_id: "project.alpha".to_string(),
        slot_id: slot_id.to_string(),
        slot_kind: AgentCompositionSlotKind::Prompt,
        target_module: AgentCompositionTargetModule::Prompts,
        target_ref: format!("prompt.{id}"),
        target_version_ref: Some("version.1".to_string()),
        priority,
        enabled: true,
        policy_json: "{}".to_string(),
        created_by: 30,
        updated_by: 30,
        version: 0,
        created_at: "2026-07-19T00:00:00Z".to_string(),
        updated_at: "2026-07-19T00:00:00Z".to_string(),
        deleted_at: None,
        deleted_by: None,
        retention_until: None,
    }
}

fn workspace(id: u64, workspace_id: &str) -> AgentWorkspaceRecord {
    AgentWorkspaceRecord {
        id,
        workspace_id: workspace_id.to_string(),
        tenant_id: 10,
        organization_id: 20,
        owner_user_id: 30,
        name: "Workspace".to_string(),
        description: None,
        is_default: false,
        status: AgentWorkspaceStatus::Active,
        created_by: 30,
        updated_by: 30,
        version: 0,
        created_at: "2026-07-25T00:00:00Z".to_string(),
        updated_at: "2026-07-25T00:00:00Z".to_string(),
        archived_at: None,
        archived_by: None,
        deleted_at: None,
        deleted_by: None,
        retention_until: None,
    }
}

#[test]
fn workspace_repository_enforces_optimistic_version_and_soft_delete() {
    let repository = InMemoryAgentRepository::new();
    let record = workspace(1, "workspace.alpha");
    repository.insert_workspace(record.clone()).unwrap();

    let mut stale = record.clone();
    stale.name = "Stale".to_string();
    assert!(repository.update_workspace(stale).is_err());

    let mut updated = record;
    updated.name = "Updated".to_string();
    updated.mark_updated(30, "2026-07-25T00:01:00Z");
    repository.update_workspace(updated.clone()).unwrap();
    assert_eq!(
        repository
            .get_workspace(10, 20, "workspace.alpha")
            .unwrap()
            .expect("Workspace")
            .name,
        "Updated"
    );

    updated.soft_delete(30, "2026-07-25T00:02:00Z");
    repository.update_workspace(updated).unwrap();
    assert!(repository
        .get_workspace(10, 20, "workspace.alpha")
        .unwrap()
        .is_none());
}

#[test]
fn project_repository_enforces_scope_filters_and_pagination() {
    let repository = InMemoryAgentRepository::new();
    repository
        .insert_project(project(1, 10, 20, 30, "project.alpha"))
        .unwrap();
    repository
        .insert_project(project(2, 10, 20, 31, "project.beta"))
        .unwrap();
    repository
        .insert_project(project(3, 10, 21, 30, "project.other-org"))
        .unwrap();
    repository
        .insert_project(project(4, 11, 20, 30, "project.other-tenant"))
        .unwrap();

    let query = ProjectListQuery::for_organization(10, 20)
        .for_owner(30)
        .with_search("alpha")
        .with_pagination(PaginationParams::default().with_page_size(1));
    let records = repository.list_projects(&query).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].project_id, "project.alpha");
    assert_eq!(repository.count_projects(&query).unwrap(), 1);
    assert!(repository
        .get_project(10, 21, "project.alpha")
        .unwrap()
        .is_none());
}

#[test]
fn project_repository_enforces_optimistic_version_and_soft_delete() {
    let repository = InMemoryAgentRepository::new();
    let record = project(1, 10, 20, 30, "project.alpha");
    repository.insert_project(record.clone()).unwrap();

    let mut stale = record.clone();
    stale.name = "Stale".to_string();
    assert!(repository.update_project(stale).is_err());

    let mut deleted = record;
    deleted.soft_delete(30, "2026-07-19T01:00:00Z");
    repository.update_project(deleted).unwrap();
    assert!(repository
        .get_project(10, 20, "project.alpha")
        .unwrap()
        .is_none());
    assert_eq!(
        repository
            .count_projects(&ProjectListQuery::for_organization(10, 20))
            .unwrap(),
        0
    );
}

#[test]
fn project_repository_enforces_normalized_workspace_name_uniqueness() {
    let repository = InMemoryAgentRepository::new();
    let mut first = project(1, 10, 20, 30, "project.alpha");
    first.name = "BirdCoder".to_string();
    repository.insert_project(first.clone()).unwrap();

    let mut duplicate = project(2, 10, 20, 30, "project.duplicate");
    duplicate.name = "  birdcoder  ".to_string();
    assert!(repository.insert_project(duplicate).is_err());
    assert_eq!(
        repository
            .get_project_by_workspace_name(10, 20, &first.workspace_id, "BIRDCODER")
            .unwrap()
            .expect("project by workspace name")
            .project_id,
        first.project_id
    );

    let mut other_workspace = project(3, 10, 20, 30, "project.other-workspace");
    other_workspace.workspace_id = "workspace.other".to_string();
    other_workspace.name = "birdcoder".to_string();
    repository.insert_project(other_workspace).unwrap();
}

#[test]
fn project_postgres_sql_is_scoped_parameterized_and_versioned() {
    assert!(SQL_INSERT_AGENT_PROJECT.contains("created_by"));
    for sql in [
        SQL_SELECT_AGENT_PROJECT,
        SQL_LIST_AGENT_PROJECTS,
        SQL_COUNT_AGENT_PROJECTS,
    ] {
        assert!(sql.contains("tenant_id = $1"));
        assert!(sql.contains("organization_id = $2"));
    }
    assert!(SQL_SELECT_AGENT_PROJECT.contains("deleted_at IS NULL"));
    assert!(SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME.contains("LOWER(BTRIM(name))"));
    assert!(SQL_SELECT_AGENT_PROJECT_BY_WORKSPACE_NAME.contains("workspace_id = $3"));
    assert!(SQL_LIST_AGENT_PROJECTS.contains("workspace_id = $4"));
    assert!(SQL_LIST_AGENT_PROJECTS.contains("LIMIT $9 OFFSET $10"));
    assert!(SQL_UPDATE_AGENT_PROJECT.contains("version = $25"));
    assert!(SQL_UPDATE_AGENT_WORKSPACE.contains("version = $15"));
}

#[test]
fn project_composition_slots_are_scoped_sorted_paginated_and_soft_deleted() {
    let repository = InMemoryAgentRepository::new();
    repository
        .insert_project_composition_slot(project_slot(2, "slot.prompt.secondary", 20))
        .unwrap();
    repository
        .insert_project_composition_slot(project_slot(1, "slot.prompt.primary", 10))
        .unwrap();

    let query = ProjectCompositionSlotListQuery::for_project(10, 20, "project.alpha")
        .with_pagination(PaginationParams::default().with_page_size(1));
    let page = repository.list_project_composition_slots(&query).unwrap();
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].slot_id, "slot.prompt.primary");
    assert_eq!(
        repository.count_project_composition_slots(&query).unwrap(),
        2
    );
    assert!(repository
        .get_project_composition_slot(10, 21, "project.alpha", "slot.prompt.primary")
        .unwrap()
        .is_none());

    let mut deleted = page[0].clone();
    deleted.soft_delete(30, "2026-07-19T01:00:00Z");
    repository.update_project_composition_slot(deleted).unwrap();
    assert_eq!(
        repository.count_project_composition_slots(&query).unwrap(),
        1
    );
}

#[test]
fn project_composition_slot_sql_is_scoped_filtered_paginated_and_versioned() {
    assert!(SQL_INSERT_AGENT_PROJECT_COMPOSITION_SLOT.contains("policy_json"));
    for sql in [
        SQL_SELECT_AGENT_PROJECT_COMPOSITION_SLOT,
        SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS,
        SQL_COUNT_AGENT_PROJECT_COMPOSITION_SLOTS,
    ] {
        assert!(sql.contains("tenant_id = $1"));
        assert!(sql.contains("organization_id = $2"));
        assert!(sql.contains("project_id = $3"));
        assert!(sql.contains("deleted_at IS NULL"));
    }
    assert!(SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS.contains("slot_kind = $4"));
    assert!(SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS.contains("enabled = $5"));
    assert!(SQL_LIST_AGENT_PROJECT_COMPOSITION_SLOTS
        .contains("ORDER BY priority ASC, id ASC LIMIT $6 OFFSET $7"));
    assert!(SQL_UPDATE_AGENT_PROJECT_COMPOSITION_SLOT.contains("version = $18"));
}
