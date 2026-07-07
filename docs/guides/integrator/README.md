# Integrator Guide

SDK consumption and HTTP boundaries for external applications integrating with SDKWork Agents.

## API surfaces

| Surface | Prefix | SDK family | Auth |
| --- | --- | --- | --- |
| App API | `/app/v3/api` | `@sdkwork/agents-app-sdk` | IAM app session / access token |
| Backend API | `/backend/v3/api` | `@sdkwork/agents-backend-sdk` | Backend credential |
| Open API | `/agent/v3/api` | `@sdkwork/agents-sdk` | Open credential / gateway trust |

Canonical HTTP contract: [TECH-api-specification.md](../../architecture/tech/TECH-api-specification.md).

## Response envelope (required)

All SDKWork-owned business HTTP operations return:

- **Success (2xx):** `{ "code": 0, "data": <payload>, "traceId": "<uuid>" }`
- **Error (4xx/5xx):** `application/problem+json` (`ProblemDetail`) with numeric `code` and `traceId`

Generated TypeScript SDKs (`--standard-profile sdkwork-v3`) unwrap `data` by default. Do not call managed-agent HTTP with raw `fetch` wrappers.

## Client application integration

For PC / H5 / mini-program surfaces:

1. Declare SDK dependencies in `*-core` `component.spec.json` (`contracts.sdkDependencies`).
2. Wire clients only in `*-core/src/sdk/*AppSdkClient.ts`.
3. Import types and clients from `@sdkwork/agents-*-core/sdk` in capability packages — never from `@sdkwork/agents-app-sdk` directly.

Knowledgebase selection uses `@sdkwork/knowledgebase-app-sdk` through `*-core/sdk/knowledgebaseAppSdkClient`.

## OpenAPI authority

Route counts and operation IDs are owned by generated OpenAPI under `sdks/sdkwork-agents-*-sdk/`. Regenerate SDKs after contract changes; do not hand-edit generated transport output.

## Verification before integration

```powershell
pnpm check:api-envelope
pnpm check:api-operation-patterns
pnpm check:route-path-collisions
pnpm check:pagination
pnpm check:app-sdk-consumer-imports
pnpm verify
```

## Related

- [apis/README.md](../../../apis/README.md)
- [specs/README.md](../../../specs/README.md)
- [TECH-api-reference.md](../../architecture/tech/TECH-api-reference.md)
