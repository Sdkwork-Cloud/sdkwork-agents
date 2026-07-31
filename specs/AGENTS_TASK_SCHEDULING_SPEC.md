# SDKWork Agents Task Scheduling Specification

- Version: `1.0.0`
- Status: active
- Owner: `agents-platform`
- Requirement: `docs/product/requirements/REQ-2026-0731-agent-task-scheduling.md`
- Decision: `docs/architecture/decisions/ADR-20260731-agent-task-scheduling.md`
- Machine contract: `specs/agent-task-scheduling.contract.json`

## 1. Authority And Boundary

Agents owns Task scheduling definitions, logical Runs, delivery Attempts, and
their relation to the canonical Session/Turn aggregate. Kernel owns agent
runtime mechanisms. PostgreSQL owns all correctness-relevant scheduling state.
Redis, event buses, timers, and deployment orchestrators are optional adapters.

## 2. Aggregate Vocabulary

| Type | Identity | Responsibility |
| --- | --- | --- |
| `AgentTask` | `taskId` | Versioned schedule definition and bounded execution policy. |
| `AgentTaskRun` | `runId` | One scheduled/manual/business-retry occurrence and one logical Turn identity. |
| `AgentTaskRunAttempt` | `attemptId` | One leased infrastructure delivery with fencing and diagnostic outcome. |

Task status is `active`, `paused`, `completed`, or `cancelled`. Run status is
`pending`, `claimed`, `running`, `succeeded`, `failed`, `cancelled`,
`reconciling`, or `dead_letter`. Attempt status is `claimed`, `running`,
`succeeded`, `failed`, `lease_expired`, or `cancelled`.

## 3. Schedule Contract

- One-time: `scheduledAt` is required; `cronExpression` is absent.
- Cron: `cronExpression` and `timezone` are required; `scheduledAt` is absent.
- Cron expressions contain seconds, minutes, hours, day-of-month, month, and
  day-of-week. Parsing and next-occurrence calculation use maintained cron and
  IANA timezone libraries.
- `timezone` defaults to `UTC` only when omitted by a trusted internal caller;
  public create commands require it for cron.
- `startsAt` is inclusive and `endsAt` is exclusive.
- Misfire is `skip`, `fire_once`, or bounded `catch_up`.
- Overlap is `skip` or `queue`.
- `maxConcurrentRuns` is 1 to 32; `maxCatchUpRuns` is 1 to 100;
  `maxAttempts` is 1 to 20; timeout is 1 to 86400 seconds.

## 4. Materialization

A materializer transaction selects at most its configured batch size from the
indexed predicate `status=active AND next_fire_at<=now()`, ordered by
`next_fire_at,id`, using `FOR UPDATE SKIP LOCKED`. For each selected Task it:

1. validates status, generation, schedule bounds, and misfire policy;
2. inserts zero or more uniquely scoped Runs;
3. computes and stores the next schedule cursor or terminal Task status;
4. increments Task version without changing generation;
5. writes sanitized outbox facts;
6. commits before any worker or provider call.

Unique conflicts mean the occurrence already exists and are not errors.

## 5. Claim, Lease, And Fencing

Eligible Runs are pending or retryable with `availableAt<=now()`, ordered by
priority, scheduled time, and id. Claiming uses `FOR UPDATE SKIP LOCKED`, a
bounded lease, a random raw lease token returned once, its SHA-256 hash in the
database, and a monotonic fencing token. Claim creates an Attempt atomically.

Heartbeat and completion require tenant, organization, Run, Attempt, lease
token hash, fencing token, unexpired lease, and expected state. A stale worker
cannot mutate a Run after lease recovery or cancellation.

## 6. Turn Execution

The worker loads the Task and referenced persisted Session, rechecks scope and
generation, and reserves/reuses a Turn from the Run idempotency key and payload
hash. Provider execution happens through the canonical Turn runtime facade.
Attempt retries never create a second Turn. Business retry creates a new Run
with `retryOfRunId` and a new Turn idempotency key.

## 7. Failure And Recovery

- Retryable failure uses capped exponential backoff with deterministic jitter.
- Terminal validation, authorization, or policy failures do not retry.
- Provider timeout with unknown outcome enters `reconciling`.
- Exhausted retry budget enters `dead_letter` and emits an operational event.
- Expired claims are recovered in bounded batches; the previous Attempt becomes
  `lease_expired` and a later claim receives a higher fencing token.
- Task cancellation prevents new materialization. Run cancellation follows the
  real Turn/runtime cancellation contract.

## 8. Security And Privacy

All API scope is derived from trusted request context. Public payloads never
accept tenant, organization, lease, fence, generation, counters, next fire,
worker, Attempt, or terminal result fields. Lease tokens are random, bounded,
returned once, hashed with `sdkwork-utils-rust`, and never logged. Prompt and
output content remain in the Session aggregate; audit/outbox contains ids,
states, classifications, and bounded timings only.

## 9. Query And Retention

Task and Run lists use PostgreSQL filtering, stable ordering, and bounded
pagination. Operational indexes support due Tasks, eligible Runs, active
leases, reconciliation, per-Task active Run counts, and retention. Terminal
Attempts may have shorter retention than Runs; aggregate audit retention is
never weakened by deleting delivery diagnostics.

## 10. Observability

Required metrics include materialization lag and count, eligible backlog and
age, claim latency, active leases, lease expiry, fencing rejection, Run latency
and outcome, retry/dead-letter count, reconciliation age, and outbox lag.
Labels are limited to deployment, status, trigger, failure class, and bounded
provider category. Tenant, user, Task, Run, prompt, and error text are excluded.

## 11. Verification

The release gate includes focused Rust tests, PostgreSQL live contention and
restart tests, API/SDK generation checks, database validation, pagination,
security, topology, deployment, documentation, and full repository checks.
