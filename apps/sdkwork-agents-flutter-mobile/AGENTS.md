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

Read `sdkwork.app.config.json` when changing Flutter mobile behavior, runtime config, or SDK wiring. Managed-agent APIs are owned by repository root `sdkwork-agents`.

## Local Dictionary Structure

- `AGENTS.md`, `sdkwork.app.config.json`, `specs/`, `lib/`, `pubspec.yaml`.

## Spec Resolution Order

Use dynamic progressive loading:

1. Read this `AGENTS.md` and `../../../AGENTS.md`.
2. Read `sdkwork.app.config.json` when app behavior is touched.
3. Read local `specs/` when relevant.
4. Read `../../../sdkwork-specs/README.md`, then task-specific specs only.
5. Inspect implementation files last.

## Required Specs By Task Type

- Agent/workflow: `../../../sdkwork-specs/SOUL.md`, `../../../sdkwork-specs/AGENTS_SPEC.md`, `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`.
- Code: `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`.
- Dart/Flutter: follow repository Flutter conventions (on demand).
- Frontend/mobile UI: `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md` (on demand).

Language-specific specs are on-demand only.

## Code Style Rules

Follow SDKWork naming and module boundaries. Do not duplicate kernel runtime SPI.

## Build, Test, and Verification

```powershell
cd apps/sdkwork-agents-flutter-mobile
flutter test
pnpm verify
```

## Agent Execution Rules

Use dynamic progressive loading. Do not hand-edit generated SDK output.

## Human Review Rules

Human review required for breaking API, auth, and SDK ownership changes.
