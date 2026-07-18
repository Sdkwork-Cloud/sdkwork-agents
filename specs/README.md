# SDKWork Agents Local Specifications

Application narrowing rules for the hosted Agents product. Agent kernel SPI contracts
remain authoritative under `../sdkwork-kernel/specs/`.

## Architecture Authority

| Document | Purpose |
| --- | --- |
| [AGENTS_KERNEL_BOUNDARY_SPEC.md](./AGENTS_KERNEL_BOUNDARY_SPEC.md) | Kernel vs agents vs product boundary (frozen) |
| [AGENTS_PROVIDER_TAXONOMY_SPEC.md](./AGENTS_PROVIDER_TAXONOMY_SPEC.md) | Code / autonomous / framework agent taxonomy |
| [AGENTS_KERNEL_SPI_GAP_ANALYSIS.md](./AGENTS_KERNEL_SPI_GAP_ANALYSIS.md) | SPI gaps, commercial readiness, roadmap |
| [agents-birdcoder-alignment.spec.json](./agents-birdcoder-alignment.spec.json) | Machine-readable cross-repo alignment tracker |
| [docs/architecture/AGENTS_LAYERING.md](../docs/architecture/AGENTS_LAYERING.md) | Crate and SDK layering |
| [docs/product/prd/PRD.md](../docs/product/prd/PRD.md) | Product requirements |
| [docs/architecture/tech/TECH_ARCHITECTURE.md](../docs/architecture/tech/TECH_ARCHITECTURE.md) | Technical architecture |

## Platform Framework Alignment

| Framework | Status | Integration point |
| --- | --- | --- |
| `sdkwork-web-framework` | **Integrated** | `sdkwork-routes-agents-*` + `build_served_combined_router` in kernel-bridge |
| `sdkwork-database` | **Integrated** | `database/` assets, `sdkwork-agents-database-host`, managed-store postgres path |
| `sdkwork-utils` | **Integrated** | `SdkWorkApiResponse`, `parse_bool`, `is_blank`, `trim`, `uuid` across contract/service/facade |
| `sdkwork-drive` | **Integrated for PC upload scope** | PC core uses `@sdkwork/drive-app-sdk` Drive Uploader; Agents messages and slots retain canonical Drive-backed media references only |
| `sdkwork-discovery` | **Inactive** | No first-party RPC services yet |

## Independent Module Integration

`sdkwork-agents` consumes independent capability modules; those modules do not
depend on `sdkwork-agents` for their core domain behavior.

| Module | Integration mode | SDK / contract |
| --- | --- | --- |
| `sdkwork-memory` | `slot_kind=memory` composition slot | `@sdkwork/memory-app-sdk` (when mounted) |
| `sdkwork-knowledgebase` | `slot_kind=knowledge`, `target_module=knowledgebase` | `@sdkwork/knowledgebase-app-sdk` |
| `sdkwork-skills` | `slot_kind=skill`, `target_module=skills` | `@sdkwork/skills-app-sdk` |
| `sdkwork-prompts` | `slot_kind=prompt`, `target_module=prompts` | `@sdkwork/prompts-app-sdk` |
| `sdkwork-mcp` | `slot_kind=mcp`, `target_module=mcp` | marketplace projection; federation enablement requires an approved sdkwork-mcp runtime surface |
| `sdkwork-llm` | runtime binding / model provider profile | model catalog, provider profile, credential references |
| `sdkwork-drive` | `slot_kind=drive`, `target_module=drive` | `@sdkwork/drive-app-sdk`; Drive Uploader only |

Do not reverse the dependency: memory, knowledgebase, skills, prompts, mcp, llm,
and drive own their tables, APIs, SDKs, and runtime contracts. Agents stores only
references and orchestration policy. The root `specs/component.spec.json` declares
these entries as `dependencyMode=independent-capability-module` with
`reverseDependencyPolicy=forbidden`, and `pnpm check:architecture-alignment`
validates that the direction remains unchanged.

## Kernel Dependency

Runtime composition uses sibling checkout `../sdkwork-kernel` per `DEPENDENCY_MANAGEMENT_SPEC.md`.

Products (including `sdkwork-birdcoder`) MUST consume agent runtime through
`sdkwork-agents-runtime-facade` and `@sdkwork/agents-app-sdk`, not `sdkwork-agent-provider-*`.

Client composition authority: `APP_COMPOSITION_SPEC.md` via `pnpm check:app-composition` (`verify-repo.mjs`). Do not add `dependency.composition.json`.

Client SDK boundary: only `*-core` packages may depend on generated app SDKs (`@sdkwork/agents-app-sdk`, `@sdkwork/knowledgebase-app-sdk`). Capability packages and app shells consume types and clients through `*-core/sdk` exports. Enforced by `pnpm check:architecture-alignment`.

## Verification

```powershell
pnpm verify
pnpm check
pnpm check:api-operation-patterns
pnpm check:route-path-collisions
pnpm check:pagination
pnpm check:app-sdk-consumer-imports
pnpm check:agent-sdk-workspace
pnpm check:component-port-bindings
pnpm check:frontend-composition
pnpm check:permission-composition
pnpm check:composition-resolver
pnpm check:rust-backend-composition
pnpm check:production-security
pnpm gateway:validate:cloud
node ../sdkwork-birdcoder/scripts/birdcoder-agents-integration-contract.test.mjs
```
