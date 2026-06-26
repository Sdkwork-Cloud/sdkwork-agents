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

Read `sdkwork.app.config.json` only when changing H5 application behavior, runtime config, SDK wiring, or release metadata. Managed-agent APIs are owned by repository root `sdkwork-agents`.

## Local Dictionary Structure

- `AGENTS.md`: local application agent entrypoint.
- `sdkwork.app.config.json`: H5 application identity.
- `specs/`, `packages/`, `src/`, `package.json`.

## Spec Resolution Order

Use dynamic progressive loading:

1. Read this `AGENTS.md` and `../../../AGENTS.md`.
2. Read `sdkwork.app.config.json` when app behavior or SDK wiring is touched.
3. Read local `specs/` only when relevant.
4. Read `../../../sdkwork-specs/README.md`, then task-specific root specs only.
5. Inspect implementation files last.

## Required Specs By Task Type

- Agent/workflow: `../../../sdkwork-specs/SOUL.md`, `../../../sdkwork-specs/AGENTS_SPEC.md`, `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`.
- Package scripts: `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`.
- Code changes: `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`.
- TypeScript: `../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md` (on demand).
- Frontend/H5: `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md`, `../../../sdkwork-specs/FRONTEND_SPEC.md` (on demand).

Language-specific specs are on-demand only.

## Code Style Rules

Follow `CODE_STYLE_SPEC.md` and `NAMING_SPEC.md`. Consume `@sdkwork/agents-app-sdk` through generated clients.

## Build, Test, and Verification

```powershell
pnpm start:browser
pnpm workflow:build-client-surfaces
pnpm verify
```

## Agent Execution Rules

Use dynamic progressive loading. Do not hand-edit generated SDK output or duplicate kernel runtime logic.

## Human Review Rules

Human review required for breaking API changes, auth behavior changes, and SDK ownership changes.
