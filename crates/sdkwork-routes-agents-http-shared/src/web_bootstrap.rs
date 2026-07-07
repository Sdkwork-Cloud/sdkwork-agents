use crate::generated::{APP_ROUTES, BACKEND_ROUTES, COMBINED_ROUTES, OPEN_ROUTES};
use std::sync::Arc;

use axum::http::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::Router;
use sdkwork_iam_web_adapter::IamWebRequestContextResolver;
use sdkwork_intelligence_agents_service::AgentRequestContext;
use sdkwork_web_axum::{with_web_request_context, WebFrameworkLayer};
use sdkwork_web_core::{
    DomainContextInjector, HttpRouteManifest, SecurityPolicy, WebEnvironment, WebRequestContext,
    WebRequestContextProfile,
};

fn parse_agents_web_environment(value: Option<String>) -> WebEnvironment {
    match value
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "dev" | "development" => WebEnvironment::Dev,
        "test" | "testing" => WebEnvironment::Test,
        _ => WebEnvironment::Prod,
    }
}

fn first_nonempty_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        std::env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn resolve_agents_web_environment_from_process_env() -> WebEnvironment {
    parse_agents_web_environment(first_nonempty_env(&[
        "SDKWORK_AGENTS_ENVIRONMENT",
        "SDKWORK_IM_ENVIRONMENT",
        "SDKWORK_ENV",
        "ENVIRONMENT",
    ]))
}

/// Agents HTTP security policy aligned with IM embedded dependency bootstrap behavior.
fn agents_service_security_policy(environment: &WebEnvironment) -> SecurityPolicy {
    let mut security_policy = if matches!(environment, WebEnvironment::Dev | WebEnvironment::Test) {
        SecurityPolicy::default()
    } else {
        SecurityPolicy::production()
    };
    if matches!(environment, WebEnvironment::Dev | WebEnvironment::Test) {
        security_policy.cors.allow_all_origins = true;
        security_policy
            .cross_site
            .reject_untrusted_state_changing_origins = false;
        security_policy.cross_site.reject_cookie_auth_without_origin = false;
    }
    security_policy
}

pub fn app_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(APP_ROUTES)
}

pub fn backend_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(BACKEND_ROUTES)
}

pub fn open_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(OPEN_ROUTES)
}

pub fn combined_route_manifest() -> HttpRouteManifest {
    HttpRouteManifest::new(COMBINED_ROUTES)
}

fn agent_web_request_context_profile() -> WebRequestContextProfile {
    WebRequestContextProfile {
        open_api_prefixes: vec![
            "/agent/v3/api".to_owned(),
            sdkwork_web_core::OPEN_API_PREFIX.to_owned(),
        ],
        ..WebRequestContextProfile::default()
    }
}

#[derive(Clone, Default)]
struct AgentRequestContextInjector;

impl DomainContextInjector for AgentRequestContextInjector {
    fn inject(&self, request: &mut axum::extract::Request, context: &WebRequestContext) {
        if let Some(agent_context) = agent_request_context_from_web_request(context) {
            request.extensions_mut().insert(agent_context);
        }
    }
}

fn agent_request_context_from_web_request(
    context: &WebRequestContext,
) -> Option<AgentRequestContext> {
    let principal = context.principal.as_ref()?;
    let tenant_id = principal.tenant_id().to_string();
    let subject_id = principal.user_id().to_string();
    let organization_id = principal.organization_id().map(str::to_owned);
    let roles = principal.scopes.permission_scope.clone();
    let mut agent_context = AgentRequestContext::new(tenant_id, subject_id.clone())
        .with_subject_id(subject_id)
        .with_roles(roles);
    if let Some(organization_id) = organization_id {
        agent_context = agent_context.with_organization_id(organization_id);
    }
    Some(agent_context)
}

pub fn wrap_router_with_web_framework(
    resolver: IamWebRequestContextResolver,
    route_manifest: HttpRouteManifest,
    router: Router,
) -> Router {
    let environment = resolve_agents_web_environment_from_process_env();
    let security_policy = agents_service_security_policy(&environment);
    let layer = WebFrameworkLayer::new(resolver)
        .with_profile(agent_web_request_context_profile())
        .with_security_policy(security_policy)
        .with_route_manifest(route_manifest)
        .with_domain_injector(Arc::new(AgentRequestContextInjector));
    with_web_request_context(router, layer)
}

pub async fn wrap_router_with_web_framework_from_env(
    route_manifest: HttpRouteManifest,
    router: Router,
) -> Router {
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    wrap_router_with_web_framework(resolver, route_manifest, router)
}

async fn record_agents_request_metrics(request: Request<axum::body::Body>, next: Next) -> Response {
    let response = next.run(request).await;
    sdkwork_intelligence_agents_service::AgentMetricsRegistry::global()
        .record_http_request(response.status().as_u16());
    response
}

/// Builds a combined agents managed store router with mandatory sdkwork-web-framework middleware.
pub async fn build_served_combined_router(
    state: sdkwork_intelligence_agents_service::AgentHttpState,
) -> Router {
    let router = sdkwork_intelligence_agents_service::build_combined_routes().with_state(state);
    let router = router.layer(middleware::from_fn(record_agents_request_metrics));
    wrap_router_with_web_framework_from_env(combined_route_manifest(), router).await
}

#[cfg(test)]
mod tests {
    use super::{
        agents_service_security_policy, parse_agents_web_environment,
        resolve_agents_web_environment_from_process_env,
    };
    use sdkwork_web_core::WebEnvironment;

    #[test]
    fn dev_security_policy_allows_browser_origins() {
        let policy = agents_service_security_policy(&WebEnvironment::Dev);
        assert!(policy.cors.allow_all_origins);
        assert!(!policy.cross_site.reject_untrusted_state_changing_origins);
    }

    #[test]
    fn production_security_policy_rejects_permissive_cors() {
        let policy = agents_service_security_policy(&WebEnvironment::Prod);
        assert!(!policy.cors.allow_all_origins);
    }

    #[test]
    fn parse_environment_from_agents_profile() {
        assert_eq!(
            parse_agents_web_environment(Some("development".to_owned())),
            WebEnvironment::Dev
        );
        assert_eq!(
            parse_agents_web_environment(Some("production".to_owned())),
            WebEnvironment::Prod
        );
    }

    #[test]
    fn resolve_environment_from_agents_env_key() {
        unsafe {
            std::env::set_var("SDKWORK_AGENTS_ENVIRONMENT", "development");
        }
        assert_eq!(
            resolve_agents_web_environment_from_process_env(),
            WebEnvironment::Dev
        );
        unsafe {
            std::env::remove_var("SDKWORK_AGENTS_ENVIRONMENT");
        }
    }
}
