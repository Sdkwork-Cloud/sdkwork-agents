# sdkwork-agents PRD

Status: active
Owner: agents-platform
Application: sdkwork-agents
Updated: 2026-06-28
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Add `PRD-<topic>.md` shards in this directory when the PRD grows beyond one reviewable screen.

## 1. Background And Problem

企业级智能体平台需要管理大量 AI Agent 的全生命周期：从定义、配置、绑定模型供应商、
部署发布到运行时编排。行业主流方案（如 OpenAI Agents、LangGraph、CrewAI、AutoGen）
各自封闭，缺乏统一的组合编排能力。

sdkwork-agents 解决的核心问题：

1. **Agent 身份与生命周期管理** — 统一管理 Agent 的创建、配置、发布、下线。
2. **运行时绑定与部署快照** — 将 Agent 绑定到模型供应商和运行时实现，并通过部署
   快照保证可回溯。
3. **跨模块组合编排** — Agent 需要调用记忆、知识库、技能、提示词、文件、MCP 等能力，
   但这些能力由独立的 sibling 模块拥有，不能在 agents 模块内重复实现。
4. **审计与可观测** — 所有管理操作必须可审计、可追踪。

## 2. Target Users

| 用户角色 | 使用场景 |
| --- | --- |
| 平台管理员 | 配置租户、应用注册、Agent 部署审核 |
| 业务开发者 | 创建 Agent、配置 manifest、绑定供应商 |
| 应用集成方 | 通过 Open API / App SDK 调用 Agent 能力 |
| 运维团队 | 监控 Agent 运行状态、审计日志、部署回滚 |

## 3. Goals And Non-Goals

### Goals

- 提供完整的 Agent 全生命周期管理（创建、更新、删除、恢复、状态变更）
- 支持多供应商运行时绑定（manifest-only、typed-local-provider、process-adapter、protocol-adapter）
- 通过组合槽 (composition slot) 统一引用外部模块资源
- 支持部署快照与版本回溯
- 提供不可变审计日志
- 支持多租户隔离与组织级权限
- 遵循 sdkwork-specs 标准，使用 ai_ 表前缀

### Non-Goals

- 不实现 MCP 服务器管理（由 `sdkwork-mcp` 拥有）
- 不实现知识库 / RAG（由 `sdkwork-knowledgebase` 拥有）
- 不实现记忆系统（由 `sdkwork-memory` 拥有）
- 不实现技能管理（由 `sdkwork-skills` 拥有）
- 不实现提示词管理（由 `sdkwork-prompts` 拥有）
- 不实现文件上传（由 `sdkwork-drive` 拥有）
- 不实现 Agent 运行时会话状态（由 `sdkwork-kernel` 拥有）

## 4. Scope

### Core Owned Tables (7 tables, all `ai_` prefix)

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Agent 身份、manifest 快照、生命周期、可见性 |
| `ai_agent_runtime_binding` | 供应商/运行时绑定 |
| `ai_agent_composition_slot` | Agent → 外部模块资源引用（记忆、知识、技能、提示词、文件、MCP） |
| `ai_agent_audit_event` | 不可变管理审计日志 |
| `ai_agent_session` | 托管 chat 会话（tenant/agent/owner 作用域） |
| `ai_agent_message` | 会话消息与 chat turn 持久化 |
| `ai_agent_interaction` | 实时交互（code-engine 审批流） |

### App-only Runtime Catalog APIs

| Method | Path | operationId |
| --- | --- | --- |
| GET | `/app/v3/api/ai/code_engines` | `agents.codeEngines.list` |
| GET | `/app/v3/api/ai/mcp_servers` | `agents.mcpServers.list` |

完整 API 列表见 [TECH-api-specification.md](../../architecture/tech/TECH-api-specification.md)（70 HTTP 操作）。

### Sibling Module Dependencies (referenced via composition slot)

| Module | Prefix | Composition slot_kind |
| --- | --- | --- |
| `sdkwork-memory` | `ai_` (memory-owned) | `memory` |
| `sdkwork-knowledgebase` | `kb_` | `knowledge` |
| `sdkwork-skills` | `ai_skill_*` | `skill` |
| `sdkwork-prompts` | `ai_prompt_*` | `prompt` |
| `sdkwork-drive` | `dr_` | `drive` |
| `sdkwork-mcp` | `ai_mcp_*` | `mcp` |

## 5. User Scenarios

### 5.1 Agent 创建与配置

1. 业务开发者创建 Agent，填写 manifest（名称、描述、能力需求、事件族）
2. 配置实现类型（sdkwork-native / rig-rust / openai-agents / langchain / ...）
3. 设置可见性（private / organization / tenant / public）
4. 系统生成 `ai_agent` 记录并写入审计日志

