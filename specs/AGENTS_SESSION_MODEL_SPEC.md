# SDKWork Agents Session Model Specification

- Version: `5.1.0`
- Status: active
- Owner: `agents-platform`
- Authority: `AGENTS_DOMAIN_SPEC.md`

## 1. Aggregate

`AgentSession` is the only durable product session aggregate for agent
execution. Every product and integration uses this aggregate directly through
an Agents API or SDK.

```text
AgentProject
  `- AgentSession
       |- AgentSessionRuntimeBinding
       |- AgentTurn
       |    `- AgentSessionItem (ordered)
       |- AgentInteraction
       `- AgentSessionCheckpoint
```

The aggregate is transactional. It has no alternate session store, transcript
store, read-model table, shadow table, or dual-write path.

## 2. Session

An `AgentSession` carries:

- tenant, organization, owner, agent and optional Agent Project scope;
- `sessionKind`: `assistant`, `coding`, `automation` or `im_dispatch`;
- `entrySurface` and bounded source context identifiers;
- title, lifecycle, item counters, token totals and optimistic version;
- optional parent session and fork-turn lineage.

Creation is idempotent and requires `sessionKind`, `entrySurface`,
`idempotencyKey`, `payloadHash` and `requestedAt`. Tenant, organization and
owner are resolved only from trusted request context.

A session does not contain IM resource identities, an embedded Workspace
snapshot, filesystem paths, copied catalogs, raw credentials or unbounded
provider metadata.

## 3. Runtime Binding

`AgentSessionRuntimeBinding` records the selected runtime for one session:

- Agents provider binding and model references;
- an opaque `runtimeLocationId` supplied by a product;
- host mode and transport kind;
- provider-native session, tree, parent and fork identifiers;
- lifecycle and optimistic version.

Only one current binding may be active for a session. Runtime-location details
remain owned by the product or runtime-location module.

## 4. Turn

An `AgentTurn` is one idempotent execution command. It owns:

- client request and idempotency keys plus payload hash;
- request and response item references;
- mode, lifecycle, model/provider selection and usage;
- sanitized error and trace information;
- bounded retry, availability, lease and fencing state;
- start, completion, cancellation and retention timestamps.

Creation requires `content`, `turnMode`, `idempotencyKey`, `payloadHash` and
`requestedAt`. A retry uses the same idempotency key. A different payload hash
for that key is rejected. Terminal states are `completed`, `failed` and
`cancelled`.

## 5. Session Item

An `AgentSessionItem` is an ordered, typed execution fact. Its kind is one of:

```text
user_input
system_instruction
assistant_output
reasoning
tool_call
tool_result
artifact_reference
status_notice
error_notice
```

Item state is `pending`, `completed`, `failed`, `cancelled` or `redacted`.
Completed content is immutable except for governed redaction. Text is bounded;
tool arguments and results use typed JSON contracts; artifacts use
`ai_agent_item_drive_ref`. Agents stores no raw bytes, signed URLs, IM delivery
state or read cursors.

## 6. Interaction

`AgentInteraction` is a durable pause point linked to a session and optionally a
turn. Supported kinds are approval and user question. Claim and resolution use
optimistic versioning plus a one-time claim token so concurrent service
instances cannot resolve the same interaction.

Provider-specific interaction identifiers are opaque references. They are not
public resource identities.

## 7. Checkpoint

`AgentSessionCheckpoint` represents a resumable provider or logical execution
point. It stores a stable provider checkpoint reference or Drive-backed state
reference, never an unconstrained state document or plaintext resume token.

Restore validates tenant, organization, owner, agent, session, runtime binding,
checkpoint lifecycle and expected version before invoking the runtime.

## 8. Product Integration

Product applications map their domain context to stable Agents references:

| Product fact | Agents resource |
| --- | --- |
| Project-level orchestration context | `AgentProject` |
| Durable execution context | `AgentSession` |
| One idempotent execution request | `AgentTurn` |
| Ordered input, output, tool or artifact fact | `AgentSessionItem` |
| Runtime selection | `AgentSessionRuntimeBinding` |
| Resume point | `AgentSessionCheckpoint` |
| Approval or user question | `AgentInteraction` |

Agents owns Workspace identity and Project membership. Products own their local
UI selection and device state, retain only opaque Agents resource identifiers
needed for correlation, and do not persist a second agent
execution transcript.

## 9. Verification

```powershell
cargo test -p sdkwork-intelligence-agents-service --features http-axum --lib
cargo test -p sdkwork-intelligence-agents-service --features http-axum --test http_axum_contracts
pnpm check:agents-im-boundary
pnpm db:validate
```
