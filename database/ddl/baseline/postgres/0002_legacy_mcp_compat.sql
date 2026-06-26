-- Legacy MCP marketplace table (deferred migration to sdkwork-mcp).
-- Retained for runtime compatibility until the MCP module database lands.

CREATE TABLE IF NOT EXISTS a_agent_mcp_server (
    id BIGINT NOT NULL PRIMARY KEY,
    uuid VARCHAR(96) NOT NULL,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    owner_user_id BIGINT NOT NULL,
    mcp_server_id VARCHAR(128) NOT NULL,
    code VARCHAR(128) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    protocol_version VARCHAR(32) NOT NULL,
    transport_kind VARCHAR(32) NOT NULL,
    endpoint_ref VARCHAR(128),
    command_ref VARCHAR(128),
    auth_kind VARCHAR(32) NOT NULL,
    auth_profile_id VARCHAR(128),
    capability_ids_json TEXT NOT NULL DEFAULT '[]',
    tool_count BIGINT NOT NULL DEFAULT 0,
    resource_count BIGINT NOT NULL DEFAULT 0,
    prompt_count BIGINT NOT NULL DEFAULT 0,
    capabilities_json TEXT NOT NULL DEFAULT '{}',
    categories_json TEXT NOT NULL DEFAULT '[]',
    tags_json TEXT NOT NULL DEFAULT '[]',
    security_profile_id VARCHAR(128),
    status SMALLINT NOT NULL,
    visibility SMALLINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP NULL,
    CONSTRAINT uk_a_agent_mcp_server_uuid UNIQUE (uuid),
    CONSTRAINT uk_a_agent_mcp_server_tenant_server UNIQUE (tenant_id, mcp_server_id),
    CONSTRAINT uk_a_agent_mcp_server_tenant_code UNIQUE (tenant_id, code),
    CONSTRAINT ck_a_agent_mcp_server_id_standard CHECK (
        mcp_server_id ~ '^mcp\.server\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_transport_kind CHECK (
        transport_kind IN ('stdio', 'http', 'sse', 'websocket')
    ),
    CONSTRAINT ck_a_agent_mcp_server_transport_refs CHECK (
        (transport_kind = 'stdio' AND command_ref IS NOT NULL)
        OR (transport_kind IN ('http', 'sse', 'websocket') AND endpoint_ref IS NOT NULL)
    ),
    CONSTRAINT ck_a_agent_mcp_server_endpoint_ref_standard CHECK (
        endpoint_ref IS NULL
        OR endpoint_ref ~ '^endpoint\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_command_ref_standard CHECK (
        command_ref IS NULL
        OR command_ref ~ '^command\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_auth_kind CHECK (
        auth_kind IN ('none', 'oauth2', 'api-key-ref', 'host-secret-ref')
    ),
    CONSTRAINT ck_a_agent_mcp_server_auth_profile_standard CHECK (
        auth_profile_id IS NULL
        OR auth_profile_id ~ '^profile\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_security_profile_standard CHECK (
        security_profile_id IS NULL
        OR security_profile_id ~ '^profile\.[a-z0-9_-]+(\.[a-z0-9_-]+)*$'
    ),
    CONSTRAINT ck_a_agent_mcp_server_capability_ids_standard CHECK (
        sdkwork_intelligence_agents_service_capabilities_json_is_standard(capability_ids_json)
    ),
    CONSTRAINT ck_a_agent_mcp_server_counts_non_negative CHECK (
        tool_count >= 0 AND resource_count >= 0 AND prompt_count >= 0
    ),
    CONSTRAINT ck_a_agent_mcp_server_capabilities_json CHECK (capabilities_json::jsonb IS NOT NULL),
    CONSTRAINT ck_a_agent_mcp_server_status CHECK (status IN (0, 1, 2, 3, 4)),
    CONSTRAINT ck_a_agent_mcp_server_visibility CHECK (visibility IN (0, 1, 2, 3))
);

CREATE INDEX IF NOT EXISTS idx_a_agent_mcp_server_tenant_org_status_updated
    ON a_agent_mcp_server (tenant_id, organization_id, status, updated_at DESC, code ASC);

CREATE INDEX IF NOT EXISTS idx_a_agent_mcp_server_tenant_transport_auth
    ON a_agent_mcp_server (tenant_id, transport_kind, auth_kind);