### 5.2 供应商绑定与部署

1. 业务开发者为 Agent 绑定模型供应商（provider_id、configuration_profile_id）
2. 激活绑定（同一 Agent 仅允许一个 active binding）
3. 创建部署记录，快照当前绑定信息
4. 部署状态流转：created → active → archived / failed

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

## 6. Success Metrics

| 指标 | 目标 |
| --- | --- |
| Agent CRUD API P99 延迟 | < 200ms |
| 组合槽查询 P99 延迟 | < 100ms |
| 审计日志写入成功率 | 99.99% |
| 数据库表数量 | 6 (组合平面最小完备集) |
| 跨模块耦合 | 0 (所有外部资源通过 composition slot 引用) |

## 7. Phases

### Phase 1 — Composition Plane Baseline (已完成)

- 所有表统一使用 `ai_` 前缀
- 移除 MCP / 知识库 / 记忆遗留表
- 建立 composition slot 模式
- 迁移脚本就位

### Phase 2 — Production Hardening (已完成)

- [x] 移除生产环境 AllowAllPolicyProvider，实现 IAM-backed PolicyProvider
- [x] 实现 PostgresAgentAuditSink 替换内存审计
- [x] 确保 sdkwork-agents-runtime-facade 正确集成（workspace 注册 + 文档对齐 + 清理死代码）
- [x] 修复 std::sync::Mutex 阻塞异步执行器
- [x] 添加请求追踪中间件（CORS/限流延迟到 web-framework 层统一配置）
- [x] 修复 tenant_id 空默认值安全问题
- [ ] 拆分超大文件 `http.rs` / `persistence.rs`（可维护性优化，非上线阻塞项）

### Phase 3 — Client & Observability (已完成)

- [x] Prometheus metrics 采集（`/metrics/agents`，含 `sdkwork_agents_requests_per_second`）
- [x] CI 运行 feature-gated HTTP/Postgres 契约测试（`default = ["http-axum", "postgres-sync"]`）
- [x] Postgres interaction 持久化
- [x] 生产环境禁用 Postgres 不可用时的内存静默降级
- [x] PC/H5 生产聊天页（`AgentChatView`，sessions/messages API，会话恢复 + 消息上限）
- [x] PC/H5 客户端：Auth Gate、知识库 bootstrap、运行时 catalog、composition slot 同步
- [x] 可选 sibling SDK：skills / voice catalog（无 silent fallback，分页加载）
- [x] 小程序 runtime bundle 与 TypeScript bootstrap 对齐（verify 门禁）
- [x] 客户端 E2E 流程 contract（create agent → chat，`agent-e2e-flow-contract.test.ts`）
- [x] 小程序原生 agents 列表页（App API + runtime SDK；完整编辑器仍走 `pages/agents-h5` WebView）
- [x] 生产聊天接入 `RuntimeFacadeChatCompleter`（code-engine facade，无 contract stub）
- [x] App API 会话 owner 隔离（`owner_scope`）
- [x] 聊天 turn 原子持久化（Postgres 事务 + `insert_chat_turn`）
- [ ] Flutter Dart SDK 与移动端屏幕（`pending-dart-sdk`）

### Phase 4 — Commercial GA (进行中)

- [ ] 应用商店/渠道发布元数据（截图、描述、`sdkwork.workflow.json` GA 渠道）
- [x] 端到端自动化 contract（create → chat，纳入 `test:agent-contracts`）
- [x] Live gateway 冒烟脚本（`pnpm smoke:live`，需运行中的 gateway）
- [ ] 端到端 live 全链路（Auth + CRUD + Chat + 真实 code-engine 推理，见 [smoke-test.md](../../runbooks/smoke-test.md)）
- [ ] Grafana 仪表盘对接 `/metrics` + `/metrics/agents`（运维平台）
- [x] 移除客户端虚假 catalog fallback（Voice / Skills）
- [x] SDK `sendAgentChatMessageSync` 封装非流式 chat send

## 8. Linked Requirements

- [技术架构设计](../../architecture/tech/TECH_ARCHITECTURE.md)
- [Composition Database Spec](../../../crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md)
- [Database Schema Contract](../../../database/contract/schema.yaml)
- [Table Registry](../../../database/contract/table-registry.json)

## 9. Open Questions

- 组合槽的 policy_json 是否需要标准化 schema？（当前为自由 JSON）
- 是否需要支持组合槽的条件启用（基于运行时上下文）？
- outbox 事件的消费方协议是否需要标准化？
