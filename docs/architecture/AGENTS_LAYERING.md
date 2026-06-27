# SDKWork Agents — Layering

SDKWork Agents follows the same layering model as `sdkwork-memory` and `sdkwork-specs/NAMING_SPEC.md`.

## Kernel vs application

| Layer | Repository | Responsibility |
| --- | --- | --- |
| Runtime SPI | `sdkwork-kernel` | Agent lifecycle, providers, sessions, internal runtime API, operational HTTP (`/health`, `/metrics`, `/internal/v3/api/...`) |
| Application domain | `sdkwork-agents` | Managed agents registry, marketplace, knowledge/memory metadata, open/app/backend HTTP surfaces, SDK families |

`sdkwork-kernel` is mechanism-only (Linux-kernel style). It must not own product policy or managed-agent CRUD.

## Canonical crates (application-owned)

| Crate | Pattern | Role |
| --- | --- | --- |
| `sdkwork-intelligence-agents-service` | `sdkwork-<domain>-<capability>-service` | Domain service: commands, policies, HTTP handlers, managed-store persistence |
| `sdkwork-routes-agents-{open,app,backend}-api` | `sdkwork-routes-<capability>-<surface>` | Route boundaries per HTTP surface |
| `sdkwork-routes-agents-http-shared` | shared route manifests + web-framework bootstrap | OpenAPI-derived manifests, `build_served_combined_router` |
| `sdkwork-agents-kernel-bridge` | application composition | Merges kernel operational router + agents HTTP router |
| `sdkwork-agents-standalone-gateway` | `sdkwork-<application-code>-standalone-gateway` | Runnable process |

## SDK families

| Family | Authority | Prefix |
| --- | --- | --- |
| `sdkwork-agents-sdk` | `sdkwork-agents-open-api` | `/agent/v3/api` |
| `sdkwork-agents-app-sdk` | `sdkwork-agents-app-api` | `/app/v3/api` |
| `sdkwork-agents-backend-sdk` | `sdkwork-agents-backend-api` | `/backend/v3/api` |

Internal runtime SDK (`sdkwork-agent-internal-sdk`) remains kernel-owned.

## Database env keys

| Store | Service code | Env prefix |
| --- | --- | --- |
| Application registry (`agents_*` tables) | `AGENTS` | `SDKWORK_AGENTS_DATABASE_*` |
| Managed agents store (`a_*` tables) | `AGENTS_STORE` | `SDKWORK_AGENTS_STORE_DATABASE_*` |
| Kernel runtime sessions | kernel-owned | `SDKWORK_AGENT_SERVER_DATABASE_*` |

## Capability ownership

Application-owned: `agents-domain-service`, `agents-http-routes`, `agents-persistence`, `agents-sdks`, `agents-api-contracts`.

Kernel-owned: `agent-runtime`, `agent-session`, `agent-runtime-persistence`, `agent-internal-api`, `agent-server-operational-http`.
