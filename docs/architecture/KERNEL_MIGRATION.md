# Kernel → Agents Migration

Status: complete (2026-06-26)

## What moved

| Was (`sdkwork-kernel`) | Now (`sdkwork-agents`) |
| --- | --- |
| `sdkwork-agent-business` | `crates/sdkwork-intelligence-agents-service` |
| `crates/sdkwork-routes-agent-*-api` | `crates/sdkwork-routes-agents-*-api` |
| `apis/agent-business/` | `apis/agents/` |
| `sdks/sdkwork-agent-*-sdk/` | `sdks/sdkwork-agents-*-sdk/` |

## What stayed in kernel

- `sdkwork-agent-kernel` — runtime SPI
- `sdkwork-agent-server` — operational HTTP + internal runtime API
- `sdkwork-agent-database` — **runtime session** persistence only
- `crates/sdkwork-routes-agent-internal-{manifest,api}`
- `sdks/sdkwork-agent-internal-sdk`

## Layering rule

Kernel = mechanism (Linux-kernel style).  
Agents application = managed-agent policy, CRUD, marketplace, knowledge/memory metadata HTTP, SDKs.

See also:

- [`AGENTS_LAYERING.md`](./AGENTS_LAYERING.md)
- [`../sdkwork-kernel/docs/architecture/decisions/ADR-20260626-agents-application-layer-separation.md`](../sdkwork-kernel/docs/architecture/decisions/ADR-20260626-agents-application-layer-separation.md)

## Verification

```powershell
# Application
cd sdkwork-agents
pnpm verify

# Kernel (runtime SPI only)
cd sdkwork-kernel
node scripts/check-kernel-standards.mjs
cargo build --workspace
```
