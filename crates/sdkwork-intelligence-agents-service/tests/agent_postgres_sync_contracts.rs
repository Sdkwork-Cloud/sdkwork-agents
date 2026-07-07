#![cfg(feature = "postgres-sync")]

use sdkwork_intelligence_agents_service::{
    SQL_COUNT_AGENT_INTERACTIONS, SQL_COUNT_AGENT_MESSAGES, SQL_COUNT_AGENT_SESSIONS,
    SQL_COUNT_AGENT_TASKS, SQL_INSERT_AGENT, SQL_INSERT_AGENT_COMPOSITION_SLOT,
    SQL_INSERT_AGENT_INTERACTION, SQL_INSERT_AGENT_MESSAGE, SQL_INSERT_AGENT_PROVIDER_BINDING,
    SQL_INSERT_AGENT_SESSION, SQL_INSERT_AGENT_TASK, SQL_INSERT_AUDIT_EVENT, SQL_LIST_AGENT,
    SQL_LIST_AGENT_COMPOSITION_SLOTS, SQL_LIST_AGENT_INTERACTIONS, SQL_LIST_AGENT_MESSAGES,
    SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT, SQL_LIST_AGENT_PROVIDER_BINDINGS,
    SQL_LIST_AGENT_SESSIONS, SQL_LIST_AGENT_TASKS, SQL_NEXT_MESSAGE_SEQUENCE,
    SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID, SQL_SELECT_AGENT_COMPOSITION_SLOT,
    SQL_SELECT_AGENT_INTERACTION, SQL_SELECT_AGENT_MESSAGE, SQL_SELECT_AGENT_PROVIDER_BINDING,
    SQL_SELECT_AGENT_SESSION, SQL_SELECT_AGENT_TASK, SQL_UPDATE_AGENT,
    SQL_UPDATE_AGENT_COMPOSITION_SLOT, SQL_UPDATE_AGENT_INTERACTION, SQL_UPDATE_AGENT_MESSAGE,
    SQL_UPDATE_AGENT_PROVIDER_BINDING, SQL_UPDATE_AGENT_SESSION, SQL_UPDATE_AGENT_TASK,
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
    tenant_scoped_select_sql(
        SQL_SELECT_AGENT_COMPOSITION_SLOT,
        "ai_agent_composition_slot",
    );
    tenant_scoped_list_sql(
        SQL_LIST_AGENT_COMPOSITION_SLOTS,
        "ai_agent_composition_slot",
    );
    tenant_scoped_update_sql(
        SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        "ai_agent_composition_slot",
    );
}

// ---------------------------------------------------------------------------
// SQL Injection Security Tests (T-01)
// ---------------------------------------------------------------------------

/// Verifies that all SQL queries use parameterized queries ($1, $2, etc.)
/// and do NOT use string concatenation or format strings for user input.
/// This prevents SQL injection attacks per SECURITY_SPEC requirements.
fn assert_parameterized_query(sql: &str, query_name: &str) {
    // Check that query uses parameterized placeholders
    let has_params = sql.contains("$1") || sql.contains("?");
    assert!(
        has_params,
        "{query_name} must use parameterized queries ($1, $2, etc.) to prevent SQL injection"
    );

    // Check for dangerous string concatenation patterns
    let dangerous_patterns = [
        "format!(",
        "concat(",
        "String::from(",
        ".to_owned() +",
        "\" + ",
        "\" +",
        "+ \"",
        "f\"{",
        "println!(",
        "eprintln!(",
    ];

    for pattern in dangerous_patterns {
        assert!(
            !sql.contains(pattern),
            "{query_name} must not use string concatenation pattern '{pattern}' - use parameterized queries"
        );
    }

    // Verify LIKE clauses use parameterized input, not concatenated strings
    if sql.contains("LIKE") {
        assert!(
            sql.contains("LIKE LOWER($") || sql.contains("LIKE $") || sql.contains("LIKE '%$"),
            "{query_name} LIKE clause must use parameterized input, not string concatenation"
        );
    }
}

/// Verifies that INSERT statements use parameterized values.
fn assert_safe_insert(sql: &str, query_name: &str) {
    assert_parameterized_query(sql, query_name);

    // INSERT should use VALUES ($1, $2, ...) pattern
    assert!(
        sql.contains("VALUES") || sql.contains("values"),
        "{query_name} must be a valid INSERT statement with VALUES clause"
    );
}

