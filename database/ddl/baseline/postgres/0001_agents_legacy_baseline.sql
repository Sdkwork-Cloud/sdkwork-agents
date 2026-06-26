-- SDKWork Agents application baseline.
-- Agent runtime/session persistence is owned by sdkwork-kernel (sdkwork-agent-database).
-- This module stores application-level deployment and registry metadata only.

CREATE TABLE IF NOT EXISTS agents_app_registry (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT NOT NULL,
    application_key TEXT NOT NULL,
    kernel_slot_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_agents_app_registry_tenant_app
    ON agents_app_registry (tenant_id, application_key);
