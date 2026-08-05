# SDKWork Agent SDK

`sdkwork-agents-sdk` is the developer/open SDK family for the agent domain.

| Field | Value |
| --- | --- |
| SDK family | `sdkwork-agents-sdk` |
| API authority | `sdkwork-agents-open-api` |
| API prefix | `/agent/v3/api` |
| TypeScript package | `@sdkwork/agents-sdk` |
| SDK generator type | `custom` until external `sdkwork-v3` generator supports `/agent/v3/api` |
| Audience | Developer and integration authors |

The authority OpenAPI is `openapi/sdkwork-agents-open-api.openapi.yaml`.
The derived generator input is `openapi/sdkwork-agents-open-api.sdkgen.yaml`.

Generated TypeScript transport output belongs under
`sdkwork-agents-sdk-typescript/generated/server-openapi`.

## Generate

Run from repository root:

```powershell
node .\sdks\workspace-agent-sdkgen.mjs --family open --mode dry-run
node .\sdks\workspace-agent-sdkgen.mjs --family open --mode apply
```

The generator command uses:

```text
--standard-profile sdkwork-v3
--api-prefix /agent/v3/api
--package-name @sdkwork/agents-sdk
```

The OpenAPI authority is complete and locally validated. The current
`sdkwork-v3` standard profile is limited to `app`, `backend`, and `im`
prefixes for direct sdkgen, so `/agent/v3/api` transport is derived from the
strict-profile app SDK source by `materialize-agent-open-sdk-from-app.mjs`.
The derivation strips app-only operations (`restore`, `agentEngines`,
`mcpServers`) so the open SDK surface matches the 27-operation open authority.

## Consume

Developer tooling, integrations, and local hosts should consume
`@sdkwork/agents-sdk` rather than calling `/agent/v3/api` with raw HTTP.

## SDKWork Documentation Contract

Domain: intelligence
Capability: agent-open-sdk
Package type: sdk-family
Status: standardized

### Public API

Public exports are declared in `specs/component.spec.json` under `contracts.publicExports`.

### Required SDK Surface

- `SdkworkAgentClient`

### Configuration

Configuration keys and runtime entrypoints are declared in `specs/component.spec.json`.

### SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`, including deployment and runtime configuration rules where applicable.

### Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

### Extension Points

Extension points are limited to declared public exports, runtime entrypoints, SDK clients, events, and config keys.

### Verification

- `node sdks/materialize-agent-v3-openapi-boundaries.mjs`
- `node sdks/sdkwork-agents-sdk/bin/verify-sdk.mjs`
- `node sdks/test/verify-agent-sdk-ownership-boundaries.test.mjs`
- `node scripts/check-agent-sdk-workspace.mjs`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
