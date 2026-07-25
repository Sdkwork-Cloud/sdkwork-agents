use std::collections::BTreeSet;
use std::sync::Arc;

use axum::Router;
use sdkwork_intelligence_agents_service::AgentHttpState;
use sdkwork_routes_agents_http_shared::{agent_request_context_injector, app_route_manifest};
use sdkwork_web_bootstrap::ReadinessCheck;
use sdkwork_web_core::{DomainContextInjector, HttpRoute, HttpRouteManifest};

use crate::readiness::AgentHttpReadinessCheck;

/// Host-neutral Agents App API contribution assembled from one repository-backed state.
pub struct ApiAssemblyContribution {
    pub router: Router,
    pub route_manifest: HttpRouteManifest,
    pub openapi: serde_json::Value,
    pub permission_catalog: Vec<&'static str>,
    pub domain_context_injectors: Vec<Arc<dyn DomainContextInjector>>,
    pub readiness_check: Arc<dyn ReadinessCheck>,
}

/// Builds the unwrapped Agents App API for a gateway that owns the single Web Framework layer.
pub async fn assemble_app_api_contribution() -> anyhow::Result<ApiAssemblyContribution> {
    let state = tokio::task::spawn_blocking(sdkwork_agents_kernel_bridge::build_agent_http_state)
        .await
        .map_err(|error| anyhow::anyhow!("agents state bootstrap worker failed: {error}"))??;
    Ok(contribution_from_state(state))
}

fn contribution_from_state(state: AgentHttpState) -> ApiAssemblyContribution {
    let route_manifest = app_route_manifest();
    let openapi = sdkwork_web_contract::build_openapi_document(
        "SDKWork Agents App API",
        route_manifest.routes(),
    );
    let permission_catalog = permission_catalog(route_manifest.routes());
    let readiness_check = Arc::new(AgentHttpReadinessCheck::new(state.clone()));
    start_owner_background_tasks(&state);
    let router = sdkwork_routes_agents_app_api::build_router().with_state(state);

    ApiAssemblyContribution {
        router,
        route_manifest,
        openapi,
        permission_catalog,
        domain_context_injectors: vec![agent_request_context_injector()],
        readiness_check,
    }
}

fn start_owner_background_tasks(state: &AgentHttpState) {
    // Dropping a Tokio JoinHandle detaches the process-lifetime task. Runtime shutdown still
    // terminates it, while the composing gateway remains independent of Agents worker internals.
    drop(state.spawn_turn_reconciliation_worker());
}

fn permission_catalog(routes: &[HttpRoute]) -> Vec<&'static str> {
    let mut permissions = BTreeSet::new();
    for route in routes {
        if let Some(permission) = route.required_permission {
            permissions.insert(permission);
        }
        if let Some(alternate_permissions) = route.alternate_permissions {
            permissions.extend(alternate_permissions.iter().copied());
        }
    }
    permissions.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdkwork_web_contract::{route_inventory_from_openapi, route_inventory_from_routes};

    #[test]
    fn app_api_manifest_openapi_and_auth_inventories_match() {
        let manifest = app_route_manifest();
        let openapi = sdkwork_web_contract::build_openapi_document(
            "SDKWork Agents App API",
            manifest.routes(),
        );

        assert_eq!(
            route_inventory_from_routes(manifest.routes()),
            route_inventory_from_openapi(&openapi).expect("valid Agents App API OpenAPI inventory")
        );
    }

    #[test]
    fn app_api_permission_catalog_is_the_manifest_permission_union() {
        let manifest = app_route_manifest();
        let catalog = permission_catalog(manifest.routes());
        let mut expected = manifest
            .routes()
            .iter()
            .flat_map(|route| {
                route
                    .required_permission
                    .into_iter()
                    .chain(route.alternate_permissions.into_iter().flatten().copied())
            })
            .collect::<Vec<_>>();
        expected.sort_unstable();
        expected.dedup();

        assert_eq!(expected, catalog);
    }
}
