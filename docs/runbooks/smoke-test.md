# Live Smoke Test

Use after deployment or local `pnpm dev` when you need runtime evidence beyond `pnpm verify` contracts.

Automated flow contract (mocked SDK, no live server): `pnpm --filter @sdkwork/agents-pc test:agent-contracts` includes `agent-e2e-flow-contract.test.ts` (create agent → session → chat).

Gateway-only live smoke (server must be running):

```powershell
pnpm dev          # separate terminal
pnpm smoke:live   # GET /health + /metrics/agents
```

## Prerequisites

1. Standalone gateway or split API running with Postgres managed store.
2. IAM session or dev bypass only in **development** (`SDKWORK_AGENTS_DEV_AUTH_BYPASS=true` must be `false` in staging/production).
3. PC or H5 client pointed at the correct `VITE_SDKWORK_AGENTS_*_APP_API_BASE_URL`.

## Smoke sequence

| Step | Action | Pass criterion |
| --- | --- | --- |
| 1 | `GET /health` | HTTP 200 |
| 2 | `GET /metrics/agents` | Prometheus text with `sdkwork_agents_` metrics |
| 3 | Sign in via client Auth Gate | Session tokens stored; no 401 on app API |
| 4 | Create agent | `code: 0`, `data.item.agentId` present |
| 5 | Open chat route | Session created; messages list loads |
| 6 | Send message | Assistant reply from code-engine facade (`RuntimeFacadeChatCompleter`); `runtimeMode` ≠ contract stub; `traceId` in network response |

## Commands

```powershell
# Repository gates (required before any cutover)
pnpm verify

# Local dev stack
pnpm dev

# PC desktop
pnpm start:desktop

# H5 browser
pnpm start:browser
```

## Failure triage

| Symptom | Check |
| --- | --- |
| 401 on `/app/v3/api` | IAM tokens, Auth Gate, `SDKWORK_ACCESS_TOKEN` in dev only |
| 5xx on agent CRUD | `SDKWORK_AGENTS_STORE_DATABASE_*`, migrations (`pnpm db:status`) |
| Chat empty / error | Agent published, model binding, kernel runtime availability |
| Client SDK errors | `pnpm workflow:build-agents-app-sdk`, base URL topology |

See [incident-rollback.md](./incident-rollback.md) and [monitoring.md](./monitoring.md).
