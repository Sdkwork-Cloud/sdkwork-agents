# sdkwork-agents PRD

Status: active
Owner: agents-platform
Application: sdkwork-agents
Updated: 2026-07-18
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Add `PRD-<topic>.md` shards in this directory when the PRD grows beyond one reviewable screen.

## 1. Background And Problem

企业级智能体平台需要管理大量 AI Agent 的全生命周期：从定义、配置、绑定模型供应商、
部署发布到运行时编排。行业主流方案（Codex、Claude Code、OpenCode、OpenClaw、Hermes、
Rig、LangGraph 等）各自封闭，缺乏统一的组合编排能力与可插拔内核抽象。

### 架构定位

| 模块 | 角色 | 类比 |
| --- | --- | --- |
| `sdkwork-kernel` | 智能体运行时内核 — SPI、Provider 插件、对象模型 | Linux Kernel |
| `sdkwork-agents` | 托管智能体业务应用 — 数据库、API、SDK、组合编排 | 发行版 / 业务层 |
| `sdkwork-birdcoder` | 多代码智能体集成产品 — 通过 agents 消费内核能力 | 专业 IDE 应用 |

产品定位：`sdkwork-agents` 是 SDKWork 的 **Agent Business Plane**。它把
`sdkwork-kernel` 的 SPI、Provider 插件和运行机制封装为可商业化交付的业务模块，
对外提供稳定的数据库模型、Open/App/Backend API、SDK、审计、会话、消息、任务和组合编排。
它不是第二个 kernel，也不拥有外部内容域；外部能力由独立模块封装，
`sdkwork-agents` 只通过组合槽、runtime binding、SDK 或 facade 消费这些模块。

sdkwork-agents 解决的核心问题：

1. **Agent 身份与生命周期管理** — 统一管理 Agent 的创建、配置、发布、下线。
2. **运行时绑定与可回溯配置** — 将 Agent 绑定到模型供应商和运行时实现，并通过
   manifest、默认任务意图和 active binding 形成可审计的运行快照。
3. **跨模块组合编排** — Agent 需要调用 memory、knowledgebase、skills、prompts、
   mcp、llm 等能力，但这些能力由独立模块拥有，不能在 agents 模块内重复实现。
4. **审计与可观测** — 所有管理操作必须可审计、可追踪。
5. **可插拔代码/自主智能体** — 通过 kernel Provider SPI 统一接入 Codex、Claude Code、
   OpenCode（T1 代码智能体）以及 OpenClaw、Hermes（T2 自主智能体）、Rig（T3 框架）。
6. **产品隔离** — BirdCoder 等应用仅通过 agents HTTP/SDK 和 runtime-facade 访问运行时，
   禁止直连 kernel provider crate。

## 2. Target Users

| 用户角色 | 使用场景 |
| --- | --- |
| 平台管理员 | 配置租户、应用注册、Agent 部署审核 |
| 业务开发者 | 创建 Agent、配置 manifest、绑定供应商 |
| 应用集成方 | 通过 Open API / App SDK 调用 Agent 能力 |
| 运维团队 | 监控 Agent 运行状态、审计日志、部署回滚 |
| BirdCoder 产品团队 | 通过统一 agents facade 管理 Codex、Claude Code、OpenCode 等代码智能体 |
| 安全与合规团队 | 审核 provider 绑定、组合槽引用、工具审批、审计与数据边界 |

## 3. Goals And Non-Goals

### Goals

- 提供完整的 Agent 全生命周期管理（创建、更新、删除、恢复、状态变更）
- 支持多供应商运行时绑定（manifest-only、typed-local-provider、process-adapter、protocol-adapter）
- 通过组合槽 (composition slot) 统一引用外部模块资源（memory、knowledgebase、skills、prompts、MCP），通过 runtime binding / provider profile 引用 LLM 能力
- 提供托管会话/消息持久化、实时交互、计划任务与 chat turn API（95 个 HTTP 操作）
- 通过 `sdkwork-agents-runtime-facade` 暴露 code-engine catalog 与 turn 执行
- 通过 manifest、runtime binding、composition slot 和 audit event 支持运行配置回溯
- 提供不可变审计日志
- 支持多租户隔离与组织级权限
- 保证 BirdCoder、PC/H5/小程序/Flutter 等产品通过 composed SDK 或 runtime facade 访问 agents 能力
- 对齐商业化落地需要的 API、SDK、可观测性、失败关闭、审计和 provider 可插拔扩展
- 遵循 sdkwork-specs 标准，使用 ai_ 表前缀

