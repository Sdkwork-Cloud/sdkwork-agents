# REVIEW-20260730 Hybrid Execution Contract

- Status: pending-human-review
- Outcome: No-Go
- Date: `2026-07-30`
- Owner: `agents-platform`
- Requirement: [REQ-2026-0730](../../product/requirements/REQ-2026-0730-hybrid-agent-execution-orchestration.md)
- Decision: [ADR-20260730](../../architecture/decisions/ADR-20260730-hybrid-execution-placement-orchestration.md)

## Findings

| Severity | Finding | Required closure |
| --- | --- | --- |
| P0 | No public execution target or Task override exists. | Approve domain/API/SDK naming, inheritance, validation, compatibility, and capability negotiation. |
| P0 | Runtime Binding combines caller-written placement-like fields and provider continuity. | Approve split model and migration/removal of client placement writes. |
| P0 | Persisted binding is not consumed by Turn placement. | Make runtime input and execution path require resolved placement. |
| P0 | Task uses a transient Session stub. | Require and authorize a persisted canonical Session. |
| P0 | Cancel, restore, and stale reconciliation change database state only. | Define Kernel calls, confirmation, retries, and reconciliation obligations. |
| P0 | Process-local semaphore is the only active concurrency gate. | Integrate reviewed distributed admission/quota/capacity without treating local semaphore as proof. |
| P0 | Lease/fencing fields do not protect active completion. | Define execution-attempt ownership and stale-write rejection end to end. |
| P0 | Outbox is not transactionally produced and audit is not in the aggregate transaction. | Approve and implement atomic write/outbox/audit pattern. |
| P0 | Task repository/query scope has incomplete organization-isolation evidence. | Complete all organization-leading SQL and negative tests. |
| P0 | Local inline auth/business audit uses process-memory stores. | Select durable local owner persistence and recovery policy. |
| P1 | App API operation counts disagree across accepted requirement, PRD, docs, checks, and current OpenAPI. | Reconcile authored source, generated SDKs, docs, and validators in one reviewed change. |

## Protected Decisions

- public field/resource/error names and Session/Task compatibility;
- App/Open/Backend API exposure and generated SDK ownership;
- PostgreSQL tables, columns, constraints, indexes, migration, backfill, and
  rollback;
- tenant/organization authorization and audit semantics;
- Kernel port/RPC and raw lease credential handling;
- local persistence, data residency, backup/restore/purge, and offline identity;
- cloud Workspace/checkpoint storage authority;
- production topology, SLO, rollout, and release evidence.

## Required Approval

- Product owner approves behavior and non-goals.
- Agents domain owner approves target inheritance and binding split.
- API/SDK owner approves public surface and generation plan.
- Database owner approves organization isolation and migration/rollback.
- Kernel owner approves the sole placement port and version/error contract.
- Security/privacy approve trust, credentials, residency, retention, and audit.
- SRE/release approve durability, capacity, observability, rollout, and evidence.

## Decision

No-Go for API, SDK, database, Kernel-port, production topology, or commercial
claim changes. Candidate docs and contract tests may proceed. Implementation
starts only after REQ is ready/accepted, ADR is accepted, this review is
Approved, Kernel PRD-05 is accepted, and required Sandbox contracts authorize
runtime implementation.