/// Verifies that UPDATE statements use parameterized SET clauses.
fn assert_safe_update(sql: &str, query_name: &str) {
    assert_parameterized_query(sql, query_name);

    // UPDATE should have WHERE clause with tenant_id filter
    assert!(
        sql.contains("WHERE"),
        "{query_name} must have WHERE clause for security filtering"
    );
}

/// Verifies that SELECT statements have proper bounds (LIMIT).
fn assert_safe_select(sql: &str, query_name: &str) {
    assert_parameterized_query(sql, query_name);

    // SELECT should have LIMIT for pagination/safety
    assert!(
        sql.contains("LIMIT"),
        "{query_name} must have LIMIT clause to prevent unbounded result sets"
    );
}

#[test]
fn sql_injection_prevention_all_queries() {
    // Verify all SQL constants use parameterized queries

    // Agent queries
    assert_safe_select(
        SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
        "SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID",
    );
    assert_parameterized_query(SQL_LIST_AGENT, "SQL_LIST_AGENT");
    assert_safe_insert(SQL_INSERT_AGENT, "SQL_INSERT_AGENT");
    assert_safe_update(SQL_UPDATE_AGENT, "SQL_UPDATE_AGENT");

    // Provider binding queries
    assert_safe_select(
        SQL_SELECT_AGENT_PROVIDER_BINDING,
        "SQL_SELECT_AGENT_PROVIDER_BINDING",
    );
    assert_parameterized_query(
        SQL_LIST_AGENT_PROVIDER_BINDINGS,
        "SQL_LIST_AGENT_PROVIDER_BINDINGS",
    );
    assert_safe_insert(
        SQL_INSERT_AGENT_PROVIDER_BINDING,
        "SQL_INSERT_AGENT_PROVIDER_BINDING",
    );
    assert_safe_update(
        SQL_UPDATE_AGENT_PROVIDER_BINDING,
        "SQL_UPDATE_AGENT_PROVIDER_BINDING",
    );

    // Composition slot queries
    assert_safe_select(
        SQL_SELECT_AGENT_COMPOSITION_SLOT,
        "SQL_SELECT_AGENT_COMPOSITION_SLOT",
    );
    assert_parameterized_query(
        SQL_LIST_AGENT_COMPOSITION_SLOTS,
        "SQL_LIST_AGENT_COMPOSITION_SLOTS",
    );
    assert_safe_insert(
        SQL_INSERT_AGENT_COMPOSITION_SLOT,
        "SQL_INSERT_AGENT_COMPOSITION_SLOT",
    );
    assert_safe_update(
        SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        "SQL_UPDATE_AGENT_COMPOSITION_SLOT",
    );

    // Audit event queries
    assert_safe_insert(SQL_INSERT_AUDIT_EVENT, "SQL_INSERT_AUDIT_EVENT");

    // Session queries
    assert_safe_select(SQL_SELECT_AGENT_SESSION, "SQL_SELECT_AGENT_SESSION");
    assert_parameterized_query(SQL_LIST_AGENT_SESSIONS, "SQL_LIST_AGENT_SESSIONS");
    assert_safe_insert(SQL_INSERT_AGENT_SESSION, "SQL_INSERT_AGENT_SESSION");
    assert_safe_update(SQL_UPDATE_AGENT_SESSION, "SQL_UPDATE_AGENT_SESSION");

    // Message queries
    assert_safe_select(SQL_SELECT_AGENT_MESSAGE, "SQL_SELECT_AGENT_MESSAGE");
    assert_parameterized_query(SQL_LIST_AGENT_MESSAGES, "SQL_LIST_AGENT_MESSAGES");
    assert_safe_insert(SQL_INSERT_AGENT_MESSAGE, "SQL_INSERT_AGENT_MESSAGE");
    assert_safe_update(SQL_UPDATE_AGENT_MESSAGE, "SQL_UPDATE_AGENT_MESSAGE");
    assert_parameterized_query(SQL_NEXT_MESSAGE_SEQUENCE, "SQL_NEXT_MESSAGE_SEQUENCE");
}

#[test]
fn postgres_session_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_SESSION, "ai_agent_session");
    tenant_scoped_list_sql(SQL_LIST_AGENT_SESSIONS, "ai_agent_session");
    tenant_scoped_update_sql(SQL_UPDATE_AGENT_SESSION, "ai_agent_session");
}

