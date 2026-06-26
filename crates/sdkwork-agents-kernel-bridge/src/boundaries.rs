//! Capability ownership between SDKWork Agents application and sdkwork-kernel.

/// Capabilities owned by the Agents application repository.
pub const AGENTS_OWNED_CAPABILITIES: &[&str] = &[
    "application-topology",
    "deployment-packaging",
    "application-registry-metadata",
    "product-runtime-config",
    "agents-domain-service",
    "agents-http-routes",
    "agents-persistence",
    "agents-sdks",
    "agents-api-contracts",
];

/// Capabilities owned by sdkwork-kernel (runtime SPI only).
pub const KERNEL_OWNED_CAPABILITIES: &[&str] = &[
    "agent-runtime",
    "agent-session",
    "agent-model-provider",
    "agent-tool-provider",
    "agent-runtime-persistence",
    "agent-internal-api",
    "agent-server-operational-http",
];
