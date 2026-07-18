# Repository Guidelines

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing application tasks. Start with the sections that route the current task; related-spec references are not a startup bundle.

## SDKWORK Standards

The canonical standards index is `../../../sdkwork-specs/README.md`, and `../../../sdkwork-specs/AGENTS_SPEC.md` governs this entrypoint. Read the relevant task-matrix row first and do not copy global normative bodies locally.

## Application Identity

Read `sdkwork.app.config.json` only for application identity, SDK/API inventory, release metadata, packaging, or app-owned capabilities. Runtime values belong to source configuration, not the application declaration.

## Local Dictionary Structure

Use `AGENTS.md` as the application routing entrypoint. Read `.sdkwork/`, `specs/`, application source, tests, and documentation only when the current task reaches the contract each location governs.

## Spec Resolution Order

Use dynamic progressive loading: read this file and `../../AGENTS.md`, then applicable local contracts, then the relevant task route in `../../../sdkwork-specs/README.md`, and only afterward inspect implementation files. Language-specific specs load on demand only.

## Required Specs By Task Type

Code changes load `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`, and only the touched `TYPESCRIPT_CODE_SPEC.md` or `FRONTEND_CODE_SPEC.md`. Language-specific specs are on-demand only. Package-command work loads `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`; packaging workflow work loads `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`.

## Code Style Rules

Consume remote capabilities through composed SDKs and application-owned bootstrap adapters. Use `@sdkwork/utils` for shared helpers. Do not introduce raw HTTP, manual authentication headers, generated transport imports, local SDK forks, or duplicated shared utilities.

## Build, Test, and Verification

Choose the narrowest verification for the changed surface. Run application typecheck, tests, and package build for mini-program changes; run workspace-wide checks only when the change crosses that boundary.

## Agent Execution Rules

Follow specifications before memory and evidence before completion. Keep SDK construction, authentication, runtime environment selection, and host capability bridges in their owning layers.

## Task-Specific Standards

SDK consumer work loads `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md` and runs `check-app-sdk-consumer-imports.mjs`. API work loads `../../../sdkwork-specs/API_SPEC.md` and its validators. List/search work loads `../../../sdkwork-specs/PAGINATION_SPEC.md` and `check-pagination.mjs`. Source configuration work loads `../../../sdkwork-specs/SOURCE_CONFIG_SPEC.md` and `check-source-config-standard.mjs`.

## Human Review Rules

Human review is required for public API changes, security exceptions, database migrations, generated SDK ownership changes, destructive operations, and cross-application standards changes.
