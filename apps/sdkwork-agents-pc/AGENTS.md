# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing tasks in this application root. Follow specs before memory, dictionary before context, stop on ambiguity, and evidence before completion.

## SDKWORK Standards

Canonical SDKWORK specs path from this application root:

- `../../../sdkwork-specs/README.md`
- `../../../sdkwork-specs/SOUL.md`
- `../../../sdkwork-specs/AGENTS_SPEC.md`
- `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`
- `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`
- `../../../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../../../sdkwork-specs/NAMING_SPEC.md`

Do not copy root standard text into this application root. If these relative paths do not resolve, stop and report the broken workspace layout.

## Application Identity

Read `sdkwork.app.config.json` only when changing PC application behavior, runtime config, SDK wiring, release metadata, packaging, or app-owned capabilities. Managed-agent HTTP and SDK authorities are owned by the repository root `sdkwork-agents` application, not `sdkwork-kernel`.

## Local Dictionary Structure

- `AGENTS.md`: local application agent entrypoint and relative SDKWork spec index.
- `CLAUDE.md`, `GEMINI.md`, `CODEX.md`: compatibility shims that point to `AGENTS.md`.
- `sdkwork.app.config.json`: PC application identity and release metadata.
- `.sdkwork/`: application dictionary for local skills, plugins, manifests, and AI workspace metadata.
- `specs/`: local PC application/component contracts.
- `packages/`: PC React package family for agents client surfaces.
- `src/`: thin PC application bootstrap and shell entry.
- `package.json`: app-surface command manifest governed by `PNPM_SCRIPT_SPEC.md`.

## Spec Resolution Order

Use dynamic progressive loading:

1. Read this `AGENTS.md` and the repository root `../../../AGENTS.md`.
2. Read `sdkwork.app.config.json` only when app behavior, runtime config, SDK wiring, or release metadata is touched.
3. Read local `specs/README.md` and `specs/component.spec.json` only when local contracts are relevant.
4. Read `../../../sdkwork-specs/README.md`, then only the task-specific root specs.
5. Inspect implementation files after the relevant standards are clear.

## Required Specs By Task Type

- Agent/workflow changes: `../../../sdkwork-specs/SOUL.md`, `../../../sdkwork-specs/AGENTS_SPEC.md`, `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`, `../../../sdkwork-specs/TEST_SPEC.md`.
- Package script changes: `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`, `../../../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`.
- Any code change: `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`, plus only the touched language/framework spec.
- TypeScript/Node code: `../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md` (loaded on demand).
- Frontend/UI code: `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md`, `../../../sdkwork-specs/UI_ARCHITECTURE_SPEC.md`, `../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md` (loaded on demand).
- SDK integration: `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`, `../../../sdkwork-specs/SDK_SPEC.md`.

Language-specific specs are on-demand; do not load unrelated specs for unrelated tasks.

## Code Style Rules

Read `../../../sdkwork-specs/CODE_STYLE_SPEC.md` and `../../../sdkwork-specs/NAMING_SPEC.md` before code changes. Use generated `@sdkwork/agents-*` SDK clients; do not call managed-agent HTTP with raw fetch wrappers.

Build scripts, dev runners, and `pnpm clean` must follow `CODE_STYLE_SPEC.md` §7 (Build Source Integrity And Self-Healing). Git-tracked build-critical source files must be verified before builds and self-healed from git when missing; `clean` must not delete them.

## Build, Test, and Verification

Run from repository root:

```powershell
pnpm start:desktop
pnpm workflow:build-client-surfaces
pnpm workflow:typecheck-client-surfaces
pnpm verify
```

## Agent Execution Rules

Use dynamic progressive loading and the convention dictionary before broad source loading. Do not hand-edit generated SDK output. Do not duplicate kernel runtime SPI in client packages.

## HTTP API Response Envelope

All L2+ `app-api`, `backend-api`, and SDKWork-owned business `open-api` HTTP contracts `MUST` follow `API_SPEC.md` section 4.5, section 14, and section 15:

- **Input:** typed request bodies, section 14.1 list/search/command input, `SdkWorkListQuery`, and `q` for free-text search.
- **Success output:** `SdkWorkApiResponse` with `{ "code": 0, "data": <payload>, "traceId": "<server-uuid>" }`.
- **Error output:** HTTP 4xx/5xx `application/problem+json` (`ProblemDetail`) with numeric `code` and `traceId`.
- Success `code` is numeric `int32`; HTTP 2xx JSON bodies `MUST` use `0` only. REST semantics remain on HTTP status (`201`, `202`, etc.).
- Platform error codes are numeric non-zero values per section 15.3 (`40001`, `40101`, `40401`, …).
- Single resource: `data.item`
- Lists: `data.items` + `data.pageInfo` (`PageInfo.mode` is `offset` or `cursor`)
- Commands: `data.accepted` plus optional `resourceId` / `status`
- Async accept (`202`): `data.operationId`, `data.status`, optional `pollUrl`

Vendor compatibility `open-api` routes that mirror upstream tool or provider wire (for example OpenAI `/v1/*`, Claude Code, Codex) `MAY` opt out only when every exempt operation declares `x-sdkwork-wire-protocol: external` and `x-sdkwork-external-protocol-id` per `API_SPEC.md` section 4.5.2. SDKWork-owned business `open-api` operations `MUST NOT` opt out.

Errors `MUST` use HTTP 4xx/5xx with `application/problem+json` (`ProblemDetail`) including required numeric `code` and `traceId`. Business failures `MUST NOT` use HTTP 2xx with non-zero `code`, string wire codes, `success`, or human `message`.

Forbidden legacy envelopes and fields: `PlusApiResult`, `AppbaseApiResult`, `StoreApiResult`, `SdkWorkResponse`, per-domain `*ApiResult`, wire field `requestId`, bare domain DTOs at the HTTP root, and top-level `{ items, pageInfo, traceId }` without `data`.

Handlers `MUST` serialize success and map errors through `sdkwork-web-framework` response mapping. Generated HTTP SDKs (`--standard-profile sdkwork-v3`) unwrap `data` by default and expose typed numeric `ProblemDetail.code` / `traceId` on errors; use `.raw` when the full envelope is required.

Before completing API contract, SDK generation, or frontend service work, run:

```bash
node <sdkwork-specs>/tools/check-api-response-envelope.mjs --workspace <workspace-root>
```

Authority: `sdkwork-specs/API_SPEC.md` section 4.5 and sections 14–16, `SDK_SPEC.md` section 4.2, `FRONTEND_SPEC.md`, `MIGRATION_SPEC.md` section 4.2.

## Human Review Rules

Request human review before breaking SDKWork standards, changing public naming, altering security/auth behavior, or changing generated SDK ownership.
