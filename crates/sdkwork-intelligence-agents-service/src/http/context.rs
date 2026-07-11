use crate::response::ApiProblem;
use crate::validation::{parse_owner_user_id, parse_tenant_id};
use axum::body::Body;
use axum::http::{HeaderMap, Request};
use sdkwork_agent_kernel::PolicySubject;

const HEADER_SUBJECT_ID: &str = "x-subject-id";
const HEADER_SUBJECT_TENANT_ID: &str = "x-subject-tenant-id";
const HEADER_SUBJECT_ROLES: &str = "x-subject-roles";
const HEADER_SDKWORK_USER_ID: &str = "x-sdkwork-user-id";
const HEADER_SDKWORK_ACTOR_ID: &str = "x-sdkwork-actor-id";
const HEADER_SDKWORK_TENANT_ID: &str = "x-sdkwork-tenant-id";
const HEADER_SDKWORK_PERMISSION_SCOPE: &str = "x-sdkwork-permission-scope";
const HEADER_SDKWORK_TRACE_ID: &str = "x-sdkwork-trace-id";
const HEADER_SDKWORK_REQUEST_ID: &str = "x-sdkwork-request-id";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRequestContext {
    pub tenant_id: String,
    pub organization_id: Option<String>,
    pub owner_user_id: String,
    pub subject_id: String,
    pub roles: Vec<String>,
    /// W3C trace id resolved from `traceparent` or `x-sdkwork-trace-id` at the gateway boundary.
    pub trace_id: Option<String>,
    /// Server request id used as the Problem+json fallback correlation when no trace id is present.
    pub request_id: Option<String>,
}

impl AgentRequestContext {
    pub fn new(tenant_id: impl Into<String>, owner_user_id: impl Into<String>) -> Self {
        let owner_user_id = owner_user_id.into();
        Self {
            tenant_id: tenant_id.into(),
            organization_id: None,
            subject_id: owner_user_id.clone(),
            owner_user_id,
            roles: Vec::new(),
            trace_id: None,
            request_id: None,
        }
    }

    pub fn with_organization_id(mut self, organization_id: impl Into<String>) -> Self {
        self.organization_id = Some(organization_id.into());
        self
    }

    pub fn with_subject_id(mut self, subject_id: impl Into<String>) -> Self {
        self.subject_id = subject_id.into();
        self
    }

    pub fn with_roles(mut self, roles: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.roles = roles.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub(crate) fn from_gateway_subject_headers(headers: &HeaderMap) -> Result<Self, ApiProblem> {
        let subject_id = required_header_any(
            headers,
            &[
                HEADER_SUBJECT_ID,
                HEADER_SDKWORK_USER_ID,
                HEADER_SDKWORK_ACTOR_ID,
            ],
        )?;
        let tenant_id = required_header_any(
            headers,
            &[HEADER_SUBJECT_TENANT_ID, HEADER_SDKWORK_TENANT_ID],
        )?;
        let mut roles = Vec::new();
        if let Some(roles_header) = optional_header_any(
            headers,
            &[HEADER_SUBJECT_ROLES, HEADER_SDKWORK_PERMISSION_SCOPE],
        ) {
            for role in roles_header
                .split([',', ' '])
                .map(str::trim)
                .filter(|role| !role.is_empty())
            {
                roles.push(role.to_string());
            }
        }
        let request_id = optional_header_any(headers, &[HEADER_SDKWORK_REQUEST_ID])
            .unwrap_or_else(synthesize_request_id);
        let trace_id = optional_header_any(headers, &[HEADER_SDKWORK_TRACE_ID])
            .or_else(|| {
                sdkwork_web_core::trace::resolve_trace_context(headers, &request_id)
                    .traceparent
                    .as_str()
                    .split('-')
                    .nth(1)
                    .map(str::to_owned)
            })
            .or_else(|| Some(request_id.clone()));
        Ok(Self {
            tenant_id,
            organization_id: None,
            owner_user_id: subject_id.clone(),
            subject_id,
            roles,
            trace_id,
            request_id: Some(request_id),
        })
    }

    fn subject(&self) -> PolicySubject {
        let mut subject = PolicySubject::new(self.subject_id.clone(), self.tenant_id.clone());
        for role in &self.roles {
            subject = subject.with_role(role.clone());
        }
        subject
    }
}

pub(crate) fn build_web_request_context(
    agent_context: &AgentRequestContext,
    request: &Request<Body>,
    api_surface: sdkwork_web_core::WebApiSurface,
) -> sdkwork_web_core::WebRequestContext {
    let transport = sdkwork_web_core::WebTransportFacts {
        path: request.uri().path().to_owned(),
        method: request.method().as_str().to_owned(),
        auth_token_present: request.headers().get("authorization").is_some(),
        access_token_present: request.headers().get("x-sdkwork-access-token").is_some(),
        api_key_present: request.headers().get("x-sdkwork-api-key").is_some(),
        oauth_bearer_present: request.headers().get("x-sdkwork-oauth-bearer").is_some(),
        agent_token_present: request.headers().get("x-sdkwork-agent-token").is_some(),
    };
    sdkwork_web_core::WebRequestContext {
        request_id: sdkwork_web_core::ServerRequestId(
            agent_context
                .request_id
                .clone()
                .unwrap_or_else(synthesize_request_id),
        ),
        api_surface,
        auth_mode: sdkwork_web_core::WebAuthMode::DualToken,
        transport,
        principal: None,
        locale: None,
        client_kind: None,
        operation: None,
        trace_id: agent_context.trace_id.clone(),
    }
}

pub(crate) fn synthesize_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("ag-{nanos:032x}")
}

