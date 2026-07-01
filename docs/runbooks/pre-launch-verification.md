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
| Spec gates | `pnpm check` (composition, architecture, identity, API envelope, deploy, docs, scripts, workflow, topology, database) |
| App SDK build | `pnpm workflow:build-agents-app-sdk` (ensures generated TypeScript dist matches OpenAPI) |
| Rust build | `cargo build --workspace` |
| Rust tests | `cargo test --workspace --all-features` (includes HTTP + Postgres contract suites) |
| Mini-program runtime | `pnpm --filter @sdkwork/agents-mini-program build` |
| Node contracts | `pnpm check:contracts` (platform integration, database framework, Open SDK surface, mini-program runtime, client surface readiness) |
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

## 4. API envelope

```powershell
pnpm check:api-envelope
```

All L2+ app/backend/open business operations must use `SdkWorkApiResponse` success bodies and `ProblemDetail` errors per [TECH-api-specification.md](../architecture/tech/TECH-api-specification.md).

## 5. Packaging (CI parity)

GitHub workflow: `.github/workflows/package.yml` → `sdkwork.workflow.json`.

CI validate lifecycle mirrors local gates plus client typecheck.

## 6. Post-launch (not blockers)

Documented in [TECH_ARCHITECTURE.md](../architecture/tech/TECH_ARCHITECTURE.md) §10:

- Token-level SSE streaming (kernel)
- Rate limit / CORS / Prometheus (web-framework)
- Flutter Dart SDK wiring
- Drive uploader integration when upload ships

## Sign-off

| Gate | Pass criterion |
| --- | --- |
| `pnpm verify` | Exit code 0 |
| Topology | `sdkwork-topology validate` ok |
| Database | `db:validate` + no drift on target env |
| Deploy | `check-deploy-standard` ok for target profile |
