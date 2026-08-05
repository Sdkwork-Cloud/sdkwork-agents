use axum::Router;
use sdkwork_routes_drive_app_api::wrap_router_with_iam_web_framework;

/// Wire the embedded Drive App API (`/app/v3/api/assets*` and sibling drive
/// routes) into the Agents standalone gateway.
///
/// The Drive assembly owns its database lifecycle (`SDKWORK_DATABASE_URL`,
/// shared with the Agents application), auth policy refresh task, download
/// token signing preflight, and domain outbox dispatcher. Its contribution
/// exposes a raw business router; the Agents gateway supplies the IAM web
/// request context resolver so Drive routes authenticate through the same
/// IAM layer as the Agents and IAM surfaces.
pub(super) async fn wire_drive_app_router() -> Result<Router, String> {
    let contribution = sdkwork_api_drive_assembly::assemble_app_api_contribution()
        .await
        .map_err(|error| format!("failed to assemble embedded Drive app API: {error}"))?;
    let resolver = sdkwork_iam_web_adapter::iam_web_request_context_resolver_from_env().await;
    Ok(wrap_router_with_iam_web_framework(
        resolver,
        contribution.router,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_router_wire_contract_is_present() {
        // The embedding entrypoints are public across the workspace boundary:
        // the Drive assembly must keep exporting the app-api contribution and
        // the raw business router must stay composable for gateway-owned layers.
        let _ = sdkwork_routes_drive_app_api::app_route_manifest();
        let _ = sdkwork_api_drive_assembly::assembly_route_count();
    }
}
