# Monitoring And Alerting

Operational observability for SDKWork Agents staging and production.

## 1. Endpoints

The gateway and Task Worker are separate processes and must be scraped and
alerted independently.

| Process | Endpoint | Purpose |
| --- | --- | --- |
| Gateway | `GET /healthz` and `GET /livez` | process liveness and Kubernetes alias |
| Gateway | `GET /readyz` | database, schema and required-dependency readiness |
| Gateway | `GET /metrics` | framework Prometheus exposition |
| Gateway | `GET /metrics/agents` | Agents domain Prometheus exposition |
| Task Worker | `GET /healthz` and `GET /livez` | Worker process liveness and alias |
| Task Worker | `GET /readyz` | scheduling acceptance; false while draining or when database readiness fails |
| Task Worker | `GET /metrics` | Worker HTTP and scheduler Prometheus exposition |

Scrape through the approved private operations network. Do not expose metrics
publicly without platform authentication.

## 2. Required Signals

| Signal | Dimensions | Use |
| --- | --- | --- |
| HTTP request count/latency | surface, route template, method, status | SLO and regression detection |
| Turn lifecycle count/latency | mode, status, provider class | execution health |
| Provider failures/timeouts | provider class, normalized error | dependency health |
| Project Session synchronization | provider class, outcome, normalized issue code | explicit import health and partial-result rate |
| Database pool | total, active, idle, wait, timeout | saturation and leak detection |
| Interaction backlog | kind, age bucket | blocked human workflows |
| Outbox backlog | status, age bucket, attempt bucket | delivery reliability |
| Idempotency conflicts | operation/resource | client correctness and abuse detection |

Never use raw tenant/user IDs, content, prompts, tool arguments, credentials or
claim tokens as metric labels.

### 2.1 Task Worker Metrics

| Metric | Type | Meaning |
| --- | --- | --- |
| `sdkwork_agents_task_worker_materialized_total` | counter | logical Runs materialized |
| `sdkwork_agents_task_worker_claimed_total` | counter | delivery Attempts claimed |
| `sdkwork_agents_task_worker_recovered_leases_total` | counter | expired Run leases recovered |
| `sdkwork_agents_task_worker_retries_total` | counter | infrastructure retries of the same Run |
| `sdkwork_agents_task_worker_fencing_rejections_total` | counter | stale owner/fence heartbeat rejection |
| `sdkwork_agents_task_worker_reconciliation_examined_total` | counter | reconciling Runs examined |
| `sdkwork_agents_task_worker_reconciliation_reconciled_total` | counter | Runs moved to a terminal result |
| `sdkwork_agents_task_worker_reconciliation_pending_total` | counter | Runs whose canonical Turn is not terminal |
| `sdkwork_agents_task_worker_reconciliation_conflicts_total` | counter | reconciliation compare-and-set conflicts |
| `sdkwork_agents_task_worker_heartbeats_total` | counter | lease heartbeat operations |
| `sdkwork_agents_task_worker_heartbeat_failures_total` | counter | failed lease heartbeats |
| `sdkwork_agents_task_worker_executions_total{outcome}` | counter | bounded outcomes: `succeeded`, `failed`, `reconciling`, `dead_letter`, `cancelled` |
| `sdkwork_agents_task_worker_operation_errors_total` | counter | scheduler repository errors |
| `sdkwork_agents_task_worker_forced_drain_total` | counter | shutdowns exceeding the drain timeout |
| `sdkwork_agents_task_worker_inflight` | gauge | current Run executions |
| `sdkwork_agents_task_worker_materialization_duration_seconds` | summary | due-Task materialization duration |
| `sdkwork_agents_task_worker_claim_duration_seconds` | summary | Run claim transaction duration |
| `sdkwork_agents_task_worker_execution_duration_seconds` | summary | delivery Attempt execution duration |
| `sdkwork_agents_task_run_latency_seconds` | summary | scheduled occurrence to terminal result latency |
| `sdkwork_agents_task_due` | gauge | active Tasks due for materialization |
| `sdkwork_agents_task_materialization_lag_seconds` | gauge | age of the oldest due occurrence |
| `sdkwork_agents_task_run_eligible` | gauge | pending Runs eligible for claim |
| `sdkwork_agents_task_run_eligible_oldest_age_seconds` | gauge | age of the oldest eligible Run |
| `sdkwork_agents_task_run_active_leases` | gauge | unexpired claimed/running leases |
| `sdkwork_agents_task_run_reconciling` | gauge | Runs awaiting canonical outcome reconciliation |
| `sdkwork_agents_task_run_reconciliation_oldest_age_seconds` | gauge | age of the oldest reconciling Run |
| `sdkwork_agents_outbox_pending` | gauge | undelivered transactional outbox facts |
| `sdkwork_agents_outbox_oldest_age_seconds` | gauge | age of the oldest undelivered outbox fact |

