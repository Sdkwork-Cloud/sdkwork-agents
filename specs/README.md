# SDKWork Agents Local Specifications

Application narrowing rules for the hosted Agents product. Agent kernel SPI contracts
remain authoritative under `../sdkwork-kernel/specs/`.

## Platform Framework Alignment

| Framework | Status | Integration point |
| --- | --- | --- |
| `sdkwork-web-framework` | **Integrated** | `sdkwork-routes-agents-*` + `build_served_combined_router` in kernel-bridge |
| `sdkwork-database` | **Integrated** | `database/` assets, `sdkwork-agents-database-host`, managed-store postgres path |
| `sdkwork-utils` | **Integrated** | `SdkWorkApiResponse`, `parse_bool`, `is_blank`, `trim`, `uuid` across contract/service/facade |
| `sdkwork-drive` | **Declared; upload deferred** | Composition slots reference drive; Uploader wiring when upload ships |
| `sdkwork-discovery` | **Deferred** | No first-party RPC services yet |

Client composition authority: `APP_COMPOSITION_SPEC.md` via `pnpm check:app-composition` (`verify-repo.mjs`). Do not add `dependency.composition.json`.

Client SDK boundary: only `*-core` packages may depend on generated app SDKs (`@sdkwork/agents-app-sdk`, `@sdkwork/knowledgebase-app-sdk`). Capability packages and app shells consume types and clients through `*-core/sdk` exports. Enforced by `pnpm check:architecture-alignment`.

## Kernel Dependency

Runtime composition uses sibling checkout `../sdkwork-kernel` per `DEPENDENCY_MANAGEMENT_SPEC.md`.
