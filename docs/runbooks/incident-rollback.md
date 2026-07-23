# Incident Response And Rollback

Use for elevated errors, data-integrity risk, security exposure or failed
release verification.

## 1. Triage

1. Classify impact by surface, tenant scope, operation and lifecycle state.
2. Capture server-owned `traceId`, release version, deployment profile and
   relevant bounded resource identifiers.
3. Check health, metrics, PostgreSQL pool/status, outbox backlog and provider
   binding health.
4. Verify development auth bypass is disabled and no credential mode changed.

Do not log Session Item content, tool arguments, credentials, API keys or claim
tokens while collecting evidence.

## 2. Containment

- Stop the rollout and route traffic to the last healthy replica set.
- Disable an unhealthy provider binding through its governed command when the
  incident is provider-specific.
- Preserve read access when safe; reject new execution when write integrity is
  uncertain.
- Do not enable an in-memory store, raw provider transport, alternate Session
  path or relaxed authorization as a workaround.

## 3. Application Rollback

1. Select the last signed, checksum-verified artifact compatible with the
   current database contract.
2. Redeploy through the same profile and configuration authority.
3. Verify `GET /health`, metrics, one authenticated Session retrieve and one
   idempotency reconciliation.
4. Monitor error, Turn latency, database pool and outbox backlog until stable.

## 4. Database Recovery

1. Stop writers when data integrity is at risk.
2. Run `pnpm db:status` and `pnpm db:drift:check` against the target.
3. Do not reverse schema manually. Use only a reviewed database-framework plan
   compatible with the deployed binary.
4. Restore a verified PostgreSQL snapshot only under the platform recovery
   procedure, then validate migration history and table registry.
5. Reconcile idempotent Turn/outbox state before reopening writes.

Cross-module data is never repaired by writing IM, Drive, Skill or other
dependency tables from Agents.

## 5. Recovery Verification

```powershell
node scripts/generate-agents-api-docs.mjs --check
node scripts/check-agent-sdk-workspace.mjs
pnpm db:validate
pnpm topology:validate
cargo test -p sdkwork-intelligence-agents-service --features http-axum --test http_axum_contracts
```

Repeat the live [smoke test](./smoke-test.md) against the recovered candidate.

## 6. Follow-Up

- record timeline, root cause, affected release and bounded impact;
- add the narrow regression test before closing;
- update an ADR only when architecture or contract behavior changes;
- rotate exposed credentials and verify revocation when security is involved;
- retain audit and release evidence according to policy.
