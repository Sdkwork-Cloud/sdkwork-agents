//! Blocking facade over `sdkwork-database-sqlx` PostgreSQL pools for sync repositories.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};
use sdkwork_utils_rust::is_blank;
use sqlx::PgPool;
use std::future::Future;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub type PgRow = sqlx::postgres::PgRow;

pub fn map_sqlx_error(error: sqlx::Error) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::provider_error("postgres_error", error.to_string())
}

pub fn map_pool_error(error: PoolError) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::validation(format!("database pool: {error}"))
}

pub fn map_database_config_error(
    error: sdkwork_database_config::ConfigError,
) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::validation(format!("database config: {error}"))
}

#[derive(Debug, Clone)]
pub struct BlockingPostgresPool {
    pool: PgPool,
    runtime: ManuallyDrop<Arc<Runtime>>,
    database_pool: DatabasePool,
}

impl BlockingPostgresPool {
    pub fn from_database_pool(
        database_pool: DatabasePool,
        runtime: Arc<Runtime>,
    ) -> Result<Self, PoolError> {
        let pool = database_pool.as_postgres().cloned().ok_or_else(|| {
            PoolError::DatabaseConfig("expected postgres database pool".to_owned())
        })?;
        Ok(Self {
            pool,
            runtime: ManuallyDrop::new(runtime),
            database_pool,
        })
    }

    pub fn connect_from_config(
        config: DatabaseConfig,
    ) -> Result<Self, sdkwork_agent_kernel::KernelError> {
        let runtime = Arc::new(Runtime::new().map_err(|error| {
            sdkwork_agent_kernel::KernelError::provider_error(
                "postgres_runtime_error",
                format!("tokio runtime: {error}"),
            )
        })?);
        let database_pool = runtime
            .block_on(create_pool_from_config(config))
            .map_err(map_pool_error)?;
        Self::from_database_pool(database_pool, runtime).map_err(map_pool_error)
    }

    pub fn connect(connection_uri: &str) -> Result<Self, sdkwork_agent_kernel::KernelError> {
        let engine = DatabaseEngine::from_url(connection_uri).ok_or_else(|| {
            sdkwork_agent_kernel::KernelError::validation(format!(
                "unsupported postgres connection url: {connection_uri}"
            ))
        })?;
        if engine != DatabaseEngine::Postgres {
            return Err(sdkwork_agent_kernel::KernelError::validation(format!(
                "expected postgres engine for url: {connection_uri}"
            )));
        }
        Self::connect_from_config(DatabaseConfig {
            engine,
            url: connection_uri.to_owned(),
            ..DatabaseConfig::default()
        })
    }

    pub fn connect_from_sdkwork_env(
        service_name: &str,
    ) -> Result<Self, sdkwork_agent_kernel::KernelError> {
        let legacy_uri_key = format!("SDKWORK_{}_POSTGRES_URI", service_name.to_uppercase());
        if let Ok(uri) = std::env::var(&legacy_uri_key) {
            let trimmed = uri.trim();
            if !is_blank(Some(trimmed)) {
                return Self::connect(trimmed);
            }
        }

        let config = DatabaseConfig::from_env(service_name).map_err(map_database_config_error)?;
        match config.engine {
            DatabaseEngine::Postgres => Self::connect_from_config(config),
            other => Err(sdkwork_agent_kernel::KernelError::validation(format!(
                "service {service_name} resolved database engine {other:?}, expected Postgres"
            ))),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn database_pool(&self) -> &DatabasePool {
        &self.database_pool
    }

    pub fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        self.runtime.block_on(future)
    }

    pub fn run<F, T>(&self, future: F) -> Result<T, sqlx::Error>
    where
        F: Future<Output = Result<T, sqlx::Error>>,
    {
        self.runtime.block_on(future)
    }

    pub fn run_kernel<F, T>(&self, future: F) -> Result<T, sdkwork_agent_kernel::KernelError>
    where
        F: Future<Output = Result<T, sqlx::Error>>,
    {
        self.run(future).map_err(map_sqlx_error)
    }

    pub fn execute_batch_sql(&self, sql: &str) -> Result<(), sdkwork_agent_kernel::KernelError> {
        let pool = self.pool.clone();
        let sql = sql.to_owned();
        self.run_kernel(async move { sqlx::raw_sql(&sql).execute(&pool).await.map(|_| ()) })
    }

    /// Returns the number of connections currently in the pool (both idle and active).
    pub fn pool_size(&self) -> u32 {
        self.pool.size()
    }

    /// Returns the number of idle connections in the pool.
    pub fn pool_idle_connections(&self) -> u32 {
        self.pool.num_idle() as u32
    }

    /// Returns pool utilization as a ratio (0.0 to 1.0).
    /// Calculated as (total - idle) / total when total > 0, else 0.0.
    pub fn pool_utilization(&self) -> f64 {
        let total = self.pool.size();
        if total == 0 {
            return 0.0;
        }
        let idle = self.pool.num_idle() as u32;
        let active = total.saturating_sub(idle);
        active as f64 / total as f64
    }

    /// Returns pool health metrics as a tuple: (total_connections, idle_connections, active_connections, utilization_ratio).
    pub fn pool_metrics(&self) -> PoolMetrics {
        let total = self.pool.size();
        let idle = self.pool.num_idle() as u32;
        let active = total.saturating_sub(idle);
        let utilization = if total > 0 {
            active as f64 / total as f64
        } else {
            0.0
        };
        PoolMetrics {
            total_connections: total,
            idle_connections: idle,
            active_connections: active,
            utilization,
        }
    }
}

impl Drop for BlockingPostgresPool {
    fn drop(&mut self) {
        // `Runtime` must not be dropped from inside another Tokio runtime (for example
        // when standalone gateway tears down embedded agents routes after bind/serve failure).
        let runtime = unsafe { ManuallyDrop::take(&mut self.runtime) };
        if Arc::strong_count(&runtime) == 1 && tokio::runtime::Handle::try_current().is_ok() {
            if std::thread::spawn(move || drop(runtime)).join().is_err() {
                tracing::warn!(
                    target: "sdkwork.agents.postgres_sync_pool",
                    "failed to join runtime shutdown thread"
                );
            }
            return;
        }

        drop(runtime);
    }
}

/// Connection pool health metrics for monitoring and observability.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoolMetrics {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub utilization: f64,
}

#[macro_export]
macro_rules! pg_execute {
    ($pool:expr, $sql:expr $(, $param:expr)* $(,)?) => {{
        let pg = $pool.pool().clone();
        $pool.run_kernel(async {
            sqlx::query::<sqlx::Postgres>($sql) $(.bind(&$param))* .execute(&pg).await.map(|result| result.rows_affected())
        })
    }};
}

#[macro_export]
macro_rules! pg_query {
    ($pool:expr, $sql:expr $(, $param:expr)* $(,)?) => {{
        let pg = $pool.pool().clone();
        $pool.run_kernel(async {
            sqlx::query::<sqlx::Postgres>($sql) $(.bind(&$param))* .fetch_all(&pg).await
        })
    }};
}

#[macro_export]
macro_rules! pg_query_optional {
    ($pool:expr, $sql:expr $(, $param:expr)* $(,)?) => {{
        let pg = $pool.pool().clone();
        $pool.run_kernel(async {
            sqlx::query::<sqlx::Postgres>($sql) $(.bind(&$param))* .fetch_optional(&pg).await
        })
    }};
}