### Non-Goals

- 不实现 MCP 服务器管理（由 `sdkwork-mcp` 拥有）
- 不实现知识库 / RAG（由 `sdkwork-knowledgebase` 拥有）
- 不实现记忆系统（由 `sdkwork-memory` 拥有）
- 不实现技能管理（由 `sdkwork-skills` 拥有）
- 不实现提示词管理（由 `sdkwork-prompts` 拥有）
- 不实现 LLM 模型目录、模型供应商凭证、模型网关或模型运行面（由 `sdkwork-llm` / kernel provider 拥有）
- 不在 agents 域实现对象存储、上传会话或本地上传 API；PC 上传通过 `sdkwork-drive` Drive Uploader，agents 只保存 canonical Drive 资源引用和编排元数据
- 不定义 kernel Provider SPI（由 `sdkwork-kernel` 拥有）
- 不实现 coding-session / workbench 投影（由 `sdkwork-birdcoder` 拥有）
- 不在 agents 内重复实现 Codex/Claude/OpenCode 官方 SDK 绑定（由 kernel provider 插件拥有）
- 不在 kernel `AgentRun` 投影稳定前创建 `ai_agent_task_run` 或 `agents.taskRuns.*` 权威 API

### Kernel vs Agents 数据边界

| 数据 | 所有者 | 说明 |
| --- | --- | --- |
| Agent 身份、绑定、组合槽、审计 | `sdkwork-agents` (`ai_*`) | 产品权威 |
| 托管 chat 会话与消息 | `sdkwork-agents` (`ai_agent_session`, `ai_agent_message`) | 产品查询权威 |
| 实时交互（审批/问答） | `sdkwork-agents` (`ai_agent_interaction`) | 持久化权威 |
| Provider 执行态 (run/step) | `sdkwork-kernel` | 机制；`agents.taskRuns.*` 投影待 kernel SPI |
| 计划任务 | `sdkwork-agents` (`ai_agent_task`) | 产品权威；`agents.tasks.*` 已上线 |
| 记忆内容 | `sdkwork-memory` | 通过 composition slot 引用 |
| 知识库内容 / RAG 索引 | `sdkwork-knowledgebase` | 通过 `knowledge` composition slot 引用 |
| 技能定义与技能包 | `sdkwork-skills` | 通过 `skill` composition slot 引用 |
| 提示词资产与版本 | `sdkwork-prompts` | 通过 `prompt` composition slot 引用 |
| MCP 服务器与工具市场 | `sdkwork-mcp` | 通过 `mcp` composition slot / marketplace projection 引用 |
| LLM 模型、供应商、凭证、模型网关 | `sdkwork-llm` / `sdkwork-kernel` provider | 通过 runtime binding、configuration profile、model provider 引用 |

## 4. Scope

### Core Owned Tables (8 tables, all `ai_` prefix)

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Agent 身份、manifest 快照、生命周期、可见性 |
| `ai_agent_runtime_binding` | 供应商/运行时绑定 |
| `ai_agent_composition_slot` | Agent → 外部模块资源引用（记忆、知识、技能、提示词、文件、MCP） |
| `ai_agent_audit_event` | 不可变管理审计日志 |
| `ai_agent_session` | 托管 chat 会话（tenant/agent/owner 作用域） |
| `ai_agent_message` | 会话消息与 chat turn 持久化 |
| `ai_agent_interaction` | 实时交互（code-engine 审批流） |
| `ai_agent_task` | 计划任务（kernel `AgentTask` 投影） |

