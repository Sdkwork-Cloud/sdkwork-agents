# Operator Guide

Deployment and runtime validation for SDKWork Agents.

## Pre-flight

From the application root:

```powershell
pnpm verify
pnpm topology:validate
pnpm db:validate
```

These gates cover API and SDK standards, operation semantics, route collision checks, pagination, SDK import boundaries, composition boundaries, deploy manifest (`deployments/deploy.yaml`), topology spec, and database framework alignment.

**Staging/production cutover:** [runbooks/pre-launch-verification.md](../../runbooks/pre-launch-verification.md)

**Ongoing operations:** [runbooks/monitoring.md](../../runbooks/monitoring.md), [runbooks/smoke-test.md](../../runbooks/smoke-test.md), [runbooks/incident-rollback.md](../../runbooks/incident-rollback.md)

## Deployment profiles

Authoritative manifest: [deployments/deploy.yaml](../../../deployments/deploy.yaml)

| Profile | Use case |
| --- | --- |
| `cloud.production` | Cloud web + API domains through application public ingress |
| `standalone.production` | Single application public ingress binary package |
| `standalone.development` | Local source-tree dev |

Topology and env keys: [specs/topology.spec.json](../../../specs/topology.spec.json), `configs/topology/*.env`.

## Database

```powershell
pnpm db:status
pnpm db:migrate
pnpm db:drift:check
```

## Packaging (CI)

GitHub workflow: `.github/workflows/package.yml` → reusable `sdkwork-package.yml` driven by `sdkwork.workflow.json`.

Manual dispatch inputs include sibling dependency refs (`SDKWORK_KERNEL_REF`, `SDKWORK_KNOWLEDGEBASE_REF`, etc.).

## Non-GA Platform Scope

Rate limiting, CORS, and dashboard wiring are owned by `sdkwork-web-framework` and ops. `sdkwork-agents` must consume those platform capabilities instead of reimplementing them locally. See [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md) section 10.

## Canon

- [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md)
- [runbooks/pre-launch-verification.md](../../runbooks/pre-launch-verification.md)
- [runbooks/monitoring.md](../../runbooks/monitoring.md)
- [runbooks/smoke-test.md](../../runbooks/smoke-test.md)
- [runbooks/incident-rollback.md](../../runbooks/incident-rollback.md)
