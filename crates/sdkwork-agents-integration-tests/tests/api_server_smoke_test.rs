use axum::body::Body;
use axum::http::{Request, StatusCode};
use sdkwork_agents_contract::env_test_lock;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

fn gateway_test_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("create gateway test runtime"))
}

struct TestEnvVar {
    key: &'static str,
    previous_value: Option<String>,
}

struct GatewayTestEnvironment {
    variables: Vec<TestEnvVar>,
    runtime_dir: PathBuf,
}

impl GatewayTestEnvironment {
    fn new(test_name: &str) -> Self {
        let runtime_dir = std::env::temp_dir().join(format!(
            "sdkwork-agents-{test_name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock should be after Unix epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&runtime_dir).expect("create gateway test runtime dir");

        let mut environment = Self {
            variables: Vec::new(),
            runtime_dir,
        };

        let database_path = environment.runtime_dir.join("agent-server.sqlite");
        environment.set("SDKWORK_DEPLOYMENT_ENV", "development");
        environment.set("ENVIRONMENT", "development");
        environment.set("SDKWORK_AGENTS_ENVIRONMENT", "development");
        environment.set("SDKWORK_AGENTS_CONFIG_PROFILE", "development");
        environment.set("SDKWORK_AGENTS_DEV_AUTH_BYPASS", "true");
        environment.set("SDKWORK_KERNEL_ENVIRONMENT", "development");
        environment.set("SDKWORK_KERNEL_DEPLOYMENT_PROFILE", "standalone");
        environment.set("SDKWORK_KERNEL_RUNTIME_TARGET", "server");
        environment.remove("SDKWORK_KERNEL_PROFILE_ID");
        environment.set("SDKWORK_KERNEL_INGRESS_AUTH_MODE", "open");
        environment.set("SDKWORK_KERNEL_METRICS_AUTH_MODE", "open");
        environment.set("SDKWORK_AGENT_RUNTIME_DATABASE_ENGINE", "sqlite");
        environment.set("SDKWORK_DATABASE_PATH", database_path.to_string_lossy());
        let iam_database_url = format!(
            "sqlite://{}?mode=rwc",
            database_path.to_string_lossy().replace('\\', "/")
        );
        environment.set("SDKWORK_IAM_DATABASE_URL", iam_database_url);
        environment.set("SDKWORK_IAM_DATABASE_ENGINE", "sqlite");
        environment.set("SDKWORK_IAM_DATABASE_MAX_CONNECTIONS", "1");
        environment.set("SDKWORK_IAM_DATABASE_MIN_CONNECTIONS", "0");
        environment.set("SDKWORK_IAM_DATABASE_ACQUIRE_TIMEOUT", "60");
        environment.remove("SDKWORK_AGENT_RUNTIME_DATABASE_URL");
        environment.remove("SDKWORK_AGENT_RUNTIME_POSTGRES_URI");

        environment
    }

    fn set(&mut self, key: &'static str, value: impl AsRef<str>) {
        self.variables.push(TestEnvVar {
            key,
            previous_value: std::env::var(key).ok(),
        });
        std::env::set_var(key, value.as_ref());
    }

    fn remove(&mut self, key: &'static str) {
        self.variables.push(TestEnvVar {
            key,
            previous_value: std::env::var(key).ok(),
        });
        std::env::remove_var(key);
    }
}

impl Drop for GatewayTestEnvironment {
    fn drop(&mut self) {
        for variable in self.variables.iter().rev() {
            match &variable.previous_value {
                Some(value) => std::env::set_var(variable.key, value),
                None => std::env::remove_var(variable.key),
            }
        }
        if let Err(error) = std::fs::remove_dir_all(&self.runtime_dir) {
            eprintln!(
                "failed to remove gateway test runtime dir '{}': {error}",
                self.runtime_dir.display()
            );
        }
    }
}

#[test]
fn api_server_bootstrap_health_and_metrics_contracts() {
    let _guard = env_test_lock();
    let _environment = GatewayTestEnvironment::new("bootstrap-health");
    gateway_test_runtime().block_on(async {
        let app = sdkwork_api_agents_standalone_gateway::build_router()
            .await
            .expect("agents standalone-gateway bootstrap should succeed with dev inline auth");

        for path in ["/healthz", "/readyz", "/livez"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri(path)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "expected {path} to be ready"
            );
        }

        let metrics = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(metrics.status(), StatusCode::OK);
    });
}

#[test]
fn gateway_assembly_composes_kernel_router() {
    let _guard = env_test_lock();
    let _environment = GatewayTestEnvironment::new("gateway-assembly");
    gateway_test_runtime().block_on(async {
        let assembly = sdkwork_api_agents_assembly::assemble_api_router()
            .await
            .expect("gateway assembly should compose kernel routes");

        let healthz = assembly
            .router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(healthz.status(), StatusCode::OK);
    });
}

#[test]
fn app_database_migrate_only_succeeds_with_postgres_baseline_contract() {
    let _guard = env_test_lock();
    let baseline = include_str!("../../../database/ddl/baseline/postgres/0001_agents_baseline.sql");
    assert!(baseline.contains("CREATE TABLE IF NOT EXISTS ai_agent_session"));
    assert!(baseline.contains("CREATE TABLE IF NOT EXISTS ai_agent_task"));
    assert!(!baseline.contains("ai_agent_task_run"));
}