### Agent Provider Taxonomy

| Tier | Family | engine_key / type | Catalog default |
| --- | --- | --- | --- |
| T1 | 代码智能体 | `codex`, `claude-code`, `gemini`, `opencode` | Yes (`codeEngines.list`) |
| T2 | 自主智能体 | `openclaw`, `hermes` | Opt-in bootstrap |
| T3 | 框架智能体 | `rig-rust` (`implementation_type`) | Via binding, not code-engine catalog |
| T4 | 编排框架 | `langgraph`, `crewai`, … | Declared enum; provider binding requires an approved provider manifest and SDK integration before enablement |

详见 [`specs/AGENTS_PROVIDER_TAXONOMY_SPEC.md`](../../../specs/AGENTS_PROVIDER_TAXONOMY_SPEC.md)。

### App-only Runtime Catalog APIs

| Method | Path | operationId |
| --- | --- | --- |
| GET | `/app/v3/api/ai/code_engines` | `agents.codeEngines.list` |
| GET | `/app/v3/api/ai/mcp_servers` | `agents.mcpServers.list` |

完整 API 列表见 [TECH-api-specification.md](../../architecture/tech/TECH-api-specification.md)（95 HTTP 操作：27 Open / 35 App / 33 Backend）。

### Independent Module Dependencies

| Module | Owned capability | Agents integration mode | Dependency direction |
| --- | --- | --- | --- |
| `sdkwork-memory` | 永久记忆、用户记忆、成长型记忆、记忆检索/写入 | `slot_kind=memory` composition slot + memory app SDK when mounted | agents → memory |
| `sdkwork-knowledgebase` | 知识库空间、RAG 索引、知识文档检索 | `slot_kind=knowledge`, `target_module=knowledgebase` + knowledgebase app SDK | agents → knowledgebase |
| `sdkwork-skills` | 技能定义、技能包、技能市场、技能调用元数据 | `slot_kind=skill`, `target_module=skills` + skills app SDK | agents → skills |
| `sdkwork-prompts` | Prompt 模板、系统提示词、版本化提示词资产 | `slot_kind=prompt`, `target_module=prompts` + prompts app SDK | agents → prompts |
| `sdkwork-mcp` | MCP server、工具目录、MCP marketplace/federation | `slot_kind=mcp`, `target_module=mcp`; agents 只做 marketplace projection / future federation | agents → mcp |
| `sdkwork-llm` | LLM 模型目录、供应商配置、模型网关、凭证引用 | `ai_agent_runtime_binding` + `configuration_profile_id` + kernel model provider profile | agents → llm / kernel provider |
| `sdkwork-drive` | 文件上传、对象存储、资源下载、Drive Uploader | `slot_kind=drive`, `target_module=drive`; upload 使用 Drive Uploader | agents → drive |

Dependency rule: the independent modules above MUST NOT depend on `sdkwork-agents`.
They expose their own APIs, SDKs, schemas, tables, and runtime contracts. `sdkwork-agents`
stores only references (`slot_kind`, `target_module`, `target_ref`, provider/profile ids)
and orchestration policy; it does not own or duplicate their business data.
Machine-readable constraint: `specs/component.spec.json#contracts.sdkDependencies` declares
these modules with `dependencyMode=independent-capability-module` and
`reverseDependencyPolicy=forbidden`; `pnpm check:architecture-alignment` validates the contract.

### Functional Requirements And Acceptance Criteria

