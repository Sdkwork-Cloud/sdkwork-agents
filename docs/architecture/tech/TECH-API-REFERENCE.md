# SDKWork Agents API And SDK Reference

- Version: `5.1.0`
- Status: active
- Complete operation inventory:
  [TECH-api-specification.md](./TECH-api-specification.md)

## 1. Surface Selection

| Consumer | Surface | SDK | Authentication |
| --- | --- | --- | --- |
| PC, H5, Flutter and product app | App API | `@sdkwork/agents-app-sdk` / `sdkwork_agents_app_sdk` | dual token |
| Operator console and automation | Backend API | `@sdkwork/agents-backend-sdk` | dual token/operator context |
| External integration | Open API | `@sdkwork/agents-sdk` | `X-API-Key` |

Do not call Backend API from user-facing packages. Do not place Open API keys in
the app/backend token manager. Do not deep-import generated transport files.

## 2. Resource Map

| Resource | Purpose | Primary operation namespace |
| --- | --- | --- |
| Agents | Managed definitions and lifecycle | `agents.*` |
| Provider bindings | Provider/model selection | `agents.providerBindings.*` |
| Projects | Reusable orchestration context | `agents.projects.*` |
| Sessions | Durable execution context | `agents.sessions.*` |
| Turns | Idempotent execution lifecycle | `agents.turns.*` |
| Session items | Ordered typed execution facts | `agents.sessionItems.*` |
| Item feedback | Per-user item assessment | `agents.itemFeedback.*` |
| Interactions | Approval and user-question pause points | `agents.interactions.*` |
| Checkpoints | Resume points | `agents.checkpoints.*` |
| Runtime bindings | Session runtime selection | `agents.sessionRuntimeBindings.*` |
| Tasks | Scheduled/deferred commands | `agents.tasks.*` |
| Composition slots | Independent capability references | `agents.compositionSlots.*` |

Surface-specific availability is authoritative in the generated inventory.

## 3. TypeScript App SDK

```typescript
import {
  completeAgentTurn,
  createClient,
  type CreateAgentSessionRequest,
  type CreateAgentTurnRequest,
} from '@sdkwork/agents-app-sdk';

const client = createClient({
  baseUrl: 'https://agents.example.com/app/v3/api',
  tokenManager,
});

const sessionInput: CreateAgentSessionRequest = {
  sessionKind: 'coding',
  entrySurface: 'pc',
  idempotencyKey: sessionKey,
  payloadHash: sessionPayloadHash,
  requestedAt: new Date().toISOString(),
};

const session = await client.ai.sessions.create(agentId, sessionInput);

const turnInput: CreateAgentTurnRequest = {
  content: prompt,
  turnMode: 'interactive',
  idempotencyKey: turnKey,
  payloadHash: turnPayloadHash,
  requestedAt: new Date().toISOString(),
};

const result = await completeAgentTurn(
  client,
  agentId,
  session.item.sessionId,
  turnInput,
);
```

`result` is `{ session, turn, items }`. List/retrieve operations use
`client.ai.sessions`, `client.ai.turns`, `client.ai.sessionItems`,
`client.ai.itemFeedback` and `client.ai.interactions`.

Streaming:

```typescript
for await (const event of client.ai.turns.stream(
  agentId,
  sessionId,
  turnInput,
  { stream: true },
)) {
  if (event.eventType === 'delta') renderDelta(event.delta ?? '');
  if (event.eventType === 'completion') commit(event.response);
}
```

## 4. Flutter App SDK

Dependency package: `sdkwork_agents_app_sdk`.

```dart
import 'package:sdkwork_agents_app_sdk/sdkwork_agents_app_sdk.dart';

final client = SdkworkAppClient.withBaseUrl(
  baseUrl: 'https://agents.example.com',
  authToken: authToken,
  accessToken: accessToken,
);

final session = await client.ai.agentsSessionsCreate(
  agentId,
  CreateAgentSessionRequest(
    sessionKind: 'coding',
    entrySurface: 'flutter',
    idempotencyKey: sessionKey,
    payloadHash: sessionPayloadHash,
    requestedAt: DateTime.now().toUtc().toIso8601String(),
  ),
);

final events = client.ai.agentsTurnsStream(
  agentId,
  sessionId,
  CreateAgentTurnRequest(
    content: prompt,
    turnMode: 'interactive',
    idempotencyKey: turnKey,
    payloadHash: turnPayloadHash,
    requestedAt: DateTime.now().toUtc().toIso8601String(),
  ),
  true,
);
```

The Agents Flutter mobile core accepts the canonical full App surface URL and
normalizes it before constructing this generated client. Feature code receives
the client through bootstrap injection.

## 5. Contract Rules

- Session creation requires `sessionKind`, `entrySurface`, `idempotencyKey`,
  `payloadHash` and `requestedAt`.
- Turn creation requires `content`, `turnMode`, `idempotencyKey`, `payloadHash`
  and `requestedAt`.
- Retry the same command with the same key and hash.
- Treat a key/hash conflict as a client correctness error.
- Use `expectedVersion` for optimistic commands.
- Page with `page` and language-level `pageSize`; the SDK serializes
  `page_size`.
- Treat Session Items as execution facts, not IM communication resources.
- Persist only stable Agents identifiers in a consuming product.

## 6. Authorities And Verification

- Authored OpenAPI:
  `crates/sdkwork-intelligence-agents-service/specs/openapi/`
- SDK family manifests: `sdks/sdkwork-agents-*-sdk/sdk-manifest.json`
- Database:
  `crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`

```powershell
node scripts/generate-agents-api-docs.mjs --check
node scripts/check-agent-sdk-workspace.mjs
node sdks/workspace-agent-sdkgen.mjs --mode dry-run
```
