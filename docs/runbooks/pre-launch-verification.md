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
| Spec gates | `pnpm check` (composition, component ports, frontend, permissions, Rust backend, API envelope, operation patterns, route collisions, pagination, SDK imports, agent SDK workspace, apps index, production security, deploy, cloud gateway bundle, docs, scripts, workflow, topology, database) |
| App SDK build | `pnpm workflow:build-agents-app-sdk` (ensures generated TypeScript dist matches OpenAPI) |
| Rust build | `cargo build --workspace` |
| Rust tests | `cargo test --workspace --all-features` (includes HTTP + Postgres contract suites) |
| Mini-program runtime | `pnpm --filter @sdkwork/agents-mini-program build` |
| Node contracts | `pnpm check:contracts` (platform integration, database framework, root quality gates, production security, frontend service identity, Open SDK surface, mini-program runtime, client surface readiness) |
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
pnpm gateway:validate:cloud
```

Authoritative file: [deployments/deploy.yaml](../../../deployments/deploy.yaml).

Validate the target profile id matches the release environment, for example
`cloud.production` for hosted production and `standalone.production` for the
single-ingress production package.

Cloud profile cutover must also validate the generated cloud gateway bundle so
gateway configuration and topology surfaces stay aligned before packaging or
deployment approval.

## 3.1 App manifest release channel

All `sdkwork.app.config.json` files must keep `publish.status`, media readiness,
and release channels aligned. While an application manifest still contains
generated placeholder media assets, the release channel must remain `BETA`; do
not expose `release.latest.STABLE`, `release.defaultChannel=STABLE`, or
`STABLE` release notes until product media assets are governed and GA-ready.

The root quality gate enforces this in `pnpm check:contracts`. Real GA cutover
requires replacing placeholder media with governed Drive-backed or immutable
public media resources before moving the channel to `STABLE`.

## 4. API, SDK, pagination, and composition gates

```powershell
pnpm check:api-envelope
pnpm check:api-operation-patterns
pnpm check:route-path-collisions
pnpm check:pagination
pnpm check:app-sdk-consumer-imports
pnpm check:agent-sdk-workspace
pnpm check:component-port-bindings
pnpm check:frontend-composition
pnpm check:frontend-service-identity
pnpm check:permission-composition
pnpm check:composition-resolver
pnpm check:rust-backend-composition
pnpm check:production-security
```

All L2+ app/backend/open business operations must use `SdkWorkApiResponse` success bodies and `ProblemDetail` errors, follow the API operation matrix, avoid path collisions, expose standard pagination, consume composed SDK imports, preserve SDKWork composition boundaries, and prove production-like profiles reject dev inline auth and contract runtime fallback per [TECH-api-specification.md](../architecture/tech/TECH-api-specification.md) and [TECH_ARCHITECTURE.md](../architecture/tech/TECH_ARCHITECTURE.md).

The frontend service identity gate must also pass before cutover. It verifies authored PC/H5 agents service source does not call `crypto.randomUUID()` directly, so request/trace identity remains server-owned and client-side business IDs go through approved SDKWork utility helpers. PC/H5 chat services must reject malformed SDK responses that omit `assistantMessage.messageId`; they must not synthesize local fallback message IDs for persisted server messages. Root quality gates also verify PC/H5 session bridges preserve IAM/AppContext fields from appbase or JWT claims and do not locally default `environment`, `deploymentMode`, or `authLevel`.

The same production-security gate must also prove runtime resilience on panic-prone infrastructure paths: failure to install Ctrl+C or SIGTERM handlers is logged for operators and does not terminate the process, dev-only static policy construction rejects unsafe production-like bootstrap or falls back to deny-all instead of panicking, poisoned in-memory repository, audit, or metrics locks recover through centralized helpers instead of `expect` panics, managed-store repository and Postgres adapter constructors propagate default Snowflake ID initialization errors through `KernelResult`, route manifest build scripts return explicit OpenAPI/environment/file-system errors instead of panicking, and `AgentRepository` persistence methods remain required trait methods so incomplete adapters fail at compile time.

## 5. Packaging (CI parity)

GitHub workflow: `.github/workflows/package.yml` → `sdkwork.workflow.json`.

CI validate lifecycle mirrors local gates plus client typecheck. The server artifact must include
the release gateway binary, `database/**` lifecycle assets, production topology templates,
`deployments/**` including HPA/PDB manifests, and `sdkwork.app.config.json`. Packaging only the
binary is invalid because managed-store initialization and cluster deployment would be incomplete.

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
| Deploy | `check-deploy-standard` and `gateway:validate:cloud` ok for target profile |
