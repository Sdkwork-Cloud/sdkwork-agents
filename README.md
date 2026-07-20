# SDKWork Agents
repository-kind: application

Domain: `intelligence`  
Capability: `agents`  
Status: pre-launch (kernel-composed HTTP service; production postgres path wired)

SDKWork Agents is the hosted agent product application. It owns the intelligence
`agents` domain service, HTTP routes, SDKs, and managed-store persistence. It
composes [`sdkwork-kernel`](../sdkwork-kernel/) for runtime SPI only (sessions,
internal runtime API, operational HTTP). See [`docs/architecture/AGENTS_LAYERING.md`](docs/architecture/AGENTS_LAYERING.md).

## Canonical References

- Product PRD: [`docs/product/prd/PRD.md`](docs/product/prd/PRD.md)
- Technical architecture: [`docs/architecture/tech/TECH_ARCHITECTURE.md`](docs/architecture/tech/TECH_ARCHITECTURE.md)
- Standards: [`../sdkwork-specs/README.md`](../sdkwork-specs/README.md)
- Kernel: [`../sdkwork-kernel/README.md`](../sdkwork-kernel/README.md)
- Local specs: [`specs/README.md`](specs/README.md)
- Documentation index: [`docs/README.md`](docs/README.md)

## Quick Start

```powershell
pnpm install
pnpm dev
pnpm verify
```

## Workspace Crates

| Crate | Role |
| --- | --- |
| `sdkwork-intelligence-agents-service` | Domain service: managed agents, composition slots, marketplace, managed-store persistence |
| `sdkwork-routes-agents-{open,app,backend}-api` | HTTP route boundaries per surface |
| `sdkwork-routes-agents-http-shared` | OpenAPI route manifests + web-framework bootstrap |
| `sdkwork-agents-contract` | Runtime env helpers (`SDKWORK_AGENTS_*`, dev auth gating) |
| `sdkwork-agents-kernel-bridge` | Composes kernel operational router + agents HTTP router |
| `sdkwork-agents-database-host` | Application `agents_*` registry database lifecycle |
| `sdkwork-api-agents-assembly` | Gateway router assembly |
| `sdkwork-api-agents-standalone-gateway` | Runnable binary (`sdkwork-api-agents-standalone-gateway`) |
| `sdkwork-agents-integration-tests` | API bootstrap, gateway, and database smoke tests |

## Database & Migration

```powershell
pnpm db:materialize:contract
cargo run -p sdkwork-api-agents-standalone-gateway -- db-migrate
```

Application metadata uses `SDKWORK_AGENTS_DATABASE_*`. Kernel agents managed store persistence uses `SDKWORK_AGENTS_STORE_DATABASE_*`.

## Deployment

- Deploy manifest: [`deployments/deploy.yaml`](deployments/deploy.yaml) (cloud + standalone profiles)
- Docker: [`deployments/docker/Dockerfile`](deployments/docker/Dockerfile)
- Kubernetes: [`deployments/kubernetes/`](deployments/kubernetes/)
- Topology profiles: [`etc/topology/`](etc/topology/)
- Local env template: [`.env.example`](.env.example)

Pre-flight: `pnpm verify` and `pnpm topology:validate`. See [docs/runbooks/pre-launch-verification.md](docs/runbooks/pre-launch-verification.md).

## Platform Integration

| Framework | Status |
| --- | --- |
| `sdkwork-web-framework` | Integrated via `sdkwork-routes-agents-*` route crates |
| `sdkwork-database` | Integrated (app registry + agents managed store + kernel runtime DB) |
| `sdkwork-utils` | Integrated in contract, service response/validation, runtime-facade |
| `sdkwork-drive` | Integrated for PC uploads | PC core Drive Uploader facade + `@sdkwork/drive-app-sdk`; Agents persists canonical Drive references only |
| `sdkwork-discovery` | Deferred until RPC services ship |

## Application Roots

- [apps directory index](apps/README.md)
