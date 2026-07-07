# SDKWork Agents — Kernel SPI Gap Analysis

- Version: 1.0.0
- Status: active
- Updated: 2026-07-07
- Owner: agents-platform
- Related: [`AGENTS_PROVIDER_TAXONOMY_SPEC.md`](./AGENTS_PROVIDER_TAXONOMY_SPEC.md), [`../../sdkwork-kernel/specs/AGENT_KERNEL_SPEC.md`](../../sdkwork-kernel/specs/AGENT_KERNEL_SPEC.md)

## 1. Executive Summary

`sdkwork-kernel` provides a Linux-kernel-style agent SPI with strong provider plugin
architecture. `sdkwork-agents` correctly owns business persistence and HTTP/SDK surfaces.
**sdkwork-birdcoder** correctly routes agent operations through `sdkwork-agents-runtime-facade`.

Remaining gaps are concentrated in: token-level SSE streaming (G1), kernel-owned
live-interaction SPI projection (G3), T2 autonomous default catalog opt-in (G4),
and commercial GA client surfaces (Flutter / A2).

Pre-launch agents-owned surfaces are complete: **95 HTTP operations** (27 Open / 35
App / 33 Backend), **`ai_agent_task`** + **`ai_agent_interaction`** persistence,
task auto-execution, interaction HTTP on App/Backend, atomic provider-binding
activation, chat payload limits, and inference timeout guards.

## 2. Strengths (Aligned)

| Area | Evidence | Status |
| --- | --- | --- |
| Kernel SPI breadth | 20+ specs under `sdkwork-kernel/specs/` | Strong |
| Provider plugins | codex, claude-code, opencode, gemini-cli, openclaw, hermes, rig | Implemented |
| Agents composition plane | 8 `ai_*` tables (incl. task + interaction), composition slots | Complete |
| HTTP + SDK | 95 operations (27/35/33), SdkWorkApiResponse envelope | Complete |
| BirdCoder boundary | `sdkwork-birdcoder-kernel-bridge` → agents facade only | Enforced |
| Independent modules | `sdkwork-memory`, `sdkwork-knowledgebase`, `sdkwork-skills`, `sdkwork-prompts`, `sdkwork-mcp` via composition slot; `sdkwork-llm` via runtime binding/profile; no table duplication | Correct |
| Pagination | SQL LIMIT/OFFSET, PageInfo | Aligned |
| Audit + IAM policy | Postgres audit, IAM-backed policy | Production |

## 3. Kernel SPI Gaps

### G1 — Token-level streaming (P1)

| Item | Current | Target |
| --- | --- | --- |
| `ModelProvider::stream` | Single SSE `completion` event wrapping full response | Chunked `message.delta` kernel events → agents SSE |
| Spec | `AGENT_MODEL_PROVIDER_SPI_SPEC.md` | Implement stream path in all T1 providers |
| Agents API | `?stream=true` returns one event | Multi-event stream aligned with kernel `KernelEvent` |

**Owner:** sdkwork-kernel providers → sdkwork-agents HTTP SSE adapter.

### G2 — Task scheduling persistence (P1) — **agents layer done**

| Item | Current | Target |
| --- | --- | --- |
| Kernel model | `AgentTask`, `AgentRun`, `AgentStep` defined | Complete |
| Agents DB | `ai_agent_task` + Postgres sync | **Done** |
| HTTP API | `agents.tasks.list/create/retrieve/cancel` (App + Backend) | **Done** |
| Execution | `create_task` defers LLM by default; inline via `metadataJson.autoExecute: true` (legacy `deferExecution`) | **Done** |
| Non-GA scope | `agents.taskRuns.*` projection APIs | Kernel run SPI |

**Owner:** sdkwork-agents (persistence + API — complete); kernel (run/step execution SPI).

### G3 — Live interaction SPI ownership (P1) — **product HTTP done; kernel SPI pending**

| Item | Current | Target |
| --- | --- | --- |
| Agents HTTP | `agents.interactions.*` on App/Backend (Open API excluded) | **Done** |
| Persistence | `ai_agent_interaction` Postgres + in-memory repository | **Done** |
| OpenCode permission / Q&A | `sdkwork-agents-runtime-facade/live_interaction.rs` | Kernel `agent.live_interaction` SPI per `KERNEL_PRODUCT_PROJECTION_SPEC.md` |
| BirdCoder bridge | delegates to agents facade | Unchanged consumer path |

### G4 — Extended autonomous engines in default catalog (P2)

