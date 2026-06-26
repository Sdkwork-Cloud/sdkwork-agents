-- Default sdkwork-agents application registry row for development bootstrap.
-- Canonical tenant: 100001, application_key: sdkwork-agents.

INSERT INTO ai_app_registry (
    id,
    tenant_id,
    application_key,
    kernel_slot_id,
    created_at,
    updated_at
)
VALUES (
    1,
    100001,
    'sdkwork-agents',
    'default',
    NOW(),
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    tenant_id = EXCLUDED.tenant_id,
    application_key = EXCLUDED.application_key,
    kernel_slot_id = EXCLUDED.kernel_slot_id,
    updated_at = EXCLUDED.updated_at;
