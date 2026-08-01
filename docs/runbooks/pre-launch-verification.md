# Pre-Launch Verification Runbook

Use this runbook before staging or production release. Record command, commit,
profile, timestamp, exit code and important output in release evidence.

## 1. Preconditions

- Sibling `sdkwork-specs`, `sdkwork-kernel`, `sdkwork-web-framework`,
  `sdkwork-database`, `sdkwork-utils` and SDK generator workspaces resolve.
- Production uses PostgreSQL with TLS and a bounded connection pool.
- `database/database.manifest.json` keeps `lifecycle.autoMigrate=false`; the
  release workflow applies pending migrations explicitly before rollout.
- `SDKWORK_AGENTS_DEV_AUTH_BYPASS` is absent or false.
- App/Backend token providers and Open API key providers are configured
  independently.
- Required independent capability endpoints are explicit in the selected
  source configuration.
- No secret is stored in manifests, checked-in profiles, logs or SDK examples.

## 2. Contract And Documentation Gates

```powershell
node scripts/generate-agents-api-docs.mjs --check
node ../sdkwork-specs/tools/check-api-response-envelope.mjs --workspace .
node ../sdkwork-specs/tools/check-api-operation-patterns.mjs --workspace .
node ../sdkwork-specs/tools/check-route-path-collisions.mjs --workspace .
node ../sdkwork-specs/tools/check-pagination.mjs --workspace .
pnpm check:agents-im-boundary
pnpm check:docs
```

Pass criteria:

- operation counts are App 102, Backend 58 and Open 56;
- no client-writable tenant, organization or user selector exists;
- all list operations use store-level pagination;
- offset and cursor list declarations match their `PageInfo` mode;
- Session Item pagination uses opaque keyset cursors bound to owner, Session,
  filters, and sort order;
- Session, Turn, SessionItem and Interaction naming is consistent;
- IM reverse dependencies and IM-owned persistence are absent.

## 3. SDK Gates

```powershell
node sdks/workspace-agent-sdkgen.mjs --mode dry-run
node scripts/agents-open-sdk-surface-contract.test.mjs
node scripts/check-agent-sdk-workspace.mjs
pnpm --filter @sdkwork/agents-app-sdk test

flutter analyze apps/sdkwork-agents-flutter-mobile
flutter analyze apps/sdkwork-agents-flutter-mobile/packages/sdkwork_agents_flutter_mobile_core
flutter test apps/sdkwork-agents-flutter-mobile/packages/sdkwork_agents_flutter_mobile_core
```

Pass criteria:

- dry run reports `hasChanges: false` for every family/language;
- Open SDK exposes exactly its 56 API-key operations;
- TypeScript consumers use scoped package roots;
- Flutter package root contains `pubspec.yaml` and
  `lib/sdkwork_agents_app_sdk.dart`;
- Flutter mobile core consumes `sdkwork_agents_app_sdk` through its generated
  package root and no raw HTTP transport.

## 4. Database Gates

```powershell
pnpm db:validate
pnpm db:status
pnpm db:drift:check
```

Validate the target profile separately. The expected module contains 23
Agents-owned PostgreSQL `ai_*` tables. Confirm migrations/checksums match the
baseline, `init` uses the greenfield baseline only when its completion anchor is
absent, and an anchored partial schema fails drift validation instead of being
rebuilt. There must be no extra session, item, interaction or dependency-owned
tables.

For a configured integration environment:

```powershell
pnpm test:database:postgres-live
```

The live suite requires `SDKWORK_DATABASE_URL`,
`SDKWORK_DATABASE_ADMIN_HOST`, `SDKWORK_DATABASE_ADMIN_DATABASE`,
`SDKWORK_DATABASE_ADMIN_USERNAME`, and `SDKWORK_DATABASE_ADMIN_PASSWORD`.
`SDKWORK_DATABASE_ADMIN_PORT` and `SDKWORK_DATABASE_ADMIN_SSL_MODE` are
optional. The administrator must be allowed to create and remove the isolated
`sdkwork_ai_test_*` database/schema used by the suite.

## 5. Rust Gates

```powershell
cargo check -p sdkwork-intelligence-agents-service --features http-axum
cargo test -p sdkwork-intelligence-agents-service --features http-axum --lib
cargo test -p sdkwork-intelligence-agents-service --features http-axum --test http_axum_contracts
cargo test -p sdkwork-agents-runtime-facade
cargo check -p sdkwork-agents-kernel-bridge
cargo check -p sdkwork-api-agents-assembly
cargo check -p sdkwork-intelligence-agents-worker
cargo test -p sdkwork-intelligence-agents-worker
cargo build --release -p sdkwork-api-agents-standalone-gateway
cargo build --release -p sdkwork-intelligence-agents-worker
```

Pass criteria include trusted request context, cross-tenant denial, idempotency
conflict, optimistic concurrency, Turn cancellation, typed SSE completion,
Interaction claim, checkpoint lifecycle and repository pagination.

## 6. Application And Deployment Gates

```powershell
pnpm check:production-security
pnpm check:release-supply-chain
pnpm topology:validate
pnpm deploy:validate:standalone
pnpm deploy:validate:cloud
pnpm workflow:typecheck-client-surfaces
pnpm workflow:build-client-surfaces
kubectl kustomize deployments/kubernetes
```

Run `pnpm verify` for the complete repository gate after all narrow checks pass.
Confirm the gateway and Task Worker are separate workloads with independent
replica counts, disruption budgets, resource limits, probes and metrics. Both
must receive a unique Pod UID through `SDKWORK_NODE_INSTANCE_ID`; readiness
must fail when the PostgreSQL-backed Snowflake node lease cannot be acquired.
The Worker must expose `/healthz`, `/readyz` and `/metrics` on its private
operations service.

## 7. Live Evidence

Deploy the exact candidate artifact, then execute
[smoke-test.md](./smoke-test.md). Confirm:

- health and metrics;
- App/Backend dual-token enforcement and Open API-key enforcement;
- create/retrieve Session;
- Workspace-scoped Project search, exact-name lookup, Project Session listing,
  and exact Project Session retrieval;
- explicit import-only Project Session synchronization with partial-result
  counts and bounded issue accounting;
- execute/retrieve/cancel Turn;
- create a Session-bound one-time Task and cron Task;
- materialize one occurrence exactly once and inspect its Run and Attempt;
- execute the same manual idempotency key twice and observe one logical Run;
- scale and restart the Worker, then verify expired-lease recovery and stale
  fencing rejection without duplicate occurrence materialization;
- newest-first opaque-cursor Session Item retrieval with chronological client
  presentation and earlier-page continuation;
- Interaction claim and resolution;
- restart persistence and idempotent retry;
- no secret or item content in logs/metrics.

Outbox rows must be created atomically with lifecycle changes. External outbox
delivery is a release blocker for event-dependent features until the approved
platform publisher SPI and its relay are integrated and verified; a local
Kafka producer or raw HTTP dispatcher is not an acceptable substitute.

## 8. Sign-Off

| Gate | Required result |
| --- | --- |
| API/docs | exact inventory and all validators pass |
| SDK | generation idempotent; TypeScript and Flutter checks pass |
| Database | framework, status and drift checks pass on target |
| Rust | service, facade, bridge and assembly checks pass |
| Security | auth modes, trusted context and isolation tests pass |
| Worker | release build, probes, bounded metrics, drain and recovery pass |
| Deployment | standalone/cloud validation and Kubernetes rendering pass |
| Live | smoke sequence, scheduling, fencing and restart reconciliation pass |

Do not waive a failed gate by enabling an alternate store, compatibility
adapter, raw transport or development auth path.
