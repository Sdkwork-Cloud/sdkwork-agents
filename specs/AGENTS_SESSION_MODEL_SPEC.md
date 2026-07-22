# SDKWork Agents Session Model Specification

- Version: `5.0.0`
- Status: active
- Owner: `agents-platform`
- Authority: `AGENTS_DOMAIN_SPEC.md`

## 1. Aggregate

`AgentSession` is the only durable product session aggregate for agent
execution. Product applications must use this aggregate rather than creating a
coding-session or assistant-conversation store.

```text
AgentProject
  `- AgentSession
       |- AgentSessionRuntimeBinding
       |- AgentTurn
       |    `- AgentSessionItem (ordered)
       |- AgentInteraction
       `- AgentSessionCheckpoint
```

## 2. Session

A session carries:

- tenant, organization, owner, agent and optional Agent Project scope;
- `sessionKind`: `assistant`, `coding`, `automation` or `im_dispatch`;
- `entrySurface`: bounded client/runtime source;
- optional stable source context identifiers;
- title, lifecycle, item counters, token totals and optimistic version;
- optional parent session and fork turn lineage.

The session does not contain an IM conversation identifier, product workspace
snapshot, filesystem path, copied model catalog or unbounded provider metadata.

## 3. Runtime Binding

`AgentSessionRuntimeBinding` records the selected runtime for one session:

- Agents provider binding and model references;
- opaque `runtimeLocationId` supplied by a product;
- host mode and transport kind;
- provider-native session/tree/parent/fork identifiers;
- lifecycle and optimistic version.

Only one current binding may be active for a session. Runtime location details
remain owned by the product/runtime-location module.

## 4. Turn

An `AgentTurn` is one idempotent command. It owns:

- client request and idempotency keys plus payload hash;
- request and response item references;
- mode, state, model/provider selection and usage;
- sanitized error and trace information;
- bounded retry, availability, lease and fencing state;
- start, completion, cancellation and retention timestamps.

The terminal states are `completed`, `failed` and `cancelled`. A retry uses the
same idempotency key; a conflicting payload hash is rejected.

## 5. Session Item

An `AgentSessionItem` is immutable after completion except for lifecycle
redaction. Its item kind is one of:

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
Delivery/read semantics belong to IM and are forbidden here. Text items use the
bounded text fields. Tool arguments/results use their typed item contract.
Artifacts use `ai_agent_item_drive_ref`; raw bytes and signed URLs are not
stored by Agents.

## 6. Interaction

`AgentInteraction` is a typed pause point linked to a session and optional turn.
Supported kinds are approval and user question. It is claimed and resolved with
optimistic versioning so two service instances cannot answer it concurrently.
Provider-specific interaction identifiers are opaque references, not public
resource identities.

## 7. Checkpoint

`AgentSessionCheckpoint` represents a resumable provider or logical execution
point. It stores a stable provider checkpoint reference or Drive-backed state
reference, never an unconstrained state document or plaintext resume token.

Checkpoint restore is an Agents command that verifies tenant, organization,
owner, session, runtime binding and lifecycle before invoking the provider.

## 8. Product Integration

BirdCoder maps one coding project to one `AgentProject`. Its former
`ai_coding_session*` and assistant `chat_*` rows migrate into this aggregate:

| Legacy fact | Agents target |
| --- | --- |
| coding/assistant session | `ai_agent_session` |
| turn/operation | `ai_agent_turn` |
| transcript message/event outcome | `ai_agent_session_item` |
| native runtime session | `ai_agent_session_runtime_binding` |
| checkpoint | `ai_agent_session_checkpoint` |
| approval/question | `ai_agent_interaction` |
| file/artifact | `ai_agent_item_drive_ref` |

No BirdCoder session binding or transcript projection table is required.