#[test]
fn postgres_message_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_MESSAGE, "ai_agent_message");
    tenant_scoped_list_sql(SQL_LIST_AGENT_MESSAGES, "ai_agent_message");
    assert!(
        SQL_UPDATE_AGENT_MESSAGE.contains("WHERE tenant_id ="),
        "ai_agent_message update SQL must filter by tenant_id"
    );
}

#[test]
fn postgres_session_list_has_mandatory_pagination() {
    assert!(
        SQL_LIST_AGENT_SESSIONS.contains("LIMIT $6"),
        "SQL_LIST_AGENT_SESSIONS must have LIMIT parameter for mandatory pagination"
    );
    assert!(
        SQL_LIST_AGENT_SESSIONS.contains("OFFSET $7"),
        "SQL_LIST_AGENT_SESSIONS must have OFFSET parameter for page navigation"
    );
}

#[test]
fn postgres_message_list_has_mandatory_pagination() {
    assert!(
        SQL_LIST_AGENT_MESSAGES.contains("LIMIT $5"),
        "SQL_LIST_AGENT_MESSAGES must have LIMIT parameter for mandatory pagination"
    );
    assert!(
        SQL_LIST_AGENT_MESSAGES.contains("OFFSET $6"),
        "SQL_LIST_AGENT_MESSAGES must have OFFSET parameter for page navigation"
    );
    assert_parameterized_query(SQL_COUNT_AGENT_MESSAGES, "SQL_COUNT_AGENT_MESSAGES");
    assert!(
        SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT.contains("ORDER BY sequence DESC"),
        "SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT must fetch the most recent context window first"
    );
    assert!(
        SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT.contains("LIMIT $5"),
        "SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT must bound the chat context window"
    );
}

#[test]
fn postgres_session_list_has_mandatory_count_query() {
    assert_parameterized_query(SQL_COUNT_AGENT_SESSIONS, "SQL_COUNT_AGENT_SESSIONS");
}

#[test]
fn sql_list_agent_has_mandatory_pagination() {
    // DATABASE_SPEC §16: All list queries must have mandatory LIMIT and OFFSET
    assert!(
        SQL_LIST_AGENT.contains("LIMIT $7"),
        "SQL_LIST_AGENT must have LIMIT parameter for mandatory pagination (DATABASE_SPEC §16)"
    );
    assert!(
        SQL_LIST_AGENT.contains("OFFSET $8"),
        "SQL_LIST_AGENT must have OFFSET parameter for page navigation"
    );
}

#[test]
fn sql_search_query_is_parameterized() {
    // Verify search query uses parameterized LIKE, not string concatenation
    assert!(
        SQL_LIST_AGENT.contains("LIKE LOWER($5::text)") || SQL_LIST_AGENT.contains("LIKE $5"),
        "SQL_LIST_AGENT search query must use parameterized LIKE clause to prevent SQL injection"
    );

    // Verify search query is wrapped in %...% for substring matching
    // (This wrapping should happen in application code, not SQL)
    assert!(
        !SQL_LIST_AGENT.contains("'%{}%'") && !SQL_LIST_AGENT.contains("'%s%'"),
        "SQL_LIST_AGENT must not have hardcoded search patterns - wrapping should be in application code"
    );
}