| ID | Requirement | Acceptance evidence |
| --- | --- | --- |
| FR-1 | Agent lifecycle supports create, retrieve, list, update, soft delete, restore, status control, tenant/user scoping | Open/App/Backend APIs expose lifecycle operations; App API applies owner isolation |
| FR-2 | Runtime binding is provider-neutral and allows one active binding per agent | `ai_agent_runtime_binding` persists bindings; activation is atomic and audited |
| FR-3 | External capabilities are composed through slots, not duplicated in agents tables | `ai_agent_composition_slot` stores `slot_kind`, `target_module`, `target_ref`, priority, policy, status |
| FR-4 | Hosted chat supports session/message persistence and canonical chat turn | `ai_agent_session` and `ai_agent_message`; `agents.messages.stream` persists user + assistant messages atomically and supports JSON or SSE response negotiation |
| FR-5 | Task scheduling is product-owned while execution runs remain kernel-projected | `ai_agent_task` and `agents.tasks.*` are live; `agents.taskRuns.*` is non-GA scope until kernel `AgentRun` projection is stable |
| FR-6 | Live interaction supports approval and user-question pause points | `ai_agent_interaction`; App/Backend `agents.interactions.*`; Open API excluded |
| FR-7 | Runtime catalog exposes canonical code engines and MCP marketplace projection to app clients | App API `agents.codeEngines.list` and `agents.mcpServers.list` |
| FR-8 | Product applications cannot directly consume kernel/provider crates | BirdCoder uses `sdkwork-agents-runtime-facade` and `@sdkwork/agents-app-sdk`; direct provider deps are forbidden |
| FR-9 | Commercial MVP is operable with audit, metrics, SDKs, pagination, frontend identity gates, production security gates, and fail-closed persistence | PRD Phase 4 gates plus `pnpm verify`, `pnpm check`, `pnpm check:frontend-service-identity`, `pnpm check:production-security`, API envelope, operation pattern, route collision, pagination, SDK import, composition, and BirdCoder contract checks |
| FR-10 | Independent capability modules remain upstream dependencies, never downstream consumers of agents | PRD and architecture dependency matrices say agents → memory/knowledgebase/skills/prompts/mcp/llm/drive; no reverse dependency is permitted |
| FR-11 | PC/H5 services preserve server-owned message identity, approved client business ID generation, and token-derived IAM context | Authored agents services do not call `crypto.randomUUID()` directly; generated SDK chat responses must include server `messageId`; client-required agent/execution business IDs use `@sdkwork/utils/id` helper wrappers; PC/H5 session bridges do not synthesize `environment`, `deploymentMode`, or `authLevel` defaults when IAM/JWT context omits them |
| FR-12 | PC 文件输入统一通过 Drive Uploader 形成稳定媒体资源 | 当前生产范围的 Agent 头像和聊天图片/附件/视频/语音使用 `@sdkwork/drive-app-sdk` composed uploader；业务只持久化 `drive://spaces/{spaceId}/nodes/{nodeId}` 与 `MediaResource`，短期下载 URL 仅用于预览；Creative policy 已治理但 UI 在权威 generation API 获批前不发布 |

## 5. User Scenarios

### 5.1 Agent 创建与配置

1. 业务开发者创建 Agent，填写 manifest（名称、描述、能力需求、事件族）
2. 配置实现类型（sdkwork-native / rig-rust / openai-agents / langchain / ...）
3. 设置可见性（private / organization / tenant / public）
4. 系统生成 `ai_agent` 记录并写入审计日志

### 5.2 供应商绑定与部署

1. 业务开发者为 Agent 绑定模型供应商（provider_id、configuration_profile_id）
2. 激活绑定（同一 Agent 仅允许一个 active binding）
3. 当前 manifest、默认 code task intent、active binding 和组合槽共同构成运行配置快照
4. 绑定创建、激活和状态变更写入 `ai_agent_audit_event`，通过 trace_id / request_id 回溯

### 5.3 跨模块组合

1. 业务开发者为 Agent 添加组合槽，引用外部模块资源
2. 例如：绑定知识库空间（slot_kind=knowledge, target_module=knowledgebase, target_ref=kb.space.xxx）
3. 例如：绑定 MCP 服务器（slot_kind=mcp, target_module=mcp, target_ref=mcp.server.xxx）
4. 设置优先级与启用状态
5. 运行时按 priority 顺序加载组合槽引用的资源

### 5.4 审计与回溯

1. 所有管理操作（创建、更新、删除、恢复、状态变更、绑定变更、部署、组合槽变更）写入审计日志
2. 审计日志不可变，按 tenant + agent + created_at 索引
3. 支持通过 request_id / trace_id 关联请求链路

