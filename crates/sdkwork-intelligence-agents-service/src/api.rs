#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApiOperation {
    pub method: &'static str,
    pub path: &'static str,
    pub tag: &'static str,
    pub operation_id: &'static str,
}

pub const AGENT_OPEN_API_PREFIX: &str = "/agent/v3/api";
pub const AGENT_APP_API_PREFIX: &str = "/app/v3/api";
pub const AGENT_BACKEND_API_PREFIX: &str = "/backend/v3/api";

pub const AGENT_OPEN_API_OPERATIONS: &[ApiOperation] = &[
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.create",
    },
    ApiOperation {
        method: "DELETE",
        path: "/agent/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/agent/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.update",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.create",
    },
    ApiOperation {
        method: "DELETE",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/agent/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.update",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/preview_responses",
        tag: "ai",
        operation_id: "agents.previewResponses.create",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/prompt_optimizations",
        tag: "ai",
        operation_id: "agents.promptOptimizations.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.create",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
        tag: "ai",
        operation_id: "agents.providerBindings.activate",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
        tag: "ai",
        operation_id: "agents.sessions.close",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
        tag: "ai",
        operation_id: "agents.messages.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
        tag: "ai",
        operation_id: "agents.messages.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages/{messageId}",
        tag: "ai",
        operation_id: "agents.messages.retrieve",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.list",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.create",
    },
    ApiOperation {
        method: "GET",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
        tag: "ai",
        operation_id: "agents.tasks.cancel",
    },
    ApiOperation {
        method: "POST",
        path: "/agent/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
        tag: "ai",
        operation_id: "agents.tasks.execute",
    },
];

pub const AGENT_APP_API_OPERATIONS: &[ApiOperation] = &[
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.create",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.update",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.create",
    },
    ApiOperation {
        method: "DELETE",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/app/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.update",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/preview_responses",
        tag: "ai",
        operation_id: "agents.previewResponses.create",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/prompt_optimizations",
        tag: "ai",
        operation_id: "agents.promptOptimizations.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.create",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
        tag: "ai",
        operation_id: "agents.providerBindings.activate",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
        tag: "ai",
        operation_id: "agents.sessions.close",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
        tag: "ai",
        operation_id: "agents.messages.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
        tag: "ai",
        operation_id: "agents.messages.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages/{messageId}",
        tag: "ai",
        operation_id: "agents.messages.retrieve",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
        tag: "ai",
        operation_id: "agents.interactions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
        tag: "ai",
        operation_id: "agents.interactions.approve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
        tag: "ai",
        operation_id: "agents.interactions.answer",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.list",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.create",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
        tag: "ai",
        operation_id: "agents.tasks.cancel",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
        tag: "ai",
        operation_id: "agents.tasks.execute",
    },
    ApiOperation {
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/restore",
        tag: "ai",
        operation_id: "agents.restore",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/code_engines",
        tag: "ai",
        operation_id: "agents.codeEngines.list",
    },
    ApiOperation {
        method: "GET",
        path: "/app/v3/api/ai/mcp_servers",
        tag: "ai",
        operation_id: "agents.mcpServers.list",
    },
];