The five cardinality snapshots (`task_due`, `task_run_eligible`,
`task_run_active_leases`, `task_run_reconciling`, and `outbox_pending`) saturate
at 100,000 to keep each PostgreSQL snapshot bounded. The default snapshot
interval is 60 seconds and is independently controlled by
`SDKWORK_AGENTS_TASK_METRICS_SNAPSHOT_INTERVAL_SECONDS`.

## 3. Alerts

| Alert | Condition | First action |
| --- | --- | --- |
| High 5xx rate | sustained elevated server-error ratio | correlate route and trace; inspect database/provider health |
| Auth rejection spike | abnormal 401/403 by surface | verify credential provider and gateway classification |
| Database saturation | high pool utilization or wait timeout | stop rollout; inspect slow queries and pool limits |
| Turn timeout spike | runtime timeout above baseline | inspect provider health and binding rollout |
| Synchronization failure spike | failed outcome or elevated bounded issue counts | inspect runtime binding, provider collector, server-derived working directory and provider health by trace |
| Interaction age | pending age exceeds product SLO | verify resolver clients and claim flow |
| Materialization lag | `sdkwork_agents_task_materialization_lag_seconds` exceeds schedule SLO | inspect database health, Worker readiness and materialization duration |
| Eligible Run age | `sdkwork_agents_task_run_eligible_oldest_age_seconds` grows across snapshots | inspect Worker capacity, tenant limits, claim errors and claim latency |
| Claim latency | claim summary latency exceeds database transaction budget | inspect PostgreSQL locks, indexes and pool saturation |
| Lease recovery | recovered-lease rate rises or active leases remain while eligible age grows | inspect Worker restarts, heartbeat failures, drain timeout and provider latency |
| Fencing rejection | fencing rejection counter increases unexpectedly | correlate Worker restart/lease recovery; reject stale completions |
| Retry/dead letter | retry rate or `outcome="dead_letter"` rate exceeds policy | inspect normalized provider/infrastructure failure class and retry policy |
| Reconciliation age | reconciling count or oldest age exceeds outcome SLO | inspect canonical Turn state and provider outcome lookup |
| Outbox backlog | pending count or oldest age exceeds the approved bound | block event-dependent release and verify transactional fact creation/platform relay readiness |
| No traffic | expected traffic absent | ingress, discovery, base URL and release routing |

External outbox delivery is not active until the platform provides the approved
publisher SPI. The outbox metrics currently prove transactional fact backlog,
not successful downstream publication; do not operate or document a local
Kafka/raw HTTP relay as a workaround.

## 4. Logs And Traces

Structured records include server-owned `traceId`, route template, method,
surface, normalized status, duration and bounded resource type/id. Sensitive
content is redacted. Use the response `traceId` to correlate client, gateway,
service, database and provider spans.

For `agents.projectSessions.synchronize`, record bounded aggregate issue codes
and counts, never provider payloads, local paths, Session content, or client
device paths. A `50001` response indicates that the command failed before a
typed partial result could be returned. Correlate its `traceId` across runtime
binding resolution, provider collector execution, server-derived working
directory validation, persistence, and gateway spans. Keep ordinary
Project/Session inventory reads independent so operators can distinguish an
import failure from a list availability incident.

Audit events are business evidence and are not replaced by application logs.

## 5. Release Observation

During rollout compare candidate and baseline:

- p50/p95/p99 API and Turn latency;
- error, timeout and cancellation rate;
- database pool wait and slow query count;
- idempotency conflict rate;
- Interaction, scheduling, reconciliation and outbox age;
- retry, dead-letter, recovered-lease and fencing-rejection rate;
- memory/CPU and restart count.

Rollback according to [incident-rollback.md](./incident-rollback.md) when a
release breaches the approved threshold.
