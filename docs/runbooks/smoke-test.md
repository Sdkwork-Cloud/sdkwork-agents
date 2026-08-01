# Live Smoke Test

Use after deploying the candidate artifact. Contract tests are necessary but do
not replace this runtime evidence.

## 1. Prerequisites

1. The selected standalone or cloud profile is running with PostgreSQL.
2. App credentials, operator credentials and an Open API key are available from
   approved secret/session infrastructure.
3. At least one active managed agent has a conformance-tested provider binding.
4. Client base URLs point to the exact surface prefixes.
5. `SDKWORK_AGENTS_DEV_AUTH_BYPASS=true` must be `false` in staging/production.

## 2. Infrastructure

| Step | Action | Pass criterion |
| ---: | --- | --- |
| 1 | Gateway `GET /healthz` and `GET /livez` | HTTP 200 with liveness status `ok` |
| 2 | Gateway `GET /readyz` | HTTP 200 after database/schema and required dependencies pass |
| 3 | Gateway `GET /metrics` and `GET /metrics/agents` | framework and Agents domain Prometheus exposition are present |
| 4 | Worker `GET /healthz` and `GET /livez` on the private operations service | HTTP 200 while the process is alive |
| 5 | Worker `GET /readyz` | HTTP 200 only while scheduling work is accepted and database readiness passes |
| 6 | Worker `GET /metrics` | scheduler counters, durations and bounded backlog gauges are present |
| 7 | Inspect startup logs | PostgreSQL and Snowflake node lease acquired; no auth bypass, raw lease token or secret output |

Gateway-only helper:

```powershell
pnpm smoke:live
```

## 3. App API Execution

| Step | Action | Pass criterion |
| ---: | --- | --- |
| 1 | Sign in through appbase IAM | App SDK has current dual-token session |
| 2 | List Projects with Workspace scope, `q`, and `name_exact` | Offset `PageInfo`, bounded search results, and case-insensitive exact resolution are correct |
| 3 | List Project Sessions and retrieve one by Project/Session identity | Read-only inventory is returned without invoking provider synchronization |
| 4 | Explicitly import or re-import a folder and call `projectSessions.synchronize` | Response reports synchronized/skipped/failed counts and bounded aggregate issues; one malformed provider row does not discard valid rows |
| 5 | List/retrieve a managed Agent | `code: 0`, typed resource and `traceId` |
| 6 | Create a Session | Required kind/surface/idempotency fields accepted; Session id returned |
| 7 | Retry Session creation | Same key/hash returns the same logical result |
| 8 | Execute a Turn as JSON | Response contains Session, Turn and ordered Session Items; `runtimeMode` is `agents-runtime-facade`, never a contract stub |
| 9 | Execute a Turn as SSE | Delta events followed by one typed completion event |
| 10 | List newest Session Items with `sort=-sequence`, then continue with `nextCursor` | Cursor is opaque and progressing; pages are stable and client presentation is chronological |
| 11 | Create and claim an Interaction | One claim succeeds; competing claim fails safely |
| 12 | Resolve the Interaction | Version and claim token enforced |
| 13 | Cancel an eligible Turn | Lifecycle becomes cancelled without duplicate execution |

## 4. Security Surfaces

| Check | Expected result |
| --- | --- |
| App call without valid dual tokens | 401/403 Problem Detail |
| Backend call with app-only identity | denied |
| Open call without `X-API-Key` | denied |
| Open call with app credential headers only | denied |
| Caller-supplied scope selector | rejected |
| Cross-tenant resource id | not disclosed and denied |

## 5. Scheduled Task Execution

1. Create a persisted Session, then create a Session-bound one-time Task a few
   minutes ahead with an IANA timezone. Create a six-field cron Task with the
   same Session and an explicit DST/misfire/overlap policy.
2. Observe `nextFireAt`, then wait for the Worker materialization cycle. List
   Task Runs and verify exactly one Run for the generation and scheduled instant.
3. List the Run Attempts and verify claim metadata, monotonically increasing
   fencing token, bounded lease timestamps and terminal result. Confirm the
   canonical Turn belongs to the Task Session.
4. Execute the Task manually twice with the same idempotency key and payload.
   Both responses must identify one logical Run and Turn. Reusing the key with
   a different payload must return a conflict.
5. Verify infrastructure retry reuses the Run and Turn and appends an Attempt.
   Verify a governed business retry creates a linked Run and Turn.
6. Confirm metrics advance for materialization, claim, execution and Run
   latency without tenant, user, Task, Run, worker, token or error-text labels.

## 6. Recovery And Fencing

1. Submit a Turn and interrupt the client after request dispatch.
2. Reconcile by Session and idempotency key.
3. Confirm a completed Turn is returned without another provider invocation.
4. Restart the service and retrieve the same Session, Turn and Items.
5. Start a controlled Task Run, terminate or scale down its Worker after claim,
   wait for lease expiry, then restore/scale the Worker. Confirm recovery creates
   a new Attempt with a greater fencing token and does not duplicate the Run,
   Turn or scheduled occurrence.
6. Run the ignored live PostgreSQL contract
   `postgres_expired_lease_recovery_fences_the_previous_worker` in the isolated
   `sdkwork_ai_test_*` environment. Confirm the old heartbeat/completion is
   rejected and no raw lease token appears in logs, metrics, audit or API data.
7. For an intentionally unknown provider outcome, verify the Run enters
   `reconciling` and only the canonical Turn/provider result can terminalize it.
8. Confirm lifecycle mutations create audit and outbox facts atomically without
   duplicate business facts. Do not claim external outbox delivery: that remains
   gated until the approved platform publisher SPI is available and integrated.

## 7. Failure Triage

| Symptom | Check |
| --- | --- |
| 401/403 | credential mode, token/key provider, clock and request context |
| 404 | surface base URL and route assembly |
| 409 | idempotency payload hash or optimistic version |
| 5xx on Session/Turn | PostgreSQL status, provider binding, runtime facade and trace |
| 50001 on Project Session synchronization | Preserve `traceId`; verify the Project runtime binding resolves to the intended host, the provider collector is registered and healthy, and the server-derived working directory is valid and accessible. Do not request or log a client device path. Confirm read-only Project/Session refresh still succeeds before retrying explicit import. |
| Missing SSE completion | gateway buffering, timeout, runtime error and HTTP contract |
| Item order or continuation mismatch | Session sequence constraint, requested sort, opaque cursor binding, and repository keyset query order |
| Due Task not materialized | Worker readiness, due/lag gauges, timezone/DST policy, generation and database locks |
| Run remains eligible | claim latency/errors, Worker capacity, tenant concurrency and lease recovery |
| Stale completion accepted | stop rollout; preserve Attempt/fencing evidence and run the live PostgreSQL stale-fence contract |

See [monitoring.md](./monitoring.md) and
[incident-rollback.md](./incident-rollback.md).
