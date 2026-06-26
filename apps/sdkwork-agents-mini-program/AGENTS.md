# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing tasks in this application root.

## SDKWORK Standards

- `../../../sdkwork-specs/README.md`
- `../../../sdkwork-specs/SOUL.md`
- `../../../sdkwork-specs/AGENTS_SPEC.md`
- `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`
- `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`
- `../../../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../../../sdkwork-specs/NAMING_SPEC.md`

## Application Identity

Read `sdkwork.app.config.json` when changing mini program behavior, runtime config, or SDK wiring. Managed-agent APIs are owned by repository root `sdkwork-agents`.

## Local Dictionary Structure

- `AGENTS.md`, `sdkwork.app.config.json`, `specs/`, `packages/`, `src/`, `package.json`.

## Spec Resolution Order

Use dynamic progressive loading:

1. Read this `AGENTS.md` and `../../../AGENTS.md`.
2. Read `sdkwork.app.config.json` when app behavior is touched.
3. Read local `specs/` when relevant.
4. Read `../../../sdkwork-specs/README.md`, then only task-specific specs.
5. Inspect implementation files last.

## Required Specs By Task Type

- Agent/workflow: `../../../sdkwork-specs/SOUL.md`, `../../../sdkwork-specs/AGENTS_SPEC.md`, `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`.
- Package scripts: `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`.
- Code: `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`.
- TypeScript: `../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md` (on demand).
- Mini program UI: `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md` (on demand).

Language-specific specs are on-demand only.

## Code Style Rules

Follow root code style specs. Use generated SDK clients for managed-agent APIs.

## Build, Test, and Verification

```powershell
pnpm start:mini-program
pnpm workflow:build-client-surfaces
pnpm verify
```

## Agent Execution Rules

Use dynamic progressive loading. Do not duplicate kernel runtime SPI in mini program packages.

## Human Review Rules

Human review required for breaking API, auth, and SDK ownership changes.