### 5.5 BirdCoder 多代码智能体统一管理

1. BirdCoder 从 App API 读取 `agents.codeEngines.list`，展示 Codex、Claude Code、Gemini、OpenCode 等 T1 代码引擎
2. BirdCoder 通过 `sdkwork-agents-runtime-facade` 执行 turn，不直接依赖 `sdkwork-agent-kernel` 或 `sdkwork-agent-provider-*`
3. BirdCoder 自有 `coding_session*` 和 workbench 投影；跨域关联通过 `ai_agent_task.external_ref` 或 metadata 保存
4. 当 BirdCoder 需要 agents 未覆盖的能力时，先补全 `sdkwork-agents` API/SDK/facade，而不是绕过到 kernel

### 5.6 商业化运行与扩展

1. 运营方通过 Backend API 查询审计、会话、消息、任务和交互状态
2. 运维通过 `/metrics` 与 `/metrics/agents` 接入 Prometheus/Grafana
3. 新 provider 接入遵循“有官方 SDK 走 SDK；无官方 SDK 先声明插件绑定，不做 raw HTTP 绕过”
4. T2 自主智能体（OpenClaw、Hermes）在 conformance、健康检查和安全门禁通过后才进入默认 catalog

## 6. Success Metrics

| 指标 | 目标 |
| --- | --- |
| Agent CRUD API P99 延迟 | < 200ms |
| 组合槽查询 P99 延迟 | < 100ms |
| 审计日志写入成功率 | 99.99% |
| 数据库表数量 | 8 (组合平面 + 托管会话/消息/交互 + 任务) |
| 跨模块耦合 | 0 (所有外部资源通过 composition slot 引用) |
| HTTP API 契约 | 95 operations 与 OpenAPI / SDK / route manifest 一致 |
| SDK 消费边界 | App/Backend/Open 消费 composed SDK，无 generated deep import |
| BirdCoder 边界 | 无直接 `sdkwork-agent-kernel` / `sdkwork-agent-provider-*` 依赖 |
| 商业 MVP | PC/H5/BirdCoder 可完成 Auth + CRUD + Chat + Catalog + Task 基础闭环 |
| GA 缺口透明度 | Token streaming、taskRuns、T2 catalog、Flutter SDK、Grafana 明确记录为后续门禁 |

## 7. Phases

### Phase 1 — Composition Plane Baseline (已完成)

- 所有表统一使用 `ai_` 前缀
- 移除 MCP / 知识库 / 记忆遗留表
- 建立 composition slot 模式
- 迁移脚本就位

### Phase 2 — Production Hardening (已完成)

- [x] 移除生产环境 AllowAllPolicyProvider，实现 IAM-backed PolicyProvider
- [x] 实现 `SqlAgentAuditSink<SyncPostgresAdapter>` 替换生产内存审计，并保持 row adapter 端口方言无关
- [x] 确保 sdkwork-agents-runtime-facade 正确集成（workspace 注册 + 文档对齐 + 清理死代码）
- [x] 修复 std::sync::Mutex 阻塞异步执行器
- [x] 添加请求追踪中间件（CORS/限流延迟到 web-framework 层统一配置）
- [x] 修复 tenant_id 空默认值安全问题
- [x] 拆分 `http.rs` / `persistence.rs` 的 HTTP adapter 与 PostgreSQL SQL 常量子模块（owner: agents-platform；已落地：`http/context.rs`、`http/middleware.rs`、`http/testing.rs`、`persistence/sql.rs`；门禁：`rust-service-module-boundaries.contract.test.mjs` 已接入 `pnpm check:contracts`）

### Phase 3 — Client & Observability (已完成)

