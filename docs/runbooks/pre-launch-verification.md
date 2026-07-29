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

- operation counts are App 81, Backend 48 and Open 47;
- no client-writable tenant, organization or user selector exists;
- all list operations use store-level pagination;
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
- Open SDK exposes exactly its 47 API-key operations;
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

Validate the target profile separately. The expected module contains 20
Agents-owned PostgreSQL `ai_*` tables. Confirm migrations/checksums match the
baseline, `init` uses the greenfield baseline only when its completion anchor is
absent, and an anchored partial schema fails drift validation instead of being
rebuilt. There must be no extra session, item, interaction or dependency-owned
tables.

For a configured integration environment:

```powershell
pnpm test:database:postgres-live
```

## 5. Rust Gates

```powershell
cargo check -p sdkwork-intelligence-agents-service --features http-axum
cargo test -p sdkwork-intelligence-agents-service --features http-axum --lib
cargo test -p sdkwork-intelligence-agents-service --features http-axum --test http_axum_contracts
cargo test -p sdkwork-agents-runtime-facade
cargo check -p sdkwork-agents-kernel-bridge
cargo check -p sdkwork-api-agents-assembly
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
```

Run `pnpm verify` for the complete repository gate after all narrow checks pass.

## 7. Live Evidence

Deploy the exact candidate artifact, then execute
[smoke-test.md](./smoke-test.md). Confirm:

- health and metrics;
- App/Backend dual-token enforcement and Open API-key enforcement;
- create/retrieve Session;
- execute/retrieve/cancel Turn;
- ordered Session Item pagination;
- Interaction claim and resolution;
- restart persistence and idempotent retry;
- no secret or item content in logs/metrics.

## 8. Sign-Off

| Gate | Required result |
| --- | --- |
| API/docs | exact inventory and all validators pass |
| SDK | generation idempotent; TypeScript and Flutter checks pass |
| Database | framework, status and drift checks pass on target |
| Rust | service, facade, bridge and assembly checks pass |
| Security | auth modes, trusted context and isolation tests pass |
| Deployment | standalone/cloud profile validation passes |
| Live | smoke sequence and restart reconciliation pass |

Do not waive a failed gate by enabling an alternate store, compatibility
adapter, raw transport or development auth path.
