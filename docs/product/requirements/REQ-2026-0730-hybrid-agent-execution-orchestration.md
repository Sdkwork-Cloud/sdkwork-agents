# REQ-2026-0730 Hybrid Agent Execution Orchestration

- Owner: `agents-platform`
- Status: blocked
- Priority: P0
- Source: customer
- Updated: `2026-07-30`
- Specs: `REQUIREMENTS_SPEC.md`, `AGENTS_DOMAIN_SPEC.md`, `AGENTS_SESSION_MODEL_SPEC.md`, `AGENTS_KERNEL_BOUNDARY_SPEC.md`, `API_SPEC.md`, `DATABASE_SPEC.md`, `SECURITY_SPEC.md`, `TEST_SPEC.md`

## Problem

Agents owns the canonical Workspace, Project, Session, Turn, Session Item,
Interaction, Task, Runtime Binding, and Checkpoint product model, but it does
not yet own a reviewed local/cloud execution intent or a real Kernel placement
orchestration flow. Persisted Runtime Bindings are not used to place Turn work,
Task execution constructs a transient Session stub, and cancel/restore/stale
reconciliation update database state without proving that the real runtime was
stopped or recovered.

This prevents BirdCoder and other SDK consumers from selecting local or cloud
execution safely and prevents commercial multi-node execution from being
claimed.

## Goals

- Add one reviewed per-Session execution target and an optional Task override.
- Keep all physical placement facts server-owned.
- Orchestrate execution only through one Kernel placement port.
- Separate execution placement correlation from provider/model Session
  continuity.
- Make Turn and Task execution consume the effective placement.
- Preserve tenant, organization, owner, Workspace, Project, Session, and agent
  isolation in every command and query.
- Make cancellation, timeout, reconciliation, checkpoint, and restore affect
  the real execution lifecycle.
- Persist local-profile business data durably on the local deployment and
  cloud-profile business data in the approved tenant-isolated service store.
- Provide truthful capability negotiation to generated SDK consumers.

## Non-Goals

- Owning Kernel node selection, runtime slots, execution leases, routing, or
  fencing.
- Calling Sandbox lifecycle or attachment ports directly.
- Storing host paths, VM/device details, provider allocation references,
  workspace bytes, credentials, or raw lease tokens.
- Allowing product clients to create or update a resolved placement binding.
- Extending the current combined Runtime Binding with more infrastructure
  fields.
- Hand-editing generated SDK output.

## Candidate Contract Requiring Human Review

- `ExecutionTarget = LOCAL | CLOUD`.
- Session creation requires `executionTarget`.
- Task creation may include `executionTargetOverride`; absence inherits the
  Session target.
- `ExecutionPlacementIntent` contains target and reviewed policy references,
  never physical placement.
- `AgentExecutionPlacementBinding` records an opaque Kernel placement
  correlation and product-visible lifecycle.
- `AgentProviderSessionBinding` records provider/model continuity.
- One `AgentExecutionPlacementPort` reserves, renews, releases, cancels, and
  restores execution through Kernel.

Names, request fields, response fields, state, error codes, OpenAPI paths,
database shape, and migration policy remain blocked until the linked review is
approved.

## Acceptance Criteria

1. Trusted Session creation resolves tenant, organization, owner, agent,
   Workspace, and Project scope and stores an approved execution target.
2. A Task references a persisted Session and cannot use a transient or
   synthesized Session. Its effective target is the reviewed override or the
   Session target.
3. Effective target, runtime profile, resource class, residency/network policy,
   deadline, idempotency key, and canonical payload hash form bounded placement
   intent. Clients cannot submit nodes, pool slots, Sandbox ids, paths,
   transports, leases, or fencing values.
4. Execution placement binding and provider Session binding are distinct
   domain types, persistence records, DTOs, and lifecycle rules.
5. Agents calls one injected Kernel placement port. It has no Sandbox crate,
   SDK, RPC, repository, or lifecycle dependency.
6. Turn and Task runtime input includes the authorized Workspace/Project/
   Session scope and resolved placement correlation. Execution cannot bypass
   placement when the target requires it.
