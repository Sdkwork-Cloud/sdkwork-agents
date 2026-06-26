# SDKWork Agents

Domain: `intelligence`  
Capability: `agents`  
Status: beta (kernel-composed HTTP service; production postgres path wired)

SDKWork Agents is the hosted agent product application. It owns the intelligence
`agents` domain service, HTTP routes, SDKs, and managed-store persistence. It
composes [`sdkwork-kernel`](../sdkwork-kernel/) for runtime SPI only (sessions,
internal runtime API, operational HTTP). See [`docs/architecture/AGENTS_LAYERING.md`](docs/architecture/AGENTS_LAYERING.md) and [`docs/architecture/KERNEL_MIGRATION.md`](docs/architecture/KERNEL_MIGRATION.md).

## Canonical References

- Standards: [`../sdkwork-specs/README.md`](../sdkwork-specs/README.md)
- Kernel: [`../sdkwork-kernel/README.md`](../sdkwork-kernel/README.md)
- Local specs: [`specs/README.md`](specs/README.md)
- Documentation: [`docs/README.md`](docs/README.md)

## Quick Start

```powershell
pnpm install
pnpm dev
pnpm verify
```

## Workspace Crates

| Crate | Role |
| --- | --- |
| `sdkwork-intelligence-agents-service` | Domain service: managed agents, marketplace, knowledge registry, managed-store persistence |
| `sdkwork-routes-agents-{open,app,backend}-api` | HTTP route boundaries per surface |
| `sdkwork-routes-agents-http-shared` | OpenAPI route manifests + web-framework bootstrap |
| `sdkwork-agents-contract` | Runtime env helpers (`SDKWORK_AGENTS_*`, dev auth gating) |
| `sdkwork-agents-kernel-bridge` | Composes kernel operational router + agents HTTP router |
| `sdkwork-agents-database-host` | Application `agents_*` registry database lifecycle |
| `sdkwork-agents-gateway-assembly` | Gateway router assembly |
| `sdkwork-agents-api-server` | Runnable binary (`sdkwork-agents-api-server`, `sdkwork-agents-standalone-gateway`) |
| `sdkwork-agents-integration-tests` | API bootstrap, gateway, and database smoke tests |

## Database & Migration

```powershell
pnpm db:materialize:contract
cargo run -p sdkwork-agents-api-server -- db-migrate
```

Application metadata uses `SDKWORK_AGENTS_DATABASE_*`. Kernel agents managed store persistence uses `SDKWORK_AGENTS_STORE_DATABASE_*`.

## Deployment

- Docker: [`deployments/docker/Dockerfile`](deployments/docker/Dockerfile)
- Kubernetes: [`deployments/kubernetes/`](deployments/kubernetes/)
- Topology profiles: [`configs/topology/`](configs/topology/)
- Local env template: [`.env.example`](.env.example)

## Platform Integration

| Framework | Status |
| --- | --- |
| `sdkwork-web-framework` | Integrated via `sdkwork-routes-agents-*` route crates |
| `sdkwork-database` | Integrated (app registry + agents managed store + kernel runtime DB) |
| `sdkwork-utils` | Integrated in `sdkwork-agents-contract` |
| `sdkwork-drive` | Required for file upload features |
| `sdkwork-discovery` | Deferred until RPC services ship |

## Application Roots

- [apps directory index](apps/README.md)