- [x] Prometheus metrics 采集（`/metrics/agents`，含 `sdkwork_agents_requests_per_second`）
- [x] CI 运行 feature-gated HTTP/Postgres 契约测试（`default = ["http-axum", "postgres-sync"]`）
- [x] Postgres interaction 持久化
- [x] 生产环境禁用 Postgres 不可用时的内存静默降级；managed-store 单项、列表和计数读取全部 fail-closed
- [x] PC/H5 生产聊天页（`AgentChatView`，sessions/messages API，会话恢复 + 消息上限）
- [x] PC/H5 前端服务身份与 IAM 上下文门禁（禁止直接 `crypto.randomUUID()`；服务端 `messageId` 缺失即失败；session bridge 不本地伪造 `environment` / `deploymentMode` / `authLevel`）
- [x] PC/H5 客户端：Auth Gate、知识库 bootstrap、运行时 catalog、composition slot 同步
- [x] PC Drive Uploader：生产头像与聊天媒体统一通过 composed Drive SDK；Creative media policy 已集中但原型生成 UI 不进入生产 composition；无 app-local upload API、上传表、对象存储 provider 或持久化预签名 URL
- [x] 可选 sibling SDK：skills / voice catalog（无 silent fallback，分页加载）
- [x] 小程序 runtime bundle 与 TypeScript bootstrap 对齐（verify 门禁）
- [x] 客户端 E2E 流程 contract（create agent → chat，`agent-e2e-flow-contract.test.ts`）
- [x] 小程序原生 agents 列表页（App API + runtime SDK；完整编辑器仍走 `pages/agents-h5` WebView）
- [x] 生产聊天接入 `RuntimeFacadeChatCompleter`（code-engine facade，无 contract stub）
- [x] App API 会话 owner 隔离（`owner_scope`）
- [x] 聊天 turn 原子持久化（Postgres 事务 + `insert_chat_turn`）
- [ ] Flutter Dart SDK 与移动端屏幕（当前不在 GA 证据包内；进入产品范围前必须提供自有 Dart app SDK facade）

### Phase 4 — Commercial GA (进行中)

- [ ] SQLite managed-store：完整 baseline、生命周期引导、持久化 adapter、事务、分页、审计和双引擎集成测试；在此之前不得把 runtime SQLite 数据库当作 agents managed-store 支持
- [ ] 业务写入与 `ai_agent_audit_event` 在同一数据库事务提交，或采用具备投递恢复与幂等语义的 transactional outbox
- [ ] 消息和审计高增长列表使用服务端 keyset/cursor 分页，OpenAPI、SDK、仓储 SQL 和前端消费保持同一合同
- [ ] 应用商店/渠道发布元数据（截图、描述、`sdkwork.workflow.json` GA 渠道）
- [x] 端到端自动化 contract（create → chat，纳入 `test:agent-contracts`）
- [x] `check:frontend-service-identity` 纳入 `pnpm check` / `pnpm check:contracts`
- [x] Live gateway 冒烟脚本（`pnpm smoke:live`，需运行中的 gateway）
- [ ] 端到端 live 全链路（Auth + CRUD + Chat + 真实 code-engine 推理，见 [smoke-test.md](../../runbooks/smoke-test.md)）
- [ ] Grafana 仪表盘对接 `/metrics` + `/metrics/agents`（运维平台）
- [x] 移除客户端虚假 catalog fallback（Voice / Skills）
- [x] SDK `sendAgentChatMessageSync` 封装非流式 chat send
- [x] Chat SSE 信封（`stream=true` → `201` + `text/event-stream` 单 completion 事件，SdkWorkApiResponse 信封）
- [x] 服务 worker/provider worker 有界并发与容量耗尽拒绝（默认 128/32，可按部署 profile 配置）
- [ ] Token 级增量 SSE（多 `data:` 分片；依赖 code-engine 流式分块暴露）
- [x] 任务调度 API + `ai_agent_task` 持久化（`agents.tasks.*` HTTP + Postgres；Open/App/Backend 三端）
- [x] 实时交互 API + `ai_agent_interaction` 持久化（`agents.interactions.*`；App/Backend，Open API 不含）
- [ ] `ai_agent_task_run` / `agents.taskRuns.*`（kernel `AgentRun` 投影，Phase 5）
- [ ] T2 自主智能体（openclaw、hermes）默认 catalog 纳入（CI 一致性门禁后）

