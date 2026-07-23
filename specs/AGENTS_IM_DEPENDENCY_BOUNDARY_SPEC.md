# SDKWork Agents And IM Dependency Boundary Specification

- Version: `1.1.0`
- Status: active architecture constraint
- Owner: `agents-platform`
- Consumer: `sdkwork-im`
- Related:
  - `AGENTS_KERNEL_BOUNDARY_SPEC.md`
  - `AGENTS_SESSION_MODEL_SPEC.md`
  - `../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`
  - `../../sdkwork-im/specs/IM_AGENTS_DEPENDENCY_AND_DATABASE_SPEC.md`

## 1. Dependency Direction

```text
sdkwork-im -> sdkwork-agents -> sdkwork-kernel
```

Agents never depends on IM through Cargo, pnpm, generated SDKs, HTTP clients,
runtime mounting, source aliases, repositories or database access. IM consumes
Agents through a public Agents API/SDK, runtime facade, or approved embedded API
assembly.

## 2. Semantic Separation

Agents execution semantics and IM communication semantics are different:

| Agents business resource | Meaning | IM-owned concept |
| --- | --- | --- |
| `AgentSession` | durable agent execution context | conversation/channel context |
| `AgentTurn` | idempotent execution command | delivery of an IM message |
| `AgentSessionItem` | ordered execution input/output/tool/artifact fact | IM-visible message/event |
| `AgentInteraction` | approval or user-question pause point | reaction, read state or presence |

An Agents UI may visually resemble a dialog, but that does not make its durable
resources IM resources. IM ordering, delivery, fanout and read semantics never
enter the Agents aggregate.

## 3. Data Ownership

Agents owns managed-agent identity, execution sessions, turns, session items,
interactions, checkpoints, usage, audit and Drive references.

IM owns conversations, groups, channels, contacts, memberships, invitations,
presence, delivery, read cursors, reactions, pins, threads, realtime fanout and
the correlation from an IM invocation to returned Agents resources.

Agents tables contain no IM foreign key or IM resource column. IM stores only
opaque `agentId`, `agentSessionId` and `agentTurnId` correlations required by its
dispatch workflow. Neither module writes the other module's tables.

## 4. Dispatch Contract

For one IM-triggered invocation:

1. IM authorizes the IM participant and target conversation.
2. IM resolves or creates one Agents Session through a public SDK.
3. IM submits a Turn with a stable idempotency key derived from its own source
   event and target agent.
4. Agents authorizes trusted context and persists its Session, Turn and Session
   Items atomically.
5. IM persists its own visible communication facts and opaque Agents
   correlation.
6. Retries and compensation use public commands, never cross-module SQL.

The embedded dispatch worker uses service principal
`service.sdkwork-im.agent-dispatch`. The on-behalf-of user remains the enforced
Agents session owner. Caller-supplied tenant, organization, user or role values
never replace `AgentRequestContext`.

## 5. Timeout Reconciliation

Reconciliation uses the fully scoped tuple `(tenant, organization, owner,
agent, session, idempotencyKey)`:

- `completed`: consume the persisted response without invoking the model again;
- `requested` or `running`: defer and retry reconciliation;
- `failed` or `cancelled`: apply the consumer terminal policy;
- not found: retry the same payload with the same idempotency key;
- lookup unavailable: treat the outcome as indeterminate and defer.

The reconciliation response is bounded lifecycle and resource correlation. It
does not expose repository rows or grant cross-module persistence access.

## 6. Security And Integrity

- Both modules enforce their own tenant and authorization boundary.
- Cross-module references have no database foreign keys.
- Deletion never cascades across module stores.
- Credentials, session tokens, signed URLs and complete IM payloads are never
  copied into Agents metadata.
- Consumers use the Agents SDK and do not import generated transport internals.

## 7. Verification

```powershell
pnpm check:agents-im-boundary
pnpm check:app-sdk-consumer-imports
pnpm check:rust-backend-composition
pnpm db:validate
```