pub const AGENT_BACKEND_API_OPERATIONS: &[ApiOperation] = &[
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents",
        tag: "ai",
        operation_id: "agents.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/backend/v3/api/ai/agents/{agentId}",
        tag: "ai",
        operation_id: "agents.update",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/audit_events",
        tag: "ai",
        operation_id: "agents.auditEvents.list",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots",
        tag: "ai",
        operation_id: "agents.compositionSlots.create",
    },
    ApiOperation {
        method: "DELETE",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.delete",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.retrieve",
    },
    ApiOperation {
        method: "PATCH",
        path: "/backend/v3/api/ai/agents/{agentId}/composition_slots/{slotId}",
        tag: "ai",
        operation_id: "agents.compositionSlots.update",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
        tag: "ai",
        operation_id: "agents.providerBindings.create",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
        tag: "ai",
        operation_id: "agents.providerBindings.activate",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions",
        tag: "ai",
        operation_id: "agents.sessions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}",
        tag: "ai",
        operation_id: "agents.sessions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/close",
        tag: "ai",
        operation_id: "agents.sessions.close",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/archive",
        tag: "ai",
        operation_id: "agents.sessions.archive",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
        tag: "ai",
        operation_id: "agents.messages.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages",
        tag: "ai",
        operation_id: "agents.messages.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/messages/{messageId}",
        tag: "ai",
        operation_id: "agents.messages.retrieve",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions",
        tag: "ai",
        operation_id: "agents.interactions.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}",
        tag: "ai",
        operation_id: "agents.interactions.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/approve",
        tag: "ai",
        operation_id: "agents.interactions.approve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/sessions/{sessionId}/interactions/{interactionId}/answer",
        tag: "ai",
        operation_id: "agents.interactions.answer",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.list",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks",
        tag: "ai",
        operation_id: "agents.tasks.create",
    },
    ApiOperation {
        method: "GET",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}",
        tag: "ai",
        operation_id: "agents.tasks.retrieve",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/cancel",
        tag: "ai",
        operation_id: "agents.tasks.cancel",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/tasks/{taskId}/execute",
        tag: "ai",
        operation_id: "agents.tasks.execute",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/restore",
        tag: "ai",
        operation_id: "agents.restore",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/status",
        tag: "ai",
        operation_id: "agents.status.create",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_binding_operations_are_registered() {
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "GET",
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.list",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.create",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            "agents.providerBindings.activate",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/preview_responses",
            "agents.previewResponses.create",
        );
        assert_operation(
            AGENT_OPEN_API_OPERATIONS,
            "POST",
            "/agent/v3/api/ai/agents/{agentId}/prompt_optimizations",
            "agents.promptOptimizations.create",
        );

        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "GET",
            "/app/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.list",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.create",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            "agents.providerBindings.activate",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/preview_responses",
            "agents.previewResponses.create",
        );
        assert_operation(
            AGENT_APP_API_OPERATIONS,
            "POST",
            "/app/v3/api/ai/agents/{agentId}/prompt_optimizations",
            "agents.promptOptimizations.create",
        );

        assert_operation(
            AGENT_BACKEND_API_OPERATIONS,
            "GET",
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.list",
        );
        assert_operation(
            AGENT_BACKEND_API_OPERATIONS,
            "POST",
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings",
            "agents.providerBindings.create",
        );
        assert_operation(
            AGENT_BACKEND_API_OPERATIONS,
            "POST",
            "/backend/v3/api/ai/agents/{agentId}/provider_bindings/{bindingId}/activate",
            "agents.providerBindings.activate",
        );
    }

    #[test]
    fn composition_slot_operations_are_registered_for_all_api_boundaries() {
        for (operations, prefix) in [
            (AGENT_OPEN_API_OPERATIONS, AGENT_OPEN_API_PREFIX),
            (AGENT_APP_API_OPERATIONS, AGENT_APP_API_PREFIX),
            (AGENT_BACKEND_API_OPERATIONS, AGENT_BACKEND_API_PREFIX),
        ] {
            for (method, path_suffix, operation_id) in [
                (
                    "GET",
                    "/ai/agents/{agentId}/composition_slots",
                    "agents.compositionSlots.list",
                ),
                (
                    "POST",
                    "/ai/agents/{agentId}/composition_slots",
                    "agents.compositionSlots.create",
                ),
                (
                    "GET",
                    "/ai/agents/{agentId}/composition_slots/{slotId}",
                    "agents.compositionSlots.retrieve",
                ),
                (
                    "PATCH",
                    "/ai/agents/{agentId}/composition_slots/{slotId}",
                    "agents.compositionSlots.update",
                ),
                (
                    "DELETE",
                    "/ai/agents/{agentId}/composition_slots/{slotId}",
                    "agents.compositionSlots.delete",
                ),
            ] {
                assert_operation(
                    operations,
                    method,
                    format!("{prefix}{path_suffix}").as_str(),
                    operation_id,
                );
            }
        }
    }

    #[test]
    fn openapi_specs_expose_provider_binding_and_composition_contracts() {
        let open_openapi = include_str!("../specs/openapi/agents-open-api.openapi.yaml");
        let app_openapi = include_str!("../specs/openapi/agents-app-api.openapi.yaml");
        let backend_openapi = include_str!("../specs/openapi/agents-backend-api.openapi.yaml");

        for (label, openapi, prefix) in [
            ("open", open_openapi, "/agent/v3/api"),
            ("app", app_openapi, "/app/v3/api"),
            ("backend", backend_openapi, "/backend/v3/api"),
        ] {
            for required in [
                format!("{prefix}/ai/agents/{{agentId}}/provider_bindings:"),
                format!("{prefix}/ai/agents/{{agentId}}/composition_slots:"),
                format!("{prefix}/ai/agents/{{agentId}}/composition_slots/{{slotId}}:"),
                "operationId: agents.providerBindings.list".to_string(),
                "operationId: agents.compositionSlots.list".to_string(),
                "operationId: agents.compositionSlots.create".to_string(),
                "operationId: agents.compositionSlots.retrieve".to_string(),
                "operationId: agents.compositionSlots.update".to_string(),
                "operationId: agents.compositionSlots.delete".to_string(),
                "AgentCompositionSlotKind:".to_string(),
                "AgentCompositionSlotRecord:".to_string(),
                "AgentCompositionTargetModule:".to_string(),
                "CreateAgentCompositionSlotRequest:".to_string(),
                "UpdateAgentCompositionSlotRequest:".to_string(),
                "AgentCompositionSlotResponse:".to_string(),
                "AgentCompositionSlotListResponse:".to_string(),
                "x-sdkwork-permission: agent.business.composition_slot.list".to_string(),
                "x-sdkwork-permission: agent.business.composition_slot.create".to_string(),
                "pattern: '^slot\\.[a-z0-9_-]+(\\.[a-z0-9_-]+)*$'".to_string(),
            ] {
                assert!(
                    openapi.contains(required.as_str()),
                    "{label} OpenAPI must contain {required}"
                );
            }

            if label == "open" || label == "app" || label == "backend" {
                for required in [
                    format!("{prefix}/ai/agents/{{agentId}}/sessions:"),
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}:"),
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/messages:"),
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/messages/{{messageId}}:"),
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/close:"),
                    "operationId: agents.sessions.list".to_string(),
                    "operationId: agents.sessions.create".to_string(),
                    "operationId: agents.messages.create".to_string(),
                    "operationId: agents.messages.list".to_string(),
                    "AgentChatCompletionResponse:".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }

            if label == "app" {
                for required in [
                    "AppCreateAgentSessionRequest:".to_string(),
                    "AppSendAgentChatMessageRequest:".to_string(),
                    "AppCloseAgentSessionRequest:".to_string(),
                    format!("{prefix}/ai/code_engines:"),
                    format!("{prefix}/ai/mcp_servers:"),
                    "operationId: agents.codeEngines.list".to_string(),
                    "operationId: agents.mcpServers.list".to_string(),
                    "CodeEngineCatalogListResponse:".to_string(),
                    "McpServerMarketplaceListResponse:".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }

            if label == "open" || label == "backend" {
                for required in [
                    "SendAgentChatMessageRequest:".to_string(),
                    "CreateAgentSessionRequest:".to_string(),
                    "CloseAgentSessionRequest:".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }

            if label == "backend" {
                for required in [
                    format!("{prefix}/ai/agents/{{agentId}}/sessions/{{sessionId}}/archive:"),
                    "operationId: agents.sessions.archive".to_string(),
                    "ArchiveAgentSessionRequest:".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }

            for forbidden in [
                "/ai/knowledge_bases",
                "/ai/memory_stores",
                "operationId: knowledgeBases.",
                "operationId: memoryStores.",
            ] {
                assert!(
                    !openapi.contains(forbidden),
                    "{label} OpenAPI must not contain legacy inline knowledge/memory surface {forbidden}"
                );
            }

            if label == "open" {
                assert!(
                    !openapi.contains("/agent/v3/api/ai/agents/{agentId}/restore:"),
                    "open OpenAPI must not expose agents.restore (app/backend only)"
                );
            }

            if label != "backend" {
                for required in [
                    format!("{prefix}/ai/agents/{{agentId}}/preview_responses:"),
                    format!("{prefix}/ai/agents/{{agentId}}/prompt_optimizations:"),
                    "operationId: agents.previewResponses.create".to_string(),
                    "operationId: agents.promptOptimizations.create".to_string(),
                ] {
                    assert!(
                        openapi.contains(required.as_str()),
                        "{label} OpenAPI must contain {required}"
                    );
                }
            }
        }

        for required in [
            "composition_slot_created",
            "composition_slot_updated",
            "composition_slot_deleted",
        ] {
            assert!(
                backend_openapi.contains(required),
                "backend OpenAPI audit action enum must contain {required}"
            );
        }
    }

    fn assert_operation(operations: &[ApiOperation], method: &str, path: &str, operation_id: &str) {
        assert!(
            operations.iter().any(|operation| {
                operation.method == method
                    && operation.path == path
                    && operation.operation_id == operation_id
            }),
            "{method} {path} must be registered as {operation_id}"
        );
    }

    fn count_openapi_operation_ids(openapi: &str) -> usize {
        // Count only real OpenAPI operations (`operationId: <value>` on one line).
        // Schema properties such as `SdkWorkAsyncData.operationId` use
        // `operationId:` with the type on the next line, so they must be excluded
        // to avoid false positives from the standard envelope schemas.
        openapi
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                let rest = trimmed.strip_prefix("operationId:").unwrap_or("");
                rest.trim().len() > 0
            })
            .count()
    }

    #[test]
    fn surface_operation_counts_match_canonical_inventory() {
        let open_openapi = include_str!("../specs/openapi/agents-open-api.openapi.yaml");
        let app_openapi = include_str!("../specs/openapi/agents-app-api.openapi.yaml");
        let backend_openapi = include_str!("../specs/openapi/agents-backend-api.openapi.yaml");

        assert_eq!(AGENT_OPEN_API_OPERATIONS.len(), 27);
        assert_eq!(AGENT_APP_API_OPERATIONS.len(), 35);
        assert_eq!(AGENT_BACKEND_API_OPERATIONS.len(), 33);

        assert_eq!(
            AGENT_OPEN_API_OPERATIONS.len(),
            count_openapi_operation_ids(open_openapi),
            "open operation registry must match OpenAPI authority"
        );
        assert_eq!(
            AGENT_APP_API_OPERATIONS.len(),
            count_openapi_operation_ids(app_openapi),
            "app operation registry must match OpenAPI authority"
        );
        assert_eq!(
            AGENT_BACKEND_API_OPERATIONS.len(),
            count_openapi_operation_ids(backend_openapi),
            "backend operation registry must match OpenAPI authority"
        );
    }

    #[test]
    fn open_operation_registry_must_not_include_unimplemented_surface_drift() {
        for forbidden in [
            "agents.providerBindings.retrieve",
            "agents.providerBindings.deactivate",
            "agents.providerBindings.delete",
            "agents.sessions.update",
            "agents.codeEngines.health",
            "agents.mcpServers.list",
        ] {
            assert!(
                !AGENT_OPEN_API_OPERATIONS
                    .iter()
                    .any(|operation| operation.operation_id == forbidden),
                "open registry must not include unimplemented {forbidden}"
            );
        }
    }

    #[test]
    fn operation_registry_paths_must_not_include_non_ga_scope_routes() {
        let forbidden_path_markers = [
            "/provider_bindings/{bindingId}/deactivate",
            "/code_engines/{engineKey}/health",
        ];
        for operations in [AGENT_OPEN_API_OPERATIONS] {
            for operation in operations {
                for marker in forbidden_path_markers {
                    assert!(
                        !operation.path.contains(marker),
                        "{} path must not include {marker}",
                        operation.operation_id
                    );
                }
            }
        }

        for forbidden in [
            "agents.providerBindings.retrieve",
            "agents.providerBindings.deactivate",
            "agents.providerBindings.delete",
            "agents.sessions.update",
            "agents.codeEngines.health",
        ] {
            assert!(
                !AGENT_APP_API_OPERATIONS
                    .iter()
                    .any(|operation| operation.operation_id == forbidden),
                "app registry must not include non-GA scope operation {forbidden}"
            );
        }

        for forbidden in [
            "agents.providerBindings.retrieve",
            "agents.providerBindings.deactivate",
            "agents.providerBindings.delete",
            "agents.sessions.update",
            "agents.codeEngines.list",
            "agents.codeEngines.health",
            "agents.mcpServers.list",
        ] {
            assert!(
                !AGENT_BACKEND_API_OPERATIONS
                    .iter()
                    .any(|operation| operation.operation_id == forbidden),
                "backend registry must not include non-GA scope operation {forbidden}"
            );
        }
    }
}