### Phase 5 — Task Run Projection & Advanced Runtime (规划中)

- [x] `agents.tasks.*` HTTP 操作（Open / App / Backend；`ai_agent_task` 表）
- [x] `agents.interactions.*` HTTP 操作（App / Backend；`ai_agent_interaction` 表）
- [ ] `agents.taskRuns.*` HTTP 操作与 `ai_agent_task_run` 表
- [ ] kernel `AgentRun` ↔ `ai_agent_task` 事件驱动投影
- [ ] BirdCoder coding_session ↔ agent task 交叉引用 (`external_ref`)
- [ ] Live interaction **SPI** 迁移至 kernel（agents 已提供持久化与 App/Backend HTTP）

### Commercial Readiness Gates

| Gate | Current state | Required for GA |
| --- | --- | --- |
| PC/H5 hosted chat | Ready | Real code-engine smoke under production auth |
| BirdCoder code-agent management | Ready via agents facade | Task run dashboard and coding_session correlation |
| Open/App/Backend SDK | Ready for TypeScript | Dart SDK for Flutter |
| Streaming UX | Single SSE completion event | Token-level `message.delta` from kernel provider stream |
| Autonomous engines | T2 opt-in | Conformance + health + policy gate before default catalog |
| Observability | Metrics exposed | Grafana dashboard and operator runbook |
| Release metadata | Beta manifest | GA channel metadata, screenshots, SBOM/checksum evidence |
| Managed-store engines | PostgreSQL implementation verified; SQLite managed-store absent | PostgreSQL + SQLite baseline, lifecycle, repository, transaction, pagination, audit, and integration evidence |
| Audit durability | Persistent PostgreSQL sink, but business and audit writes use separate commits | Atomic business/audit commit or transactional outbox with replay and idempotency evidence |
| High-growth pagination | Offset pagination | Cursor/keyset pagination for messages and audit events across API, SDK, SQL, and UI |
| Production gates | Root `pnpm check` covers API, SDK imports, agent SDK workspace ownership, pagination, route, composition, frontend service identity, IAM session context preservation, apps index, production security, bounded worker capacity, shutdown signal resilience, dev-only policy fail-closed behavior, in-memory lock poison recovery, managed-store ID initialization error propagation, repository read failure propagation, route manifest build-script error propagation, strict repository ports, deployment, cloud profile validation, docs, topology, and database standards | `pnpm verify`, `pnpm deploy:validate:cloud`, `pnpm check:frontend-service-identity`, `pnpm check:production-security`, live smoke, database integration, capacity, and BirdCoder integration evidence before GA |

## 8. Linked Requirements

- [技术架构设计](../../architecture/tech/TECH_ARCHITECTURE.md)
- [Kernel 边界规范](../../../specs/AGENTS_KERNEL_BOUNDARY_SPEC.md)
- [Provider 分类规范](../../../specs/AGENTS_PROVIDER_TAXONOMY_SPEC.md)
- [Kernel SPI 差距分析](../../../specs/AGENTS_KERNEL_SPI_GAP_ANALYSIS.md)
- [BirdCoder 对齐追踪](../../../specs/agents-birdcoder-alignment.spec.json)
- [Composition Database Spec](../../../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md)
- [Database Schema Contract](../../../database/contract/schema.yaml)
- [Table Registry](../../../database/contract/table-registry.json)
- [API Specification](../../architecture/tech/TECH-api-specification.md)

## 9. Open Questions

- 组合槽的 policy_json 是否需要标准化 schema？（当前为自由 JSON）
- 是否需要支持组合槽的条件启用（基于运行时上下文）？
- `ai_agent_task_run` 与 kernel `AgentRun` / `AgentStep` 的同步策略：事件驱动还是轮询投影？
- T2 引擎（openclaw、hermes）纳入默认 catalog 的 CI 门禁标准？
- Rig live backend 是否先以 feature flag 形式进入 `implementation_type=rig-rust` 生产绑定？
