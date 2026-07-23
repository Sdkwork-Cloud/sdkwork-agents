# Monitoring And Alerting

Operational observability for SDKWork Agents staging and production.

## 1. Endpoints

| Endpoint | Purpose |
| --- | --- |
| `GET /health` | process and dependency liveness/readiness |
| `GET /metrics/agents` | Prometheus exposition for Agents runtime |

Scrape through the approved private operations network. Do not expose metrics
publicly without platform authentication.

## 2. Required Signals

| Signal | Dimensions | Use |
| --- | --- | --- |
| HTTP request count/latency | surface, route template, method, status | SLO and regression detection |
| Turn lifecycle count/latency | mode, status, provider class | execution health |
| Provider failures/timeouts | provider class, normalized error | dependency health |
| Database pool | total, active, idle, wait, timeout | saturation and leak detection |
| Interaction backlog | kind, age bucket | blocked human workflows |
| Outbox backlog | status, age bucket, attempt bucket | delivery reliability |
| Idempotency conflicts | operation/resource | client correctness and abuse detection |

Never use raw tenant/user IDs, content, prompts, tool arguments, credentials or
claim tokens as metric labels.

## 3. Alerts

| Alert | Condition | First action |
| --- | --- | --- |
| High 5xx rate | sustained elevated server-error ratio | correlate route and trace; inspect database/provider health |
| Auth rejection spike | abnormal 401/403 by surface | verify credential provider and gateway classification |
| Database saturation | high pool utilization or wait timeout | stop rollout; inspect slow queries and pool limits |
| Turn timeout spike | runtime timeout above baseline | inspect provider health and binding rollout |
| Interaction age | pending age exceeds product SLO | verify resolver clients and claim flow |
| Outbox backlog | oldest pending age exceeds delivery SLO | inspect dispatcher and retry policy |
| No traffic | expected traffic absent | ingress, discovery, base URL and release routing |

## 4. Logs And Traces

Structured records include server-owned `traceId`, route template, method,
surface, normalized status, duration and bounded resource type/id. Sensitive
content is redacted. Use the response `traceId` to correlate client, gateway,
service, database and provider spans.

Audit events are business evidence and are not replaced by application logs.

## 5. Release Observation

During rollout compare candidate and baseline:

- p50/p95/p99 API and Turn latency;
- error, timeout and cancellation rate;
- database pool wait and slow query count;
- idempotency conflict rate;
- Interaction and outbox age;
- memory/CPU and restart count.

Rollback according to [incident-rollback.md](./incident-rollback.md) when a
release breaches the approved threshold.
