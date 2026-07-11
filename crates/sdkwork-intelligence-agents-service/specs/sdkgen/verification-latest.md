# Latest SDK Verification Record

- Date: 2026-06-05
- Application domain: `agent`
- SDK workspace: `sdks/`
- Generator profile: `--standard-profile sdkwork-v3`
- Generator entrypoint:
  `../sdkwork-sdk-generator/bin/sdkgen.js`

## Commands

```powershell
node .\sdks\materialize-agent-v3-openapi-boundaries.mjs
node .\sdks\workspace-agent-sdkgen.mjs --mode apply
node .\scripts\check-agent-sdk-workspace.mjs
node .\sdks\workspace-agent-sdkgen.mjs --mode dry-run
```

Generated package checks:

```powershell
node .\sdks\sdkwork-agents-sdk\sdkwork-agents-sdk-typescript\generated\server-openapi\bin\publish-core.mjs --language typescript --project-dir .\sdks\sdkwork-agents-sdk\sdkwork-agents-sdk-typescript\generated\server-openapi --action check
node .\sdks\sdkwork-agents-app-sdk\sdkwork-agents-app-sdk-typescript\generated\server-openapi\bin\publish-core.mjs --language typescript --project-dir .\sdks\sdkwork-agents-app-sdk\sdkwork-agents-app-sdk-typescript\generated\server-openapi --action check
node .\sdks\sdkwork-agents-backend-sdk\sdkwork-agents-backend-sdk-typescript\generated\server-openapi\bin\publish-core.mjs --language typescript --project-dir .\sdks\sdkwork-agents-backend-sdk\sdkwork-agents-backend-sdk-typescript\generated\server-openapi --action check
node .\sdks\sdkwork-agents-sdk\sdkwork-agents-sdk-typescript\generated\server-openapi\bin\publish-core.mjs --language typescript --project-dir .\sdks\sdkwork-agents-sdk\sdkwork-agents-sdk-typescript\generated\server-openapi --action build
node .\sdks\sdkwork-agents-app-sdk\sdkwork-agents-app-sdk-typescript\generated\server-openapi\bin\publish-core.mjs --language typescript --project-dir .\sdks\sdkwork-agents-app-sdk\sdkwork-agents-app-sdk-typescript\generated\server-openapi --action build
node .\sdks\sdkwork-agents-backend-sdk\sdkwork-agents-backend-sdk-typescript\generated\server-openapi\bin\publish-core.mjs --language typescript --project-dir .\sdks\sdkwork-agents-backend-sdk\sdkwork-agents-backend-sdk-typescript\generated\server-openapi --action build
```

The package checks are root-runnable commands and pass the generated package
directory through `--project-dir` so the verification record does not depend on
an implicit shell working directory.

## Families

- developer/open SDK (`sdkwork-agents-sdk`)
  - authority: `sdkwork-agents-open-api`
  - prefix: `/agent/v3/api`
  - package: `@sdkwork/agents-sdk`
  - output:
    `sdks/sdkwork-agents-sdk/sdkwork-agents-sdk-typescript/generated/server-openapi`
  - status: authority and derived sdkgen inputs materialized
  - generation status: script-derived from the strict-profile app SDK source,
    because the current `sdkwork-v3` standard profile supports `app`,
    `backend`, and `im` prefixes only.
  - check: pass
  - build: pass

- app SDK (`sdkwork-agents-app-sdk`)
  - authority: `sdkwork-agents-app-api`
  - prefix: `/app/v3/api`
  - package: `@sdkwork/agents-app-sdk`
  - output:
    `sdks/sdkwork-agents-app-sdk/sdkwork-agents-app-sdk-typescript/generated/server-openapi`
  - check: pass
  - build: pass

- backend SDK (`sdkwork-agents-backend-sdk`)
  - authority: `sdkwork-agents-backend-api`
  - prefix: `/backend/v3/api`
  - package: `@sdkwork/agents-backend-sdk`
  - output:
    `sdks/sdkwork-agents-backend-sdk/sdkwork-agents-backend-sdk-typescript/generated/server-openapi`
  - check: pass
  - build: pass

## Dry-Run Summary

The latest dry-run report is stored in:

- `sdkwork-intelligence-agents-service/specs/sdkgen/verification-latest.json`
- `sdkwork-intelligence-agents-service/specs/sdkgen/verification-ci.json`
- `sdks/.sdkgen-agent-workspace-report.json`

Latest dry-run state:

- `sdkwork-agents-sdk`: standard-profile generator skipped with recorded
  support gap; open SDK derivation `hasChanges=false`.
- `sdkwork-agents-app-sdk`: `hasChanges=false`, `riskLevel=low`.
- `sdkwork-agents-backend-sdk`: `hasChanges=false`, `riskLevel=low`.

## Contract Checks

- Authority OpenAPI and derived `*.sdkgen.yaml` are separated.
- `*.sdkgen.yaml` inputs inline explicit RFC 9457
  `application/problem+json` responses for generator strict profile.
- `X-Request-Id` is not exposed.
- Generated output stays under `generated/server-openapi`.
- Runtime now exposes `/agent/v3/api`, `/app/v3/api`, and `/backend/v3/api`
  route families.
