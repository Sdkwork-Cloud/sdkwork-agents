# Monitoring And Alerting

Operational observability for sdkwork-agents in staging and production.

## Health endpoints

| Endpoint | Purpose |
| --- | --- |
| `GET /health` | Liveness — process up |
| `GET /metrics/agents` | Prometheus text exposition for managed-agent HTTP |

Scrape `/metrics/agents` from the standalone gateway assembly or split API service behind your platform Prometheus stack.

## Key metrics

| Metric | Type | Use |
| --- | --- | --- |
| `sdkwork_agents_requests_per_second` | gauge | Traffic rate |
| `sdkwork_agents_http_requests_total` | counter | Request volume by route/status |
| `sdkwork_agents_http_errors_total` | counter | 4xx/5xx spike detection |

Correlate spikes with application logs (`agents.managed_store.request`) and client `traceId` from API responses (`x-sdkwork-trace-id`).

## Recommended alerts

| Alert | Condition | Action |
| --- | --- | --- |
| High 5xx rate | `rate(sdkwork_agents_http_errors_total{status=~"5.."}[5m])` elevated | See [incident-rollback.md](./incident-rollback.md) |
| Auth failures | IAM 401/403 on `/app/v3/api` | Verify session tokens, dev bypass off |
| Postgres connectivity | Managed store errors in logs | Check `SDKWORK_AGENTS_STORE_DATABASE_*`, run `pnpm db:status` |
| Zero traffic | RPS near zero during business hours | Ingress, gateway, or client base URL misconfiguration |

## Client-side signals

| Surface | Check |
| --- | --- |
| PC / H5 | Browser network tab — `SdkWorkApiResponse` `code: 0`, `traceId` present |
| Mini program | Rebuilt runtime: `pnpm --filter @sdkwork/agents-mini-program build` |
| SDK drift | `pnpm workflow:build-agents-app-sdk` before release |

## Log fields

Structured request traces should include:

- `traceId` / `x-sdkwork-trace-id`
- HTTP method and path under `/app/v3/api` or `/backend/v3/api`
- Tenant / subject when IAM context is available

## Related

- [incident-rollback.md](./incident-rollback.md) — containment and rollback
- [pre-launch-verification.md](./pre-launch-verification.md) — cutover checklist
- [TECH_ARCHITECTURE.md](../architecture/tech/TECH_ARCHITECTURE.md) §10 — platform-owned rate limit / CORS
