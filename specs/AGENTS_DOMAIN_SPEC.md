# SDKWork Agents Domain Specification

- Version: `6.0.0`
- Status: active
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`

## 1. Bounded Context

`sdkwork-agents` is the system of record for managed agents and durable agent
execution. Its canonical aggregate vocabulary is:

```text
AgentWorkspace -> AgentProject -> AgentSession -> AgentTurn -> AgentSessionItem -> AgentInteraction
```

These terms describe agent execution, not instant messaging:

- `AgentWorkspace` is the user-owned Project container; each owner scope has
  one idempotently initialized active default Workspace.
- `AgentProject` groups reusable orchestration policy and sessions within one
  Workspace.
- `AgentSession` is the durable agent execution context.
- `AgentTurn` is one idempotent request and its execution lifecycle.
- `AgentSessionItem` is an ordered typed transcript or execution item.
- `AgentInteraction` is a durable human approval or question pause point.

The application may render a session as a chat UI, but database, API, SDK,
event, service, and repository contracts use the domain terms above.

## 2. Ownership

Agents owns:

- managed agent identity, lifecycle, runtime and composition bindings;
- agent workspaces, Workspace-scoped projects, sessions, turns, session items
  and interactions;
- provider session bindings, fork lineage and resumable checkpoints;
- turn idempotency, retry/lease state, usage, audit and outbox facts;
- typed Drive references attached to session items;
- scheduled agent tasks and per-user Agents resource state.

Agents does not own:

- IM conversations, messages, members, read cursors, reactions or presence;
- prompt library content, skill packages, document content, model catalogs or Drive bytes;
- product-local duplicate workspaces, runtime-location details or filesystem paths;
- kernel provider mechanisms, transient token events, runs or steps.

Cross-domain links are stable identifiers validated through public contracts.
There are no cross-module SQL queries or foreign keys.

## 3. Composition Contract

Agent-level and Project-level composition slots use the same canonical mapping:

| `slotKind` | `targetModule` | External owner |
| --- | --- | --- |
| `memory` | `memory` | `sdkwork-memory` |
| `knowledge` | `knowledgebase` | `sdkwork-knowledgebase` |
| `skill` | `skills` | `sdkwork-skills` |
| `prompt` | `prompts` | `sdkwork-prompts` |
| `drive` | `drive` | `sdkwork-drive` |
| `document` | `documents` | `sdkwork-documents` |
| `tool` | `tools` | Agents tool orchestration contract |
| `mcp` | `mcp` | `sdkwork-mcp` |

Only the pair shown in each row is valid. In particular, `document` pairs only
with `documents`; it cannot be represented as `drive`, `knowledgebase`, or a
product-local alias. Agents stores the stable `targetRef`, optional
`targetVersionRef`, ordering and bounded orchestration policy. The external
owner retains resource content, versions, authorization and lifecycle.

## 4. Dependency Direction

```text
product applications -----> sdkwork-agents -----> sdkwork-kernel
sdkwork-im ----------------^          |
                                      +-----> independent capability SDKs
```

The mandatory communication dependency is:

```text
sdkwork-im -> sdkwork-agents -> sdkwork-kernel
```

Agents never imports IM packages, SDKs, repositories, routes or tables. IM owns
any correlation between an IM message and an Agents session/turn.

## 5. Stable External Context

Consumers may supply these bounded references:

- `sourceModule`, `sourceContextKind`, `sourceContextId` on session creation;
- `runtimeLocationId` on a session runtime binding;
- `providerBindingId`, `modelId` and provider session identifiers;
- Drive resource identifiers on item relations.

Agents stores no snapshot of the external resource and never resolves it with
cross-domain SQL. The caller performs its own product authorization and Agents
performs tenant, organization, owner, agent and session authorization.

## 6. Canonical Naming

| Agents concept | Canonical identifier | Forbidden new identifier |
| --- | --- | --- |
| Workspace | `workspaceId` / `ai_agent_workspace` | product-local Workspace id |
| Session | `sessionId` / `ai_agent_session` | `conversationId`, `chatSessionId` |
| Turn | `turnId` / `ai_agent_turn` | `chatTurnId` |
| Session item | `itemId` / `ai_agent_session_item` | `messageId`, `chatMessageId` |
| Item feedback | `ai_agent_item_feedback` | IM reaction, message feedback |
| Item Drive relation | `ai_agent_item_drive_ref` | message attachment table |
| Checkpoint | `checkpointId` / `ai_agent_session_checkpoint` | product-local checkpoint |

UI copy may use "conversation" or "message" where it improves usability. Those
words do not become Agents business resource names.

## 7. Completion Rules

The domain is aligned only when:

- BirdCoder and other products persist no second agent session transcript;
- Agents authored contracts contain no `ai_agent_message` or
  `ai_agent_chat_turn` tables;
- Agents public APIs use `sessions`, `turns`, `items`, `interactions` and
  `checkpoints`;
- Agents has no dependency on `sdkwork-im`;
- all input, output, database, SDK and documentation terms match this spec.
