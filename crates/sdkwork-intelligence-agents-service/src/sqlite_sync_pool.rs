//! Blocking facade over SDKWork SQLite pools for synchronous repository ports.

use sdkwork_database_config::{DatabaseConfig, DatabaseEngine};
use sdkwork_database_sqlx::{create_pool_from_config, DatabasePool, PoolError};
use sqlx::SqlitePool;
use std::future::Future;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub const SQLITE_MANAGED_STORE_DATABASE_SERVICE: &str = "AGENTS_STORE";

fn map_sqlx_error(error: sqlx::Error) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::provider_error("sqlite_error", error.to_string())
}

fn map_pool_error(error: PoolError) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::validation(format!("database pool: {error}"))
}

fn map_database_config_error(
    error: sdkwork_database_config::ConfigError,
) -> sdkwork_agent_kernel::KernelError {
    sdkwork_agent_kernel::KernelError::validation(format!("database config: {error}"))
}

#[derive(Debug, Clone)]
pub struct BlockingSqlitePool {
    pool: SqlitePool,
    runtime: ManuallyDrop<Arc<Runtime>>,
    database_pool: DatabasePool,
}

impl BlockingSqlitePool {
    pub fn from_database_pool(
        database_pool: DatabasePool,
        runtime: Arc<Runtime>,
    ) -> Result<Self, PoolError> {
        let pool = database_pool
            .as_sqlite()
            .cloned()
            .ok_or_else(|| PoolError::DatabaseConfig("expected sqlite database pool".to_owned()))?;
        Ok(Self {
            pool,
            runtime: ManuallyDrop::new(runtime),
            database_pool,
        })
    }

    pub fn connect_from_config(
        config: DatabaseConfig,
    ) -> Result<Self, sdkwork_agent_kernel::KernelError> {
        if config.engine != DatabaseEngine::Sqlite {
            return Err(sdkwork_agent_kernel::KernelError::validation(
                "sqlite pool requires DatabaseEngine::Sqlite",
            ));
        }
        let runtime = Arc::new(Runtime::new().map_err(|error| {
            sdkwork_agent_kernel::KernelError::provider_error(
                "sqlite_runtime_error",
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
            sdkwork_agent_kernel::KernelError::validation("unsupported sqlite connection URL")
        })?;
        if engine != DatabaseEngine::Sqlite {
            return Err(sdkwork_agent_kernel::KernelError::validation(
                "expected sqlite connection URL",
            ));
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
        let config = DatabaseConfig::from_env(service_name).map_err(map_database_config_error)?;
        if config.engine != DatabaseEngine::Sqlite {
            return Err(sdkwork_agent_kernel::KernelError::validation(format!(
                "service {service_name} must resolve to SQLite"
            )));
        }
        Self::connect_from_config(config)
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn database_pool(&self) -> &DatabasePool {
        &self.database_pool
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

    pub fn pool_size(&self) -> u32 {
        self.pool.size()
    }

    pub fn pool_idle_connections(&self) -> u32 {
        self.pool.num_idle() as u32
    }
}

impl Drop for BlockingSqlitePool {
    fn drop(&mut self) {
        let runtime = unsafe { ManuallyDrop::take(&mut self.runtime) };
        if Arc::strong_count(&runtime) == 1 && tokio::runtime::Handle::try_current().is_ok() {
            if std::thread::spawn(move || drop(runtime)).join().is_err() {
                tracing::warn!(
                    target: "sdkwork.agents.sqlite_sync_pool",
                    "failed to join SQLite runtime shutdown thread"
                );
            }
            return;
        }
        drop(runtime);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    #[test]
    fn connects_and_executes_sqlite_queries() {
        let pool = BlockingSqlitePool::connect("sqlite::memory:")
            .expect("in-memory SQLite pool should connect");
        let sqlite = pool.pool().clone();
        let value = pool
            .run_kernel(async move {
                sqlx::query("SELECT 1 AS value")
                    .fetch_one(&sqlite)
                    .await
                    .and_then(|row| row.try_get::<i64, _>("value"))
            })
            .expect("SQLite query should execute");

        assert_eq!(value, 1);
        assert!(pool.pool_size() >= 1);
    }

    #[test]
    fn rejects_non_sqlite_urls() {
        let error = BlockingSqlitePool::connect("postgres://localhost/agents")
            .expect_err("PostgreSQL URLs must not enter SQLite adapters");
        assert!(error.safe_message().contains("expected sqlite"));
    }
}
