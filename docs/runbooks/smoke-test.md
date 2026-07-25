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
| 1 | `GET /healthz` | HTTP 200 with liveness status `ok` |
| 2 | `GET /livez` | HTTP 200 with the same liveness status as `/healthz` |
| 3 | `GET /readyz` | HTTP 200 with readiness status `ready` after required dependencies pass |
| 4 | `GET /metrics` | HTTP 200 with framework Prometheus exposition |
| 5 | `GET /metrics/agents` | HTTP 200 with `sdkwork_agents_` domain metrics |
| 6 | Inspect startup logs | PostgreSQL connected; no development auth bypass or secret output |

Gateway-only helper:

```powershell
pnpm smoke:live
```

## 3. App API Execution

| Step | Action | Pass criterion |
| ---: | --- | --- |
| 1 | Sign in through appbase IAM | App SDK has current dual-token session |
| 2 | List/retrieve a managed Agent | `code: 0`, typed resource and `traceId` |
| 3 | Create a Session | Required kind/surface/idempotency fields accepted; Session id returned |
| 4 | Retry Session creation | Same key/hash returns the same logical result |
| 5 | Execute a Turn as JSON | Response contains Session, Turn and ordered Session Items; `runtimeMode` is `agents-runtime-facade`, never a contract stub |
| 6 | Execute a Turn as SSE | Delta events followed by one typed completion event |
| 7 | List Session Items | Stable ascending sequence with `PageInfo` |
| 8 | Create and claim an Interaction | One claim succeeds; competing claim fails safely |
| 9 | Resolve the Interaction | Version and claim token enforced |
| 10 | Cancel an eligible Turn | Lifecycle becomes cancelled without duplicate execution |

## 4. Security Surfaces

| Check | Expected result |
| --- | --- |
| App call without valid dual tokens | 401/403 Problem Detail |
| Backend call with app-only identity | denied |
| Open call without `X-API-Key` | denied |
| Open call with app credential headers only | denied |
| Caller-supplied scope selector | rejected |
| Cross-tenant resource id | not disclosed and denied |

## 5. Recovery

1. Submit a Turn and interrupt the client after request dispatch.
2. Reconcile by Session and idempotency key.
3. Confirm a completed Turn is returned without another provider invocation.
4. Restart the service and retrieve the same Session, Turn and Items.
5. Confirm outbox and audit processing resumes without duplicate business facts.

## 6. Failure Triage

| Symptom | Check |
| --- | --- |
| 401/403 | credential mode, token/key provider, clock and request context |
| 404 | surface base URL and route assembly |
| 409 | idempotency payload hash or optimistic version |
| 5xx on Session/Turn | PostgreSQL status, provider binding, runtime facade and trace |
| Missing SSE completion | gateway buffering, timeout, runtime error and HTTP contract |
| Item order mismatch | Session sequence constraint and repository query order |

See [monitoring.md](./monitoring.md) and
[incident-rollback.md](./incident-rollback.md).