#[test]
fn all_queries_enforce_tenant_isolation() {
    // SECURITY_SPEC: All queries must enforce tenant isolation
    let queries = [
        (
            "SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID",
            SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID,
        ),
        ("SQL_LIST_AGENT", SQL_LIST_AGENT),
        ("SQL_UPDATE_AGENT", SQL_UPDATE_AGENT),
        (
            "SQL_SELECT_AGENT_PROVIDER_BINDING",
            SQL_SELECT_AGENT_PROVIDER_BINDING,
        ),
        (
            "SQL_LIST_AGENT_PROVIDER_BINDINGS",
            SQL_LIST_AGENT_PROVIDER_BINDINGS,
        ),
        (
            "SQL_UPDATE_AGENT_PROVIDER_BINDING",
            SQL_UPDATE_AGENT_PROVIDER_BINDING,
        ),
        (
            "SQL_SELECT_AGENT_COMPOSITION_SLOT",
            SQL_SELECT_AGENT_COMPOSITION_SLOT,
        ),
        (
            "SQL_LIST_AGENT_COMPOSITION_SLOTS",
            SQL_LIST_AGENT_COMPOSITION_SLOTS,
        ),
        (
            "SQL_UPDATE_AGENT_COMPOSITION_SLOT",
            SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        ),
        ("SQL_SELECT_AGENT_SESSION", SQL_SELECT_AGENT_SESSION),
        ("SQL_LIST_AGENT_SESSIONS", SQL_LIST_AGENT_SESSIONS),
        ("SQL_UPDATE_AGENT_SESSION", SQL_UPDATE_AGENT_SESSION),
        ("SQL_SELECT_AGENT_MESSAGE", SQL_SELECT_AGENT_MESSAGE),
        ("SQL_LIST_AGENT_MESSAGES", SQL_LIST_AGENT_MESSAGES),
        ("SQL_UPDATE_AGENT_MESSAGE", SQL_UPDATE_AGENT_MESSAGE),
        ("SQL_SELECT_AGENT_INTERACTION", SQL_SELECT_AGENT_INTERACTION),
        ("SQL_LIST_AGENT_INTERACTIONS", SQL_LIST_AGENT_INTERACTIONS),
        ("SQL_UPDATE_AGENT_INTERACTION", SQL_UPDATE_AGENT_INTERACTION),
        ("SQL_SELECT_AGENT_TASK", SQL_SELECT_AGENT_TASK),
        ("SQL_LIST_AGENT_TASKS", SQL_LIST_AGENT_TASKS),
        ("SQL_UPDATE_AGENT_TASK", SQL_UPDATE_AGENT_TASK),
    ];

    for (name, sql) in queries {
        assert!(
            sql.contains("tenant_id = $") || sql.contains("tenant_id ="),
            "{name} must enforce tenant isolation via tenant_id filter"
        );
    }
}

#[test]
fn update_queries_enforce_optimistic_concurrency() {
    // DATABASE_SPEC: All UPDATE queries must enforce optimistic concurrency via version check
    let update_queries = [
        ("SQL_UPDATE_AGENT", SQL_UPDATE_AGENT),
        (
            "SQL_UPDATE_AGENT_PROVIDER_BINDING",
            SQL_UPDATE_AGENT_PROVIDER_BINDING,
        ),
        (
            "SQL_UPDATE_AGENT_COMPOSITION_SLOT",
            SQL_UPDATE_AGENT_COMPOSITION_SLOT,
        ),
        ("SQL_UPDATE_AGENT_INTERACTION", SQL_UPDATE_AGENT_INTERACTION),
        ("SQL_UPDATE_AGENT_TASK", SQL_UPDATE_AGENT_TASK),
    ];

    for (name, sql) in update_queries {
        assert!(
            sql.contains("version ="),
            "{name} must enforce optimistic concurrency via version check"
        );
    }
}

#[test]
fn postgres_interaction_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_INTERACTION, "ai_agent_interaction");
    tenant_scoped_list_sql(SQL_LIST_AGENT_INTERACTIONS, "ai_agent_interaction");
    tenant_scoped_update_sql(SQL_UPDATE_AGENT_INTERACTION, "ai_agent_interaction");
    assert_parameterized_query(SQL_COUNT_AGENT_INTERACTIONS, "SQL_COUNT_AGENT_INTERACTIONS");
    assert!(
        SQL_COUNT_AGENT_INTERACTIONS.contains("tenant_id = $1"),
        "ai_agent_interaction count SQL must filter by tenant_id"
    );
    assert!(
        SQL_INSERT_AGENT_INTERACTION.contains("tenant_id"),
        "ai_agent_interaction insert SQL must include tenant_id"
    );
}

#[test]
fn postgres_task_sql_is_tenant_scoped() {
    tenant_scoped_select_sql(SQL_SELECT_AGENT_TASK, "ai_agent_task");
    tenant_scoped_list_sql(SQL_LIST_AGENT_TASKS, "ai_agent_task");
    tenant_scoped_update_sql(SQL_UPDATE_AGENT_TASK, "ai_agent_task");
    assert!(
        SQL_INSERT_AGENT_TASK.contains("tenant_id"),
        "ai_agent_task insert SQL must include tenant_id"
    );
    assert!(
        SQL_COUNT_AGENT_TASKS.contains("tenant_id = $1"),
        "ai_agent_task count SQL must filter by tenant_id"
    );
    assert!(
        SQL_LIST_AGENT_TASKS.contains("LIMIT"),
        "ai_agent_task list SQL must paginate with LIMIT"
    );
}
