use sdkwork_web_core::{
    ServerRequestId, WebApiSurface, WebAuthMode, WebRequestContext, WebTransportFacts,
};

pub fn test_web_context() -> WebRequestContext {
    WebRequestContext {
        request_id: ServerRequestId("req-test-fixed".to_owned()),
        api_surface: WebApiSurface::AppApi,
        auth_mode: WebAuthMode::DualToken,
        principal: None,
        transport: WebTransportFacts {
            path: "/app/v3/api/ai/agents".to_owned(),
            method: "POST".to_owned(),
            auth_token_present: true,
            access_token_present: true,
            api_key_present: false,
            ingress_token_present: false,
            oauth_bearer_present: false,
            agent_token_present: false,
        },
        locale: None,
        client_kind: None,
        operation: None,
        trace_id: Some("trace-test-fixed".to_owned()),
        idempotency_key: None,
    }
}
