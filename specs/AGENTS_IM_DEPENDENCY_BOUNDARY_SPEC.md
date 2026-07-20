# SDKWork Agents And IM Dependency Boundary Specification

- Version: `1.0.0`
- Status: active architecture constraint
- Owner: `agents-platform`
- Consumer: `sdkwork-im`
- Related: `AGENTS_KERNEL_BOUNDARY_SPEC.md`,
  `../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`,
  `../../sdkwork-im/specs/IM_AGENTS_DEPENDENCY_AND_DATABASE_SPEC.md`

## 1. Mandatory Dependency Direction

The only allowed dependency direction is:

```text
sdkwork-im -> sdkwork-agents -> sdkwork-kernel
```

`sdkwork-agents` MUST NOT depend on `sdkwork-im` through Cargo, pnpm, generated
SDKs, HTTP clients, runtime route mounting, database access, copied contracts, or
source aliases. This prohibition applies to backend services, PC/H5/mobile
packages, SDK facades, tests, examples, and deployment assembly.

`sdkwork-im` MAY consume Agents only through the public
`sdkwork-agents-runtime-facade`, `@sdkwork/agents-app-sdk`,
`@sdkwork/agents-backend-sdk`, `sdkwork-api-agents-assembly` for embedded
application host composition, or another explicitly declared public Agents
surface. IM MUST NOT depend on provider crates, generated transport internals,
Agents repositories, or Agents tables.

## 2. Data Ownership

Agents owns:

- agent identity, lifecycle, runtime bindings, and composition slots;
- hosted agent execution sessions and ordered agent execution messages;
- chat turns, inference state, cancellation, errors, model/provider references,
  usage, and execution audit;
- project-scoped agent orchestration and typed Drive-backed message references.

Agents does not own:

- IM conversations, groups, channels, contacts, memberships, invitations,
  presence, read cursors, reactions, pins, threads, or realtime fanout;
- IM-visible human transcript ordering or IM message identifiers;
- the mapping from an IM conversation/message to an Agents session/turn.

Agents tables MUST NOT contain an `im_conversation_id`, `im_group_id`,
`im_message_id`, or foreign key to any `im_*` table. Agents MAY accept an opaque
idempotency key and trusted actor context from a consumer, but the consumer owns
the correlation from its resource to the returned Agents resource identifiers.

## 3. IM Consumption Contract

When IM invokes an agent, IM performs conversation authorization before calling
Agents. Agents then performs its own tenant, agent, session, command, and trusted
caller authorization. A caller-provided tenant or user selector never replaces
the trusted request context.

For every invocation:

1. IM selects an assigned `agent_id` and resolves or creates the corresponding
   Agents session through a public Agents SDK/API.
2. IM supplies a stable idempotency key for the source IM message and target
   agent. Agents rejects a conflicting payload for the same key.
3. Agents persists the execution input, turn state, assistant output, usage, and
   audit as its system of record.
4. IM persists the IM-visible user and agent messages, their conversation
   sequence, and the correlation to the Agents session/turn as its system of
   record.
5. Retry, timeout, and compensation never use cross-module table writes.

The trusted caller and the end-user subject are distinct. The embedded IM
dispatch worker uses the fixed service principal
`service.sdkwork-im.agent-dispatch`; runtime assembly grants that principal the
minimum Agents permission required by the facade. `owner_user_id` is the
on-behalf-of/audit subject and is always enforced as the session owner scope.
IM MUST NOT copy `requested_by` into `AgentsChatActor.subject_id`, attach
`ai.agents.manage` to an end user, or accept caller-selected roles.

The public runtime facade exposes idempotency reconciliation in addition to
session resolution and turn completion. A consumer may read a turn only through
the fully scoped tuple `(tenant_id, organization_id, owner_user_id, agent_id,
session_id, idempotency_key)`. The snapshot contains bounded lifecycle and
response correlation only; it does not expose repository rows or authorize any
cross-module persistence.

Timeout handling is mandatory:

- `completed`: consume the persisted Agents response and finish consumer-side
  correlation without invoking the model again;
- `requested` or `running`: defer reconciliation without consuming a new model
  attempt;
- `failed` or `cancelled`: apply the consumer terminal/retry policy;
- not found: a consumer may retry the same payload with the same idempotency key;
- lookup unavailable: treat the outcome as indeterminate and defer, never infer
  failure from the timeout.

## 4. Frontend Boundary

Standalone Agents applications MUST NOT implement generic group chat by importing
IM packages. A generic group-chat entry is either absent or expressed as an
optional host port. An IM application, as the consumer, may implement that port
and compose Agents UI or SDK surfaces.

Agents-owned project sharing is not an IM conversation. It may use Agents-owned
project membership and share grants, but it MUST NOT reproduce generic IM group,
presence, reaction, or realtime messaging behavior.

## 5. Database Integrity Rules

- Agents and IM databases have separate write owners even when deployed in one
  PostgreSQL cluster.
- Cross-module references are validated through public services and events, not
  database foreign keys.
- Agents deletion MUST NOT delete IM rows. IM deletion MUST NOT delete Agents
  rows directly.
- Consumers use explicit close/revoke commands and retain correlation records for
  audit and compensation.
- Raw credentials, auth tokens, presigned URLs, and IM payloads MUST NOT be copied
  into Agents metadata JSON.

## 6. Verification

The Agents repository MUST retain checks proving:

- no Cargo, pnpm, source alias, or SDK dependency points to `sdkwork-im`;
- no Agents DDL or database contract declares an `im_*` table or IM foreign key;
- public Agents SDK families remain the only supported product integration
  surface;
- chat persistence changes remain aligned with the managed-store database spec.

Required repository checks after implementation changes:

```powershell
pnpm check:architecture-alignment
pnpm check:app-sdk-consumer-imports
pnpm check:rust-backend-composition
pnpm db:validate
```
