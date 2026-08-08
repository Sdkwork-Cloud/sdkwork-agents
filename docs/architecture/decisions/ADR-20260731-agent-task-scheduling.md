# ADR-20260731 PostgreSQL-Authoritative Agent Task Scheduling

- Status: accepted
- Date: `2026-07-31`
- Owner: `agents-platform`
- Requirement: [REQ-2026-0731](../../product/requirements/REQ-2026-0731-agent-task-scheduling.md)

## Context

SDKWork Agents needs durable, high-volume cron scheduling across standalone and
cloud deployments. A schedule definition, logical occurrence, infrastructure
delivery, and business Turn have different lifecycle and concurrency
boundaries. Process timers, Redis sorted sets, and broker delivery alone cannot
atomically establish an occurrence and advance its schedule.

## Decision

Use three durable concepts:

```text
AgentTask (schedule definition)
  -> AgentTaskRun (one logical occurrence or business retry)
    -> AgentTaskRunAttempt (one infrastructure claim/delivery)
      -> AgentTurn (one idempotent business execution in a real AgentSession)
```

PostgreSQL is the only scheduling and execution-state authority. Schedulers
materialize due occurrences in short transactions using indexed predicates and
`FOR UPDATE SKIP LOCKED`. Workers claim Runs with leases and fencing. Provider
calls happen outside transactions. Completion is a compare-and-set operation
on Run generation, lease ownership, fencing token, and non-terminal state.

### Schedule Semantics

- `scheduleKind=one_time` requires `scheduledAt` and forbids cron.
- `scheduleKind=cron` requires a six-field cron expression and IANA timezone.
- Task changes increment `generation` and recompute `nextFireAt`.
- Occurrence uniqueness is scoped by Task, generation, and scheduled instant.
- Misfire and overlap policies are explicit and bounded.
- One-time schedules become completed after their occurrence is materialized;
  recurring schedules remain active until paused, cancelled, or expired.

### Execution Semantics

Every Task references a persisted Session matching tenant, organization,
owner, and agent. A Run derives a deterministic idempotency key and payload
hash, reserves one Turn, and invokes the canonical Turn execution path.
Infrastructure retries reuse Run and Turn. A business retry creates a linked
Run so history and billing remain explicit.

### Delivery Topology

The current standalone and cloud runtime polls PostgreSQL directly. Outbox rows
are committed transactionally, but cross-service publication is enabled only
after the platform exposes an approved event-publisher SPI with delivery,
idempotency, and observability evidence. Agents must not add a local Kafka or
raw HTTP publisher. A future broker or Redis integration may accelerate wakeups
or rate limits, but it never owns occurrences, leases, or completion state.

### External Side Effects

Agents provides exactly-once logical occurrence and idempotent internal
execution. External exactly-once is claimed only when the provider accepts and
honors the Run idempotency key. Ambiguous outcomes move to reconciliation;
they are not blindly re-executed.

## Alternatives

### Redis Sorted Set As Schedule Authority

Rejected because schedule advancement, Run insertion, audit, and outbox cannot
be committed atomically with the relational aggregate.

### Broker Delayed Messages As Schedule Authority

Rejected because broker retention, redelivery, and reordering do not provide
the required schedule cursor, generation, or tenant query semantics.

### One Kubernetes CronJob Per User Task

Rejected because cardinality, lifecycle latency, tenancy, Run history, and
portable standalone behavior are unsuitable for product-scale schedules.

### Reuse AgentTurn As The Schedule Definition

Rejected because a recurring definition and an execution occurrence have
different lifecycle, concurrency, retention, retry, and audit boundaries.

## Consequences

- The greenfield PostgreSQL baseline expands to 26 tables, including the
  already-authored Turn input queue.
- The Task public contract changes before first release; generated SDKs and UI
  consumers must migrate together.
- A scheduler/worker runtime is required in production topology.
- Capacity planning must account for due-scan, Run backlog, lease churn,
  outbox, and retained history.
- Provider integrations need idempotency/reconciliation capability metadata.
- Cross-service outbox relay is a release gate for event-dependent consumers;
  transactional row creation alone is not delivery evidence.

## Verification

- Deterministic schedule tests include timezone and daylight-saving edges.
- PostgreSQL live contention tests run multiple materializers and claimers.
- Failure injection covers crash after claim, lease expiry, late completion,
  broker loss, provider timeout, duplicate delivery, and reconciliation.
- API/SDK, database, security, performance, deployment, and documentation
  validators are release gates.

## Related Decisions

This decision preserves the Session aggregate and Kernel runtime ownership
boundaries. Task scheduling composes those authorities rather than creating an
alternate provider execution path.
