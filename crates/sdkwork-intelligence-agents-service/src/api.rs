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
        method: "POST",
        path: "/app/v3/api/ai/agents/{agentId}/restore",
        tag: "ai",
        operation_id: "agents.restore",
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
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/restore",
        tag: "ai",
        operation_id: "agents.restore",
    },
    ApiOperation {
        method: "POST",
        path: "/backend/v3/api/ai/agents/{agentId}/status",
        tag: "ai",
        operation_id: "agents.status.update",
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
        let backend_openapi =
            include_str!("../specs/openapi/agents-backend-api.openapi.yaml");

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
}
