# SDKWork Agents Technical Architecture

Status: active  
Owner: agents-platform  
Updated: 2026-06-26

## 1. Architecture Overview

SDKWork Agents is a hosted agent product application that composes `sdkwork-kernel`
for agent runtime SPI, HTTP route surfaces (`/agent/v3/api`, `/app/v3/api`, `/backend/v3/api`),
and runtime persistence. This repository owns deployment topology, packaging, application
metadata database, and verification boundaries.

## 2. Platform Integration

| Framework | Role |
| --- | --- |
| `sdkwork-kernel` | Agent runtime, route crates, internal runtime API, session persistence |
| `sdkwork-web-framework` | Mandatory HTTP interceptor chain via kernel `build_served_combined_router` |
| `sdkwork-database` | Application metadata module (`agents_*` tables) + kernel runtime DB |
| `sdkwork-utils` | Shared env parsing in `sdkwork-agents-contract` |
| `sdkwork-drive` | Required for all file upload features (Drive Uploader only) |
| `sdkwork-discovery` | Deferred until RPC services ship |

## 3. Crate Layout

```text
crates/
  sdkwork-agents-contract/       # runtime env helpers (utils)
  sdkwork-agents-kernel-bridge/  # kernel composition boundary
  sdkwork-agents-api-server/     # runnable HTTP server binary
```

## 4. Verification

```powershell
pnpm verify
pnpm check:architecture-alignment
```
