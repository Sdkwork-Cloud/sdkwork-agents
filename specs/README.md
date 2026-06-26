# SDKWork Agents Local Specifications

Application narrowing rules for the hosted Agents product. Agent kernel SPI contracts
remain authoritative under `../sdkwork-kernel/specs/`.

## Platform Framework Alignment

| Framework | Status | Integration point |
| --- | --- | --- |
| `sdkwork-web-framework` | **Integrated** | Kernel route crates + `build_served_combined_router` in `sdkwork-agents-kernel-bridge` |
| `sdkwork-database` | **Integrated** | Workspace deps; app metadata in `database/`; kernel runtime DB via `sdkwork-agent-server` |
| `sdkwork-utils` | **Integrated** | `sdkwork-agents-contract` uses `sdkwork-utils-rust::parse_bool` |
| `sdkwork-drive` | **Required for uploads** | File upload must use Drive Uploader per `DRIVE_SPEC.md` |
| `sdkwork-discovery` | **Deferred** | No first-party RPC services yet |

## Kernel Dependency

Runtime composition uses sibling checkout `../sdkwork-kernel` per `DEPENDENCY_MANAGEMENT_SPEC.md`.
