# Incident Response And Rollback

Use this runbook when sdkwork-agents production or staging shows elevated errors, data inconsistency, or failed cutover.

## 1. Triage signals

| Signal | Check |
| --- | --- |
| HTTP 5xx spike | Gateway logs, `agents.managed_store.request` trace lines |
| Auth failures | IAM session validity, `SDKWORK_AGENTS_DEV_AUTH_BYPASS` must be `false` |
| Postgres errors | `SDKWORK_AGENTS_STORE_DATABASE_*` connectivity, migration status |
| Client failures | App SDK base URLs, `pnpm workflow:build-agents-app-sdk` drift |

Prometheus (when scraped):

- `sdkwork_agents_requests_per_second`
- `sdkwork_agents_http_errors_total`
- `sdkwork_agents_http_requests_total`

Endpoint: `/metrics/agents` on the standalone gateway assembly.

## 2. Immediate containment

1. Scale down traffic at ingress or disable new deployments.
2. Confirm no pod is running with dev auth bypass or in-memory managed store in production-like profiles.
3. Capture `traceId` from failing API responses (`x-sdkwork-trace-id`).

## 3. Rollback procedure

### Application binary

1. Identify last known-good release tag from CI packaging workflow.
2. Redeploy previous container image or tar.gz artifact from `sdkwork.workflow.json` channel.
3. Run health check: `GET /health` and one authenticated `GET /app/v3/api/ai/agents` smoke call.

### Database

1. **Do not** roll back schema without a planned migration reversal.
2. Run `pnpm db:status` and `pnpm db:drift:check` from repository root against target env.
3. If migration caused regression, restore Postgres snapshot per platform runbook, then redeploy matching application version.

### Client surfaces

1. Redeploy previous PC/H5 static build if API contract changed.
2. Rebuild mini-program runtime: `pnpm --filter @sdkwork/agents-mini-program build`.

## 4. Recovery verification

From repository root:

```powershell
pnpm verify
pnpm topology:validate
pnpm db:validate
```

Staging cutover checklist: [pre-launch-verification.md](./pre-launch-verification.md).

## 5. Post-incident

1. Record root cause in `docs/architecture/decisions/` as ADR when behavior or contract changes.
2. Add regression test (HTTP contract or client contract) before closing the incident.