#[derive(Debug, Clone)]
pub(crate) struct RequestScope {
    pub(crate) tenant_id: String,
    pub(crate) organization_id: String,
    pub(crate) owner_user_id: String,
    pub(crate) subject: PolicySubject,
}

impl RequestScope {
    pub(crate) fn from_context(context: AgentRequestContext) -> Self {
        let subject = context.subject();
        Self {
            tenant_id: context.tenant_id.clone(),
            organization_id: context.organization_id.unwrap_or_else(|| "0".to_string()),
            owner_user_id: context.owner_user_id.clone(),
            subject,
        }
    }

    pub(crate) fn owner_scope(&self) -> Result<Option<u64>, ApiProblem> {
        parse_owner_user_id(&self.owner_user_id)
            .map(Some)
            .map_err(ApiProblem::from_kernel_error)
    }

    pub(crate) fn from_trusted_extension(
        mut context: AgentRequestContext,
        resource_tenant_id: String,
        organization_id: Option<String>,
        owner_user_id: Option<String>,
    ) -> Result<Self, ApiProblem> {
        let header_tenant = if context.tenant_id.is_empty() {
            None
        } else {
            Some(context.tenant_id.clone())
        };
        let tenant_id = reconcile_resource_tenant_with_subject_header(
            resource_tenant_id.as_str(),
            header_tenant,
        )?;
        context.tenant_id = tenant_id;
        if let Some(organization_id) = organization_id {
            context.organization_id = Some(organization_id);
        }
        if let Some(owner_user_id) = owner_user_id {
            context.owner_user_id = owner_user_id;
        }
        Ok(Self::from_context(context))
    }

    pub(crate) fn subject(&self) -> &PolicySubject {
        &self.subject
    }

    pub(crate) fn tenant_id_u64(&self) -> Result<u64, ApiProblem> {
        parse_tenant_id(self.tenant_id.as_str()).map_err(ApiProblem::from_kernel_error)
    }
}

pub(crate) fn reconcile_resource_tenant_with_subject_header(
    resource_tenant_id: &str,
    header_tenant_id: Option<String>,
) -> Result<String, ApiProblem> {
    let resource_tenant =
        parse_tenant_id(resource_tenant_id).map_err(ApiProblem::from_kernel_error)?;
    let Some(header_tenant_id) = header_tenant_id else {
        return Err(ApiProblem::validation(
            "subject tenant header is required for backend resource access",
        ));
    };
    let header_tenant = parse_tenant_id(header_tenant_id.as_str())
        .map_err(|_| ApiProblem::permission("subject tenant does not match resource tenant"))?;
    if header_tenant != resource_tenant {
        return Err(ApiProblem::permission(
            "subject tenant does not match resource tenant",
        ));
    }
    Ok(resource_tenant_id.to_string())
}

fn required_header_any(headers: &HeaderMap, keys: &[&str]) -> Result<String, ApiProblem> {
    optional_header_any(headers, keys).ok_or_else(|| {
        ApiProblem::validation(format!("required header missing: {}", keys.join(" or ")))
    })
}

fn optional_header_any(headers: &HeaderMap, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| optional_header(headers, key))
}

fn optional_header(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string)
}
