# Developer Guide

Local setup and verification for the Agents application root.

## Prerequisites

- Rust stable toolchain
- Node.js 22 + pnpm 10 (see root `package.json` `packageManager`)
- Sibling checkouts: `sdkwork-kernel`, `sdkwork-specs`, `sdkwork-web-framework`, `sdkwork-database`, `sdkwork-utils`, `sdkwork-app-topology`

## Install

```powershell
cd sdkwork-agents
pnpm install
```

## Verification (required before merge)

```powershell
pnpm verify
```

`pnpm verify` runs:

1. `pnpm check` — composition, architecture, identity, API envelope, deploy, docs, workflow standards, topology, database
2. `pnpm workflow:build-agents-app-sdk` — generated TypeScript dist matches OpenAPI
3. `cargo build --workspace`
4. Rust workspace tests (`cargo test --workspace --all-features`, HTTP + Postgres contract suites)
5. `pnpm --filter @sdkwork/agents-mini-program build` — mini-program runtime bundle
6. `pnpm check:contracts` — platform integration, database framework, Open SDK surface, mini-program runtime
7. PC / H5 / mini-program TypeScript typecheck
8. PC agent contracts — scope, management profile, chat service, e2e flow (create → chat)

Full staging/production checklist: [runbooks/pre-launch-verification.md](../../runbooks/pre-launch-verification.md).

Narrow checks:

```powershell
pnpm check
pnpm topology:validate
pnpm db:validate
pnpm workflow:typecheck-client-surfaces
pnpm --filter @sdkwork/agents-pc test:agent-contracts
```

## Client surfaces

```powershell
pnpm start:desktop          # PC Vite dev
pnpm start:browser          # H5 Vite dev
pnpm start:mini-program     # WeChat mini program build/watch
pnpm workflow:build-client-surfaces
```

After changing PC `*-core` SDK or session modules, sync H5:

```powershell
pnpm workflow:sync-agent-h5-from-pc
```

## SDK boundary

Generated app SDKs (`@sdkwork/agents-app-sdk`, `@sdkwork/knowledgebase-app-sdk`) are consumed only from `*-core/sdk` exports. Capability packages (`*-agents`) must not declare `@sdkwork/agents-app-sdk` directly. See [specs/README.md](../../specs/README.md).

## Canon

- [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md)
- [TECH-api-specification.md](../../architecture/tech/TECH-api-specification.md)
- [specs/README.md](../../specs/README.md)
