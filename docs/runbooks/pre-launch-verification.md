# Pre-Launch Verification Runbook

Use this checklist before staging or production cutover. Evidence replaces historical one-off readiness reports.

## 1. Full workspace verification

From repository root:

```powershell
pnpm install
pnpm verify
```

`pnpm verify` runs, in order:

| Step | Command / action |
| --- | --- |
| Spec gates | `pnpm check` (composition, component ports, frontend, permissions, Rust backend, API envelope, operation patterns, route collisions, pagination, SDK imports, apps index, production security, deploy, docs, scripts, workflow, topology, database) |
| App SDK build | `pnpm workflow:build-agents-app-sdk` (ensures generated TypeScript dist matches OpenAPI) |
| Rust build | `cargo build --workspace` |
| Rust tests | `cargo test --workspace --all-features` (includes HTTP + Postgres contract suites) |
| Mini-program runtime | `pnpm --filter @sdkwork/agents-mini-program build` |
| Node contracts | `pnpm check:contracts` (platform integration, database framework, root quality gates, production security, Open SDK surface, mini-program runtime, client surface readiness) |
| Client typecheck | PC / H5 / mini-program `tsc --noEmit` |
| PC agent contracts | scope, management profile, chat service, e2e flow (create → chat) |
| Live smoke (manual) | [smoke-test.md](../runbooks/smoke-test.md) after deployment |

Optional broader sweep:

```powershell
pnpm test
pnpm workflow:build-client-surfaces
```

## 2. Topology and database

```powershell
pnpm topology:validate
pnpm db:validate
pnpm db:drift:check
```

Confirm `configs/topology/*.env` uses canonical subject IDs (`SDKWORK_AGENTS_TENANT_ID=100001`, not legacy `1001`).

## 3. Deploy manifest

```powershell
pnpm check:deploy
```

Authoritative file: [deployments/deploy.yaml](../../../deployments/deploy.yaml).

Validate target profile matches environment (cloud split vs standalone unified).

## 4. API, SDK, pagination, and composition gates

```powershell
pnpm check:api-envelope
pnpm check:api-operation-patterns
pnpm check:route-path-collisions
pnpm check:pagination
pnpm check:app-sdk-consumer-imports
pnpm check:component-port-bindings
pnpm check:frontend-composition
pnpm check:permission-composition
pnpm check:composition-resolver
pnpm check:rust-backend-composition
pnpm check:production-security
```

All L2+ app/backend/open business operations must use `SdkWorkApiResponse` success bodies and `ProblemDetail` errors, follow the API operation matrix, avoid path collisions, expose standard pagination, consume composed SDK imports, preserve SDKWork composition boundaries, and prove production-like profiles reject dev inline auth and contract runtime fallback per [TECH-api-specification.md](../architecture/tech/TECH-api-specification.md) and [TECH_ARCHITECTURE.md](../architecture/tech/TECH_ARCHITECTURE.md).

The same production-security gate must also prove runtime resilience on panic-prone infrastructure paths: failure to install Ctrl+C or SIGTERM handlers is logged for operators and does not terminate the process, poisoned in-memory repository, audit, or metrics locks recover through centralized helpers instead of `expect` panics, managed-store repository and Postgres adapter constructors propagate default Snowflake ID initialization errors through `KernelResult`, and `AgentRepository` persistence methods remain required trait methods so incomplete adapters fail at compile time.

## 5. Packaging (CI parity)

GitHub workflow: `.github/workflows/package.yml` → `sdkwork.workflow.json`.

CI validate lifecycle mirrors local gates plus client typecheck.

## 6. GA Scope Boundaries

Documented in [TECH_ARCHITECTURE.md](../architecture/tech/TECH_ARCHITECTURE.md) §10:

| Area | GA treatment | Owner |
| --- | --- | --- |
| Token-level SSE streaming | Excluded from the current GA evidence bundle; current SDKWork contract ships single completion SSE events | `sdkwork-kernel` provider stream SPI |
| Rate limit / CORS middleware | Enforced by the shared web-framework adoption plan, not reimplemented locally | `sdkwork-web-framework` |
| Flutter mobile app | Excluded from the current GA evidence bundle until an owned Dart app SDK is available | `sdkwork-agents` mobile track |
| File upload | Excluded from product scope unless the feature wires sdkwork-drive Drive Uploader first | `sdkwork-drive` + consuming feature |

## Sign-off

| Gate | Pass criterion |
| --- | --- |
| `pnpm verify` | Exit code 0 |
| Topology | `sdkwork-topology validate` ok |
| Database | `db:validate` + no drift on target env |
| Deploy | `check-deploy-standard` ok for target profile |
