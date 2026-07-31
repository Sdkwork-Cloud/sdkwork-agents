# REVIEW-20260731 Agent Task Scheduling

- Status: approved
- Outcome: Go
- Date: `2026-07-31`
- Owner: `agents-platform`
- Approval: repository owner authorized the best production design and greenfield contract cleanup
- Requirement: [REQ-2026-0731](../../product/requirements/REQ-2026-0731-agent-task-scheduling.md)
- Decision: [ADR-20260731](../../architecture/decisions/ADR-20260731-agent-task-scheduling.md)

## Findings And Closure

| Severity | Finding | Approved closure |
| --- | --- | --- |
| P0 | Task combines schedule and execution state. | Split Task, Run, and Attempt with separate lifecycle and retention. |
| P0 | Task execution uses a transient Session. | Require a persisted Session and execute one idempotent Turn per Run. |
| P0 | No distributed occurrence or ownership control exists. | PostgreSQL atomic materialization, leases, fencing, generation, and reconciliation. |
| P0 | Retry semantics can duplicate business execution. | Infrastructure retries reuse Run/Turn; business retries create linked Runs. |
| P0 | No cron/timezone/misfire/overlap contract exists. | Adopt the bounded policy contract in `AGENTS_TASK_SCHEDULING_SPEC.md`. |
| P0 | Database authorities disagree after Turn queue work. | Publish one 23-table contract version across schema, registry, manifest, DDL, and docs. |
| P1 | Broker/Redis roles are ambiguous. | Restrict both to acceleration/delivery; PostgreSQL remains correctness authority. |
| P1 | External exactly-once could be overstated. | Require provider idempotency or reconciliation and document the guarantee boundary. |

## Approved Contract

- Breaking pre-release Task API and PostgreSQL baseline changes are approved.
- Public schedule controls are limited to schedule, bounded policies, prompt,
  Session reference, and safe metadata.
- Lease, fencing, generation, derived timestamps, Run state, Attempt state, and
  worker identity are server-owned.
- App scope is owner constrained; Backend scope requires operator permissions;
  Open scope uses isolated API-key authorization.
- Prompt and model output remain in Session Items, not Run audit/outbox/logs.
- Standalone and cloud deployments use the same state machine and SQL
  correctness model.

## Release Conditions

- All acceptance evidence in REQ-2026-0731 is present.
- No Task Session stub, in-process schedule authority, raw HTTP SDK bypass, or
  generated-file hand edit remains.
- Production configuration fails closed when the managed PostgreSQL store or
  scheduler identity is unavailable.
- Rollout, pause, drain, lease recovery, backup/restore, and rollback runbooks
  are current and exercised.