7. Placement lifecycle is explicit, versioned, idempotent, and correlated to
   one execution attempt. Only server results can advance it.
8. Cancel, timeout, stale reconciliation, checkpoint restore, and release call
   Kernel and record confirmed, retryable, or failed delivery. Database status
   alone is not completion evidence.
9. Every read/write query and uniqueness/idempotency scope enforces tenant and
   organization boundaries. Cross-organization access inside one tenant is
   covered by negative tests.
10. Aggregate changes, audit, and outbox are committed transactionally. Outbox
    publication is retry-safe and consumers are idempotent.
11. Admission and execution errors distinguish unavailable capability, policy
    denial, quota, queue deadline, capacity, placement failure, lease loss,
    fencing conflict, cancellation, and dependency degradation without leaking
    infrastructure details.
12. An authenticated, versioned, expiring capability result reports whether
    local or cloud execution is currently available for the requested policy
    and why it is unavailable. It is derived from Kernel evidence.
13. Local production profiles use durable owner persistence and audit, define
    backup/restore/purge/upgrade behavior, and do not use process-memory stores
    for required business or security facts.
14. Local execution does not upload Workspace bytes implicitly. Cloud
    execution uses only approved opaque Workspace attachment capability and
    revision references.
15. Tenant/user quotas, fairness, distributed admission, placement leases,
    fencing, and pool capacity are never simulated by the current process-local
    Turn semaphore.
16. Public OpenAPI changes are generated into all owned SDKs and operation
    inventory/docs/checks agree on the authored source count.
17. Migration, rollout, compatibility, downgrade, backfill, and rollback are
    approved before changing the PostgreSQL baseline or production schema.
18. End-to-end tests prove local and cloud Session/Task inheritance, duplicate
    command handling, real cancel/restore, dependency loss, stale completion,
    restart/recovery, and cross-tenant denial.

## Non-Functional Requirements

| Area | Required outcome |
| --- | --- |
| Security | Trusted scope only, least privilege, server-owned placement, no physical details or credentials in public contracts, complete tenant/org negative tests. |
| Privacy | Local execution data remains local by default; cloud residency, retention, export, and deletion follow approved policy references. |
| Reliability | Durable intent, transactional outbox, idempotent orchestration, confirmed cancellation, reconciliation, and no silent fallback. |
| Performance | Bounded requests and pages, indexed scope, distributed admission integration, and no process-local full-list scheduling. |
| Operability | Stable errors, health/capability diagnostics, correlated metrics/traces/logs/audit, and bounded cardinality. |

## Traceability

- [ADR-20260730 Hybrid execution intent and placement orchestration](../../architecture/decisions/ADR-20260730-hybrid-execution-placement-orchestration.md)
- [REVIEW-20260730 Hybrid execution contract](../../engineering/reviews/REVIEW-20260730-hybrid-execution-contract.md)
- [BirdCoder umbrella requirement](../../../../sdkwork-birdcoder/docs/product/requirements/REQ-2026-0006-hybrid-local-cloud-agent-execution.md)
- [Kernel distributed runtime PRD](../../../../sdkwork-kernel/docs/product/prd/PRD-05-distributed-agent-runtime.md)
- [Sandbox runtime pool requirement](../../../../sdkwork-sandbox/docs/product/requirements/REQ-2026-0019-sandbox-runtime-pool-and-fast-allocation.md)

## Verification

Verification commands will be fixed by the accepted API, database, Kernel
port, migration, and release reviews. At minimum they must include focused Rust
domain/application/repository tests, all authored OpenAPI validators, SDK
generation checks, database validation, production security checks,
organization-isolation tests, and real local/cloud integration evidence.

## Blockers

- Candidate public naming and inheritance are not approved.
- Kernel PRD-05 and the placement port are not accepted.
- Sandbox implementation remains unauthorized.
- Local persistence/residency and cloud Workspace byte authority are not
  approved.
- API/SDK/database/security/migration/release reviews are pending.
