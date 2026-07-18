# Repository Guidelines

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing application tasks. Start with the sections that route the current task; do not treat related-spec references as a startup bundle.

## SDKWORK Standards

The canonical global standards index is `../../../sdkwork-specs/README.md`; `../../../sdkwork-specs/AGENTS_SPEC.md` governs this entrypoint. Read the relevant task-matrix row first, then only the selected authority. Do not copy global `*_SPEC.md` bodies locally.

## Application Identity

Read `sdkwork.app.config.json` for application identity, SDK/API inventory, release metadata, packaging, or app-owned capabilities. Runtime environment values come from the application configuration adapters and repository deployable-root `../../etc/`, not the app manifest.

## Local Dictionary Structure

- `specs/` and nearest package `specs/component.spec.json`: application and component contracts.
- `packages/`: feature, core, composed service, and desktop packages.
- `src/`: application composition root and runtime bootstrap.
- `config/`: desktop and server runtime adapters.
- `tests/`: application contract and architecture tests.

## Spec Resolution Order

Use dynamic progressive loading: read this file and `../../AGENTS.md`, then the applicable local contract, then the relevant row in `../../../sdkwork-specs/README.md`, and only then inspect implementation files. Load language-specific standards on demand only.

## Required Specs By Task Type

Code changes load `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`, and only the touched `TYPESCRIPT_CODE_SPEC.md`, `FRONTEND_CODE_SPEC.md`, or `RUST_CODE_SPEC.md`. Language-specific specs are on-demand only. Package commands load `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`; packaging workflow changes load `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`. SDK integration loads `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`; uploads load `../../../sdkwork-specs/DRIVE_SPEC.md`.

## Code Style Rules

Consume remote capabilities only through composed application SDKs declared by `specs/component.spec.json`. Use `@sdkwork/utils` shared helpers, the application TokenManager, and bootstrap-owned client construction. UI and feature packages must not create SDK clients, raw HTTP transports, authentication headers, local upload providers, or fake-success fallbacks.

## Build, Test, and Verification

Choose the narrowest checks for the changed surface, then run application typecheck, contract tests, and build before completion. Run workspace-wide `pnpm check` and `pnpm verify` only when the change crosses the application boundary.

## Agent Execution Rules

The PC application is a static Vite client with optional Tauri desktop hosting. Agents, IAM, Drive, and other remote capabilities use composed SDKs; Drive uploads use the Drive Uploader and persist canonical `drive://` resources only. Rust agent runtime SPI and persistence remain owned by `../../../sdkwork-kernel/`. Do not introduce a Node proxy, provider secrets in the browser, generated transport imports, or duplicated kernel logic.

## Task-Specific Standards

API work loads `../../../sdkwork-specs/API_SPEC.md` and its validators. List/search work loads `../../../sdkwork-specs/PAGINATION_SPEC.md` and `check-pagination.mjs`. Source configuration work loads `../../../sdkwork-specs/SOURCE_CONFIG_SPEC.md` and `check-source-config-standard.mjs`. SDK consumer work runs `check-app-sdk-consumer-imports.mjs`.

## Human Review Rules

Human review is required for public API changes, security exceptions, database migrations, generated SDK ownership changes, destructive operations, and kernel dependency boundary changes.