| Item | Current | Target |
| --- | --- | --- |
| openclaw, hermes | Bootstrap in `code_engines.rs` but excluded from `CANONICAL_CODE_ENGINE_KEYS` | Opt-in bootstrap flag or tier-2 catalog section after CI conformance |
| Catalog API | 4 engines | Document `tier` field; expose T2 when healthy |

### G5 — Rig framework live backend (P2)

| Item | Current | Target |
| --- | --- | --- |
| `sdkwork-agent-provider-rig` | Fail-closed default backend | Feature-gated live backend mapping Rig APIs |
| Agents | `implementation_type=rig-rust` enum exists | Provider binding activation path |

### G6 — MCP federation HTTP (P2)

| Item | Current | Target |
| --- | --- | --- |
| MCP marketplace | Agents projection from composition slots | Federated list from `sdkwork-mcp` mount |
| Kernel MCP SPI | `AGENT_MCP_PROVIDER_SPI_SPEC.md` | Wire slot resolution to MCP sibling SDK |

## 4. Agents Application Gaps

### A1 — Task run projection APIs (non-GA scope)

`agents.tasks.*` lifecycle is shipped. Non-GA scope is kernel-aligned
`agents.taskRuns.*` list/retrieve for deep scheduler dashboards.

### A2 — Flutter / Dart SDK (non-GA scope)

PC/H5/MP are production-ready; Flutter remains scaffold-only until an owned Dart app SDK facade exists.

### A3 — Token streaming UX

Chat UI receives full assistant message at once; depends on G1.

### A4 — Grafana / ops dashboards

Metrics are exposed through `/metrics/agents`; dashboard wiring is owned by ops and is outside the current GA evidence bundle.

## 5. BirdCoder Integration Gaps

### B1 — Live interaction target owner

Documented transition: bridge → kernel SPI; agents facade remains product entry.

### B2 — Engine catalog parity

BirdCoder workbench should read `agents.codeEngines.list` (done); T2 engines opt-in.

### B3 — Task ↔ coding_session correlation

Coding sessions are BirdCoder-owned; `ai_agent_task` carries optional `external_ref`
for `coding_session_id` cross-link while deeper run/step views wait for
`ai_agent_task_run`.

## 6. Commercial Readiness Assessment

| Criterion | Ready | Blocker / follow-up |
| --- | --- | --- |
| Multi-tenant agent CRUD | Yes | — |
| IAM auth + audit | Yes | — |
| Hosted chat (PC/H5) | Yes | Token streaming (G1) for premium UX |
| Code-engine multi-provider | Yes (T1 ×4) | T2 opt-in (G4) |
| Open/App/Backend SDK | Yes | Flutter Dart SDK (A2) |
| BirdCoder unified management | Yes | Task run APIs (A1) for deep scheduler UI |
| Independent module composition | Yes | Mount `sdkwork-memory`, `sdkwork-knowledgebase`, `sdkwork-skills`, `sdkwork-prompts`, `sdkwork-mcp`, `sdkwork-llm`, and `sdkwork-drive` SDK/runtime endpoints in deployment |
| Security fail-closed | Yes | — |
| Observability | Partial | Grafana (A4) |
| App store GA metadata | No | PRD Phase 4 item |

**Verdict:** Commercial MVP for PC/H5 + BirdCoder code-agent workflows is **ready**.
Full GA requires Flutter SDK (A2), token streaming (G1), task-run projection APIs (A1),
and store metadata.

## 7. Improvement Roadmap

| Phase | Items | Owner |
| --- | --- | --- |
| P0 (done) | Composition plane, 95 APIs, tasks + interactions HTTP, facade, BirdCoder boundary | agents + birdcoder |
| P1 | G1 streaming, G3 kernel live-interaction SPI, task-run projection (A1) | kernel + agents |
| P2 | G4 T2 catalog, G5 Rig live, G6 MCP federation, A2 Flutter SDK | kernel + agents + siblings |
| P3 | Grafana, app store GA, cursor/keyset message pagination | ops + agents |

## 8. Verification Loop

Repeat until all gate tasks in `specs/agents-birdcoder-alignment.spec.json` pass:

```powershell
# sdkwork-agents
pnpm verify
cargo test -p sdkwork-agents-runtime-facade
cargo test -p sdkwork-intelligence-agents-service --features http-axum

# sdkwork-birdcoder
node scripts/birdcoder-agents-integration-contract.test.mjs
cargo test -p sdkwork-birdcoder-kernel-bridge

# sdkwork-kernel (from kernel root)
cargo test -p sdkwork-agent-provider-codex
cargo test -p sdkwork-agent-provider-rig
```
