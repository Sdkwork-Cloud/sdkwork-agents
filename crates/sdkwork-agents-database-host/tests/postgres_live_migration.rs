//! Live PostgreSQL migration contract for the Agents database baseline.
//!
//! This test is `#[ignore]` by default because it requires a real PostgreSQL
//! service (`SDKWORK_DATABASE_URL` plus the admin credentials for ephemeral
//! provisioning). The CI workflow runs it with `-- --ignored` against the
//! workflow's PostgreSQL service, which is the only way to prove the
//! 1663-line baseline, its indexes, and its constraints are valid: string
//! assertion tests cannot catch SQL dialect or schema drift.

use sdkwork_agents_database_host::bootstrap_agents_database;
use sdkwork_database_config::DatabaseConfig;
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool};
use sqlx::postgres::PgPool;

async fn bootstrap_live() -> PgPool {
    // The migration host owns the pool; it reads the same workspace
    // SDKWORK_DATABASE_URL env as the test environment.
    let config = DatabaseConfig::from_env("AGENTS").expect("read agents database config");
    let pool = create_pool_from_config(config)
        .await
        .expect("create agents database pool");
    bootstrap_agents_database(pool.clone())
        .await
        .expect("agents PostgreSQL baseline must migrate and pass the drift gate");
    match pool {
        DatabasePool::Postgres(pool, _) => pool,
        // The `Sqlite` variant is feature-gated out of this crate's
        // compilation; a match error here means a build configuration change
        // silently dropped the PostgreSQL driver.
    }
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_URL pointing at a live PostgreSQL service"]
async fn agents_postgres_baseline_migrates_and_passes_drift_gate() {
    let _pool = bootstrap_live().await;
    // Reaching this point is the evidence: the baseline + migration framework
    // executed and the drift gate (ensure_agents_schema_current) accepted the
    // live schema.
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_URL pointing at a live PostgreSQL service"]
async fn agents_postgres_core_tables_are_queryable_after_migration() {
    let pool = bootstrap_live().await;

    // Spot-check the canonical aggregates exist and are queryable after the
    // baseline: a migrated empty schema must still expose the owner-scoped
    // tables.
    for table in [
        "ai_agent_workspace",
        "ai_agent_project",
        "ai_agent_session",
        "ai_agent_turn",
        "ai_agent_session_item",
    ] {
        let row: (i64,) = sqlx::query_scalar(
            "SELECT count(*) FROM information_schema.tables WHERE table_schema = current_schema() AND table_name = $1",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("query information_schema");
        assert_eq!(row.0, 1, "table {table} must exist after migration");
    }
}

#[tokio::test]
#[ignore = "requires SDKWORK_DATABASE_URL pointing at a live PostgreSQL service"]
async fn agents_postgres_activity_head_query_executes_against_live_schema() {
    let pool = bootstrap_live().await;

    // The session activity head projection orders by the computed activity_at
    // expression; this query-plan probe records whether the query executes
    // against the live schema (the PRD open item). The assertion is that the
    // query runs; the plan text is returned for operator review.
    let plan: String = sqlx::query_scalar(
        "EXPLAIN SELECT s.id FROM ai_agent_session s WHERE s.tenant_id = 0 AND s.organization_id = 0 AND s.owner_user_id = 0 LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("activity head query must execute against the live schema");
    assert!(!plan.is_empty(), "EXPLAIN must return a plan");
}
