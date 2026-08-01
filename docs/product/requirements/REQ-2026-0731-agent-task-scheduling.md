# REQ-2026-0731 Durable Agent Task Scheduling

- Owner: `agents-platform`
- Status: accepted
- Priority: P0
- Source: product
- Updated: `2026-07-31`
- Specs: `REQUIREMENTS_SPEC.md`, `AGENT_KERNEL_SPEC.md`, `API_SPEC.md`, `DATABASE_SPEC.md`, `SECURITY_SPEC.md`, `EVENT_SPEC.md`, `PERFORMANCE_SPEC.md`, `TEST_SPEC.md`

## Problem

SDKWork Agents must schedule high-volume tenant work without process-local
timers, duplicate logical occurrences, stale-worker completion, or a parallel
execution model. Scheduling must preserve the canonical Session/Turn aggregate
while remaining deterministic across multiple nodes, restarts, timezones, and
provider outcomes that cannot be confirmed immediately.

## Goals

- Make `AgentTask` a durable one-time or recurring schedule definition.
- Represent each logical occurrence as `AgentTaskRun` and every infrastructure
  delivery as `AgentTaskRunAttempt`.
- Require every Task to reference one persisted, authorized `AgentSession`.
- Execute a Run through one idempotent `AgentTurn`; infrastructure retries
  reuse the same Run and Turn.
- Use PostgreSQL as the sole correctness authority for due materialization,
  claims, leases, fencing, retries, cancellation, and reconciliation.
- Support horizontal scheduler and worker scaling without duplicate logical
  occurrences or stale-worker completion.
- Provide bounded misfire, overlap, retry, timeout, retention, quota, audit,
  metric, trace, and operational controls.
- Keep standalone and cloud behavior equivalent. A cloud event bus may deliver
  outbox events, but it is never the scheduling authority.

## Non-Goals

- Guaranteeing an external provider side effect exactly once when that
  provider has no idempotency contract.
- Using Redis, an in-process timer, a message broker, or Kubernetes CronJob as
  the source of schedule truth.
- Persisting provider credentials, raw lease tokens, prompts, model output, or
  full error payloads in audit, outbox, metrics, or logs.
- Duplicating Kernel runtime or provider implementation in Agents.

## Functional Requirements

1. A Task stores a required `sessionId`, schedule kind, IANA timezone, schedule
   policy, retry policy, timeout, lifecycle status, generation, and next fire
   time. Cron parsing uses a maintained library.
2. A one-time Task has one fire time. A recurring Task uses a six-field cron
   expression with seconds and an explicit IANA timezone. DST behavior follows
   the timezone database and is deterministic.
3. Creating or changing a schedule computes `nextFireAt` on the server. Client
   supplied derived fire times, lease values, fencing values, counters, Run
   state, or worker identity are rejected.
4. Due materialization locks bounded Task rows with `FOR UPDATE SKIP LOCKED`,
   inserts one Run per occurrence, advances the Task cursor, and writes an
   outbox fact in the same transaction.
5. `(tenant, organization, task, generation, scheduledFor)` uniquely
   identifies a scheduled occurrence. Repeated scans are harmless.
6. Misfire policy is `skip`, `fire_once`, or bounded `catch_up`. Catch-up never
   exceeds the Task limit or the worker batch limit.
7. Overlap policy is `skip` or `queue`. Per-Task and per-tenant concurrency is
   enforced before execution; no unbounded in-memory queue is permitted.
8. A worker claims an eligible Run with a bounded lease, a random token stored
   only as a hash, and a monotonically increasing fencing token. Heartbeat,
   completion, retry, cancellation, and lease recovery compare ownership and
   fencing.
9. One Run owns one idempotency key, canonical payload hash, and Turn id.
   Retrying transport or infrastructure delivery reuses them. A user-requested
   business retry creates a new Run linked by `retryOfRunId`.
10. Run execution loads and authorizes the persisted Session, invokes the
    canonical `execute_turn` use case, and records the resulting Turn id. A
    transient or synthesized Session is forbidden.
11. Failures are classified as retryable, terminal, cancelled, timed out,
    lease lost, fencing conflict, policy denied, capacity limited, dependency
    degraded, or outcome unknown. Unknown provider outcomes enter
    reconciliation instead of blind retry.
12. Task changes increment `generation`. Materialization and completion reject
    stale generations. Pausing prevents future materialization but does not
    silently cancel an already-running Run.
13. Cancelling a Task prevents future Runs. Cancelling a Run routes through the
    real Turn/runtime cancellation path and records confirmed or reconciling
    delivery.
14. App, Open, and Backend APIs expose Task schedules and Run history through
    generated SDKs. List filtering and pagination execute in PostgreSQL.
15. All mutations enforce trusted tenant, organization, owner/member, agent,
    Session, Task, and Run scope and write sanitized audit/outbox facts.

## Non-Functional Requirements

| Area | Required outcome |
| --- | --- |
| Scale | Horizontal schedulers and workers; bounded batch claims; indexed due scans; no full-table polling. |
| Correctness | Atomic occurrence materialization, idempotent Run/Turn identity, lease fencing, generation checks, and reconciliation. |
| Availability | Scheduler and worker restarts recover expired leases without losing or duplicating logical occurrences. |
| Security | Server-owned control fields, hashed lease tokens, least privilege, tenant isolation, bounded payloads, sanitized diagnostics. |
| Performance | Due-scan p95 under 100 ms at the approved batch size; claim transaction p95 under 50 ms; no provider call inside a database transaction. |
| Operability | Backlog age, due lag, claim latency, lease loss, retries, dead letters, reconciliation age, and terminal outcomes are observable with bounded labels. |
| Extensibility | Trigger delivery and acceleration are ports; PostgreSQL semantics and public domain contracts remain unchanged. |

## Acceptance Evidence

- Database contract, DDL, registry, manifest, migration policy, and repository
  SQL agree on the complete 23-table inventory.
- Domain and scheduler tests cover cron/timezone/DST, misfire, overlap,
  generation, occurrence uniqueness, leases, fencing, retries, cancellation,
  reconciliation, restart, and tenant/organization denial.
- Task execution creates or reuses a persisted Turn inside the referenced
  Session; code and contract scans find no Task Session stub.
- Authored OpenAPI, generated SDKs, route inventory, product docs, and API docs
  agree.
- PostgreSQL live contention tests prove multiple schedulers and workers do not
  duplicate occurrences or accept stale completions.
- Production security, deployment, observability, database, topology, API,
  documentation, and full repository verification gates pass.
- Transactional outbox persistence is verified independently from external
  publication. Any release that requires cross-service event delivery remains
  gated on a platform-owned publisher SPI and its end-to-end delivery evidence.

## Traceability

- [ADR-20260731](../../architecture/decisions/ADR-20260731-agent-task-scheduling.md)
- [REVIEW-20260731](../../engineering/reviews/REVIEW-20260731-agent-task-scheduling.md)
- [AGENTS_TASK_SCHEDULING_SPEC.md](../../../specs/AGENTS_TASK_SCHEDULING_SPEC.md)
- [agent-task-scheduling.contract.json](../../../specs/agent-task-scheduling.contract.json)
