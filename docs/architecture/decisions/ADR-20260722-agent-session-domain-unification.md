# ADR-20260722 Agent Session Domain Unification

- Status: accepted
- Date: `2026-07-22`
- Owner: `agents-platform`
- Requirement:
  [REQ-2026-0722-agent-session-execution.md](../../product/requirements/REQ-2026-0722-agent-session-execution.md)

## Context

Managed agent execution needs durable context, idempotent commands, ordered
execution facts, provider continuity, checkpoints and typed human pause points.
Those semantics are independent of product UI style and IM delivery semantics.

## Decision

Agents owns one canonical aggregate:

```text
AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
```

All products use this aggregate through Agents APIs or SDKs. Runtime bindings
and checkpoints extend Session continuity. `sdkwork-kernel` owns execution
mechanisms. `sdkwork-im` owns communication resources and may retain opaque
Agents Session/Turn correlation.

The aggregate is the sole product persistence authority. New client surfaces,
providers and independent capabilities extend it through SDKs, manifests,
bindings and composition slots without changing its core vocabulary.

## Consequences

- API, SDK, database, service and documentation names use Session, Turn,
  SessionItem and Interaction consistently.
- Products do not persist a second agent execution history.
- IM and Agents keep separate data owners and transaction boundaries.
- Provider-native identities remain opaque runtime references.
- Skills, prompts, memory, knowledge, MCP, model profiles and Drive remain
  independent modules.
- There is no compatibility facade, shadow persistence or cross-module SQL.

## Verification

```powershell
pnpm check:agents-im-boundary
pnpm db:validate
pnpm check:api-operation-patterns
pnpm check:agent-sdk-workspace
cargo test -p sdkwork-intelligence-agents-service --features http-axum
cargo test -p sdkwork-agents-runtime-facade
```

## Supersedes / Superseded By

None.
