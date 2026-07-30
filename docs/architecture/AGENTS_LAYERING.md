# SDKWork Agents - Layering

SDKWork Agents follows the same layering model as `sdkwork-memory` and `sdkwork-specs/NAMING_SPEC.md`.

## Kernel vs application

| Layer | Repository | Responsibility |
| --- | --- | --- |
| Runtime SPI | `sdkwork-kernel` | Agent lifecycle, providers, sessions and internal runtime API (`/internal/v3/api/...`) |
| Infrastructure HTTP | `sdkwork-web-framework` + Agents gateway | Canonical anonymous liveness, readiness and metrics (`/healthz`, `/livez`, `/readyz`, `/metrics`) |
| Application domain | `sdkwork-agents` | Managed agents composition plane, open/app/backend HTTP surfaces, SDK families, hosted session/message persistence |

`sdkwork-kernel` is mechanism-only. It must not own product policy or managed-agent CRUD.

## Canonical crates (application-owned)

| Crate | Pattern | Role |
| --- | --- | --- |
| `sdkwork-intelligence-agents-service` | `sdkwork-<domain>-<capability>-service` | Domain service: commands, policies, HTTP handlers, managed-store persistence |
| `sdkwork-routes-agents-{open,app,backend}-api` | `sdkwork-routes-<capability>-<surface>` | Route boundaries per HTTP surface |
| `sdkwork-routes-agents-http-shared` | shared route manifests + web-framework bootstrap | OpenAPI-derived manifests, `build_served_combined_router` |
| `sdkwork-agents-kernel-bridge` | application composition | Merges kernel operational router + agents HTTP router |
| `sdkwork-api-agents-standalone-gateway` | `sdkwork-<application-code>-standalone-gateway` | Runnable process |

## SDK families

| Family | Authority | Prefix |
| --- | --- | --- |
| `sdkwork-agents-sdk` | `sdkwork-agents-open-api` | `/agent/v3/api` |
| `sdkwork-agents-app-sdk` | `sdkwork-agents-app-api` | `/app/v3/api` |
| `sdkwork-agents-backend-sdk` | `sdkwork-agents-backend-api` | `/backend/v3/api` |

Internal runtime SDK (`sdkwork-agent-internal-sdk`) remains kernel-owned.

## Database env keys

| Store | Table prefix | Env prefix |
| --- | --- | --- |
| Application database host | app metadata | `SDKWORK_DATABASE_*` |
| Managed agents store (composition plane) | `ai_*` | `SDKWORK_DATABASE_*` (`AGENTS_STORE` is table ownership metadata only) |
| Kernel runtime sessions | kernel-owned | `SDKWORK_DATABASE_*` with kernel-owned tables |

All agents-owned business tables use the `ai_` prefix per `DATABASE_SPEC.md`.
