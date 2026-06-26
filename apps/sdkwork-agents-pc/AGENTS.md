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

## Human Review Rules

Request human review before breaking SDKWork standards, changing public naming, altering security/auth behavior, or changing generated SDK ownership.
