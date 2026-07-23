use crate::response::ApiProblem;
use axum::body::Body;
use axum::extract::Query;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::collections::HashMap;

const CLIENT_SCOPE_QUERY_KEYS: [&str; 6] = [
    "tenantId",
    "tenant_id",
    "organizationId",
    "organization_id",
    "ownerUserId",
    "owner_user_id",
];

pub(crate) async fn reject_client_scope_selectors(request: Request<Body>, next: Next) -> Response {
    let query = match Query::<HashMap<String, String>>::try_from_uri(request.uri()) {
        Ok(Query(query)) => query,
        Err(rejection) => {
            return ApiProblem::validation(format!(
                "invalid query request: {}",
                rejection.body_text()
            ))
            .into_response_fallback();
        }
    };
    if let Some(key) = CLIENT_SCOPE_QUERY_KEYS
        .iter()
        .find(|key| query.contains_key(**key))
    {
        let problem = ApiProblem::validation(format!(
            "{key} is derived from the authenticated request context and must not be supplied"
        ));
        if let Some(context) = request
            .extensions()
            .get::<sdkwork_web_core::WebRequestContext>()
        {
            return problem.into_response_for(context);
        }
        return problem.into_response_fallback();
    }
    next.run(request).await
}
