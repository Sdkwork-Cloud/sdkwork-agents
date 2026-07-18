# SDKWork Agents PC

SDKWork Agents PC is a static Vite React application for browser delivery and optional Tauri desktop hosting. Runtime capabilities use composed application SDKs with the appbase IAM runtime and one global TokenManager; the application does not require a Node/Express proxy and browser code never receives provider API secrets.

## Runtime Architecture

- Agents chat and management use `@sdkwork/agents-app-sdk` through `@sdkwork/agents-pc-core` and application-owned service ports.
- The production composition exposes the SDK-backed Agent catalog, lifecycle editor, and `AgentChatView`; stock-data inspiration, generic creative generation, asset gallery, and local prototype workbench modules are not published as production routes.
- Avatar and Agent chat image/attachment/video/voice uploads use `@sdkwork/drive-app-sdk` through the centralized PC core Drive uploader. Media-specific Creative upload policies remain centralized but are inactive until an approved generation API owns that product surface.
- Persistent media identity is `drive://spaces/{spaceId}/nodes/{nodeId}` plus `MediaResource`; 15-minute download URLs are UI previews only.
- Knowledgebase, Skills, and Voice SDKs are optional and initialize only when their Base URLs are configured.
- No application-local upload API, upload table, object-storage provider, raw HTTP authentication header, or local fake-success response is allowed.
- The production bundle gate forbids mock/stock feature composition and keeps initial JavaScript below 260 KiB gzip; the current verified build is approximately 215 KiB gzip.

## Run Locally

```powershell
pnpm install
pnpm --filter @sdkwork/agents-pc dev
```

Copy `.env.example` to the environment-specific source configuration used by the deployment. `VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL` and `VITE_SDKWORK_AGENTS_PC_DRIVE_APP_API_BASE_URL` are public API Base URLs, never secrets.

## Verification

```powershell
pnpm --filter @sdkwork/agents-pc test:agent-contracts
pnpm --filter @sdkwork/agents-pc typecheck
pnpm --filter @sdkwork/agents-pc build
```
