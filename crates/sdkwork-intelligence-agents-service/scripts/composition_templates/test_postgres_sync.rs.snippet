#![cfg(feature = "postgres-sync")]

use sdkwork_intelligence_agents_service::{
    SQL_LIST_AGENT_COMPOSITION_SLOTS, SQL_SELECT_AGENT_COMPOSITION_SLOT,
    SQL_UPDATE_AGENT_COMPOSITION_SLOT,
};

fn tenant_scoped_select_sql(sql: &str, table: &str) {
    assert!(
        sql.contains("WHERE tenant_id = $1"),
        "{table} select SQL must filter by tenant_id"
    );
    assert!(
        sql.contains("LIMIT 1"),
        "{table} get-by-id select SQL should be bounded"
    );
}

fn tenant_scoped_list_sql(sql: &str, table: &str) {
    assert!(
        sql.contains("WHERE tenant_id = $1"),
        "{table} list SQL must filter by tenant_id"
    );
}

fn tenant_scoped_update_sql(sql: &str, table: &str) {
    assert!(
        sql.contains("WHERE tenant_id ="),
        "{table} update SQL must filter by tenant_id"
    );
    assert!(
        sql.contains("version ="),
        "{table} update SQL must enforce optimistic concurrency"
    );
}

#[test]
fn postgres_composition_slot_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_COMPOSITION_SLOT, "ai_agent_composition_slot");
    tenant_scoped_list_sql(SQL_LIST_AGENT_COMPOSITION_SLOTS, "ai_agent_composition_slot");
    tenant_scoped_update_sql(SQL_UPDATE_AGENT_COMPOSITION_SLOT, "ai_agent_composition_slot");
}
