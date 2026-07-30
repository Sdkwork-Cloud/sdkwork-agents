# ADR-20260730 Hybrid Execution Intent And Placement Orchestration

- Status: proposed
- Date: `2026-07-30`
- Owner: `agents-platform`
- Requirement: [REQ-2026-0730](../../product/requirements/REQ-2026-0730-hybrid-agent-execution-orchestration.md)

## Context

Agents is the product authority for durable execution semantics, while Kernel
owns runtime mechanisms and Sandbox owns isolated environment mechanisms. The
current Session Runtime Binding combines provider/model continuity with
caller-written runtime-location, host, and transport fields. It is not consumed
as real placement input by Turn execution. Task execution also bypasses the
canonical Session aggregate.

A hybrid product needs durable user intent without exposing infrastructure
placement to clients or moving Sandbox mechanics into Agents.

## Decision

Agents will own execution intent and orchestration, but not placement
resolution:

```text
generated product SDK
  -> Agents command and authorization
    -> durable execution intent
      -> AgentExecutionPlacementPort
        -> Kernel placement/control plane
```

Agents never calls Sandbox directly.

### Aggregate Shape

The candidate aggregate extension is:

```text
AgentWorkspace
  -> AgentProject
    -> AgentSession (default execution target)
      -> AgentExecutionPlacementBinding (server-owned correlation)
      -> AgentProviderSessionBinding (provider/model continuity)
      -> AgentTurn (effective placement attempt)
      -> AgentTask (Session reference, optional target override)
      -> AgentSessionCheckpoint
```

The exact persistence design requires database review. It must preserve one
write owner, organization-leading isolation, transactional audit/outbox, and a
direct rollback path. It cannot overload the existing combined binding with
physical Kernel or Sandbox fields.

### Intent And Effective Target

Session creation stores the reviewed target. Task override is optional and is
resolved once when an execution attempt is created. Turn inherits its Session
target unless its owning reviewed command model explicitly links it to a Task
attempt. An active attempt cannot change target in place.

Placement intent contains policy ids and bounded command context. Kernel
resolves capacity, node, provider, lease, fencing, transport, and Sandbox
attachments. Agents stores only opaque correlation and product-visible state.

### Port Boundary

The candidate `AgentExecutionPlacementPort` supports the minimum behaviors:

- reserve placement for one authorized execution attempt;
- renew ownership while work is active;
- cancel real execution;
- restore an approved checkpoint into a new attempt;
- release placement and report confirmed or retryable outcome;
- retrieve versioned target capability for product negotiation.

The accepted review must define exact names, inputs, outputs, auth context,
idempotency, deadlines, error taxonomy, lease credential handling, and version
compatibility. BirdCoder never receives raw Kernel lease credentials.

### State And Consistency

Agents placement correlation uses an explicit lifecycle compatible with
Kernel's reviewed lifecycle. State transitions use optimistic versioning and
idempotency. A terminal product state is committed only after the command's
required runtime outcome is known or a durable reconciliation obligation is
recorded.

Session/Turn/Task changes, audit, and outbox are atomic. Kernel calls have
separate provider idempotency keys so retry after an ambiguous network result
does not allocate a second placement.

### Capability Negotiation

Agents exposes target availability through a generated SDK contract only after
the public review. Capability is authenticated and policy-scoped, has contract
version and expiry, and contains stable unavailable reasons. It is derived from
Kernel evidence and does not infer from Agents deployment profile, BirdCoder
client host, or a configuration boolean.

### Local And Cloud Persistence

Local and cloud topologies use the same Agents domain model. The local profile
uses approved durable owner persistence on the device or its local deployment
volume and must define backup, restore, upgrade, purge, and offline/degraded
identity behavior. The cloud profile uses approved tenant-isolated service
persistence. BirdCoder's device-state database is never an Agents business
store.

Agents stores only Workspace attachment capability/revision references. Drive
or another approved storage owner retains bytes, encryption, retention,
residency, backup, and deletion authority.

## Alternatives

### Let Product Clients Create Runtime Placement Bindings

Rejected because clients cannot validate capacity, node trust, lease/fencing,
attachment readiness, or cleanup and could forge infrastructure state.

### Keep Provider And Placement In One Binding

Rejected because provider continuity can outlive a placement and one placement
attempt can be retried without changing provider Session identity.

### Agents Calls Sandbox Directly

Rejected because placement, routing, execution ownership, cancellation, and
recovery would have two control planes.

### Task Creates A Session Stub

Rejected because authorization, Workspace/Project association, runtime
binding, history, checkpoint, and retention invariants would be bypassed.

## Consequences

- Public API, SDK, and PostgreSQL migration work is required after approval.
- Current local client-created bindings remain transitional and cannot be used
  as cloud evidence.
- Turn runtime input and Task execution must be changed to consume placement.
- Cancel, restore, timeout, and reconciliation need real Kernel operations.
- Local persistence and commercial cloud operation require separate real
  environment evidence.

## Verification

- Domain tests prove target validation, inheritance, immutability, canonical
  Session references, and placement/provider binding separation.
- Authorization/repository tests prove tenant and organization isolation for
  all new resources and queries.
- Application tests prove idempotent reserve/renew/release, real cancel and
  restore, ambiguous response recovery, and transactional outbox/audit.
- API/SDK tests prove generated contract parity and reject client-owned
  placement fields.
- Integration tests prove local restart/recovery and cloud Kernel/Sandbox
  placement without direct Agents-to-Sandbox dependencies.

## Supersedes / Superseded By

This proposed ADR narrows the hybrid extension of
ADR-20260722-agent-session-domain-unification. It does not supersede the
canonical Session aggregate or ownership boundary.
