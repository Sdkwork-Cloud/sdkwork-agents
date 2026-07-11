use super::context::{build_web_request_context, AgentRequestContext};
use super::AgentHttpState;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::Request;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::Router;

pub const MAX_HTTP_REQUEST_BODY_BYTES: usize = 2 * 1024 * 1024;

async fn inject_gateway_agent_context(mut request: Request<Body>, next: Next) -> Response {
    let api_surface = classify_api_surface(request.uri().path());
    match AgentRequestContext::from_gateway_subject_headers(request.headers()) {
        Ok(context) => {
            let web_context = build_web_request_context(&context, &request, api_surface);
            request.extensions_mut().insert(context);
            request.extensions_mut().insert(web_context);
            next.run(request).await
        }
        Err(problem) => problem.into_response_fallback(),
    }
}

fn classify_api_surface(path: &str) -> sdkwork_web_core::WebApiSurface {
    use sdkwork_web_core::WebApiSurface;
    if path.starts_with("/app/") {
        WebApiSurface::AppApi
    } else if path.starts_with("/backend/") {
        WebApiSurface::BackendApi
    } else if path.starts_with("/agent/") || path.starts_with("/open/") {
        WebApiSurface::OpenApi
    } else if path.starts_with("/gateway/") {
        WebApiSurface::GatewayApi
    } else {
        WebApiSurface::Unknown
    }
}

async fn trace_request(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started = std::time::Instant::now();
    let response = next.run(request).await;
    let elapsed = started.elapsed();
    let status = response.status().as_u16();
    crate::infrastructure::AgentMetricsRegistry::global().record_http_request(status);
    tracing::info!(
        method = %method,
        path = %path,
        status = response.status().as_u16(),
        elapsed_ms = elapsed.as_millis() as u64,
        "agents.managed_store.request"
    );
    response
}

pub(crate) fn with_gateway_trusted_context(
    router: Router<AgentHttpState>,
) -> Router<AgentHttpState> {
    router
        .layer(DefaultBodyLimit::max(MAX_HTTP_REQUEST_BODY_BYTES))
        .layer(middleware::from_fn(trace_request))
        .layer(middleware::from_fn(inject_gateway_agent_context))
}
