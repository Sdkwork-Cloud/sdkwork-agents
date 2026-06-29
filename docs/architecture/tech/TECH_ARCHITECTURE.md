# SDKWork Agents Technical Architecture

Status: active
Owner: agents-platform
Updated: 2026-06-28

## 1. Architecture Overview

SDKWork Agents 是一个智能体组合编排平台应用，遵循"积木式"模块架构设计原则。
本仓库仅拥有 **Agent 组合平面 (Composition Plane)**：Agent 身份、运行时绑定、部署快照、
组合槽引用、审计事实、出箱事件和应用注册。所有内容域（知识库、记忆、技能、提示词、
文件、MCP）由独立的 sibling 模块拥有，Agent 通过 `ai_agent_composition_slot` 引用它们。

### 架构分层

```text
┌──────────────────────────────────────────────────────────┐
│                    API Surfaces                           │
│  /agent/v3/api   /app/v3/api   /backend/v3/api           │
├──────────────────────────────────────────────────────────┤
│              sdkwork-agents-kernel-bridge                  │
│  HTTP wiring · router assembly · AgentHttpState bootstrap │
├──────────────────────────────────────────────────────────┤
│           sdkwork-intelligence-agents-service             │
│  application service · policy · audit · domain · ports ·  │
│  persistence · infrastructure · http                     │
├──────────────────────────────────────────────────────────┤
│              sdkwork-agents-runtime-facade                │
│  code-engine provider bootstrap · catalog · turn exec ·  │
│  live interaction (codex · claude-code · gemini ·         │
│  opencode · openclaw · hermes)                           │
├──────────────────────────────────────────────────────────┤
│                    sdkwork-kernel                         │
│        (agent runtime SPI · policy · event primitives)    │
├──────────────────────────────────────────────────────────┤
│     Sibling Modules (referenced via composition slot)     │
│  memory · knowledgebase · skills · prompts · drive · mcp  │
├──────────────────────────────────────────────────────────┤
│                    PostgreSQL / SQLite                    │
└──────────────────────────────────────────────────────────┘
```

## 2. Module Boundaries

本仓库严格遵循高内聚、低耦合原则。每个模块拥有自己的数据库表和业务逻辑。

### 2.1 本仓库拥有的表 (6 tables, all `ai_` prefix)

| Table | Responsibility | Compliance |
| --- | --- | --- |
| `ai_agent` | Agent 身份、manifest 快照、生命周期 | L2 |
| `ai_agent_runtime_binding` | 供应商/运行时绑定 | L2 |
| `ai_agent_composition_slot` | Agent → 外部模块资源引用 | L2 |
| `ai_agent_audit_event` | 不可变管理审计日志 | L3 |
| `ai_agent_session` | 托管会话（tenant/agent/owner 作用域） | L2 |
| `ai_agent_message` | 会话消息与 chat turn 持久化 | L2 |

### 2.2 Sibling 模块依赖 (通过 composition slot 引用)

| Module | Table Prefix | slot_kind | target_module |
| --- | --- | --- | --- |
| `sdkwork-memory` | `ai_` (memory-owned) | `memory` | `memory` |
| `sdkwork-knowledgebase` | `kb_` | `knowledge` | `knowledgebase` |
| `sdkwork-skills` | `ai_skill_*` | `skill` | `skills` |
| `sdkwork-prompts` | `ai_prompt_*` | `prompt` | `prompts` |
| `sdkwork-drive` | `dr_` | `drive` | `drive` |
| `sdkwork-mcp` | `ai_mcp_*` | `mcp` | `mcp` |

### 2.3 平台框架依赖

| Framework | Role |
| --- | --- |
| `sdkwork-kernel` | Agent runtime SPI, route crates, policy/event primitives |
| `sdkwork-web-framework` | HTTP interceptor chain via kernel `build_served_combined_router` |
| `sdkwork-utils` | Shared env parsing, ID generation, validation helpers |
| `sdkwork-drive` | Required for all file upload features (Drive Uploader only) |

## 3. Composition Slot Pattern

`ai_agent_composition_slot` 是跨模块通信的核心机制。Agent 不拥有外部资源的数据，
而是通过组合槽引用 sibling 模块的资源。

### 3.1 组合槽结构

```sql
CREATE TABLE ai_agent_composition_slot (
    id BIGINT PRIMARY KEY,
    uuid VARCHAR(96) UNIQUE,
    tenant_id BIGINT NOT NULL,
    organization_id BIGINT NOT NULL DEFAULT 0,
    agent_id VARCHAR(128) NOT NULL,
    slot_id VARCHAR(128) NOT NULL,          -- slot.{kind}.{name}
    slot_kind VARCHAR(64) NOT NULL,         -- memory|knowledge|skill|prompt|drive|tool|mcp
    target_module VARCHAR(64) NOT NULL,     -- memory|knowledgebase|skills|prompts|drive|mcp
    target_ref VARCHAR(256) NOT NULL,       -- 外部资源的稳定引用 ID
    target_version_ref VARCHAR(128),         -- 可选的版本固定
    priority INTEGER NOT NULL DEFAULT 0,    -- 编排顺序
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    policy_json JSONB NOT NULL DEFAULT '{}', -- 槽级策略覆盖（非密钥）
    status SMALLINT NOT NULL DEFAULT 1,
    version BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    UNIQUE (tenant_id, agent_id, slot_id)
);
```

### 3.2 组合槽示例

| slot_id | slot_kind | target_module | target_ref | 说明 |
| --- | --- | --- | --- | --- |
| `slot.memory.agent.default` | `memory` | `memory` | `mem.space.product` | 绑定记忆空间 |
| `slot.knowledge.agent.docs` | `knowledge` | `knowledgebase` | `kb.space.docs` | 绑定知识库空间 |
| `slot.skill.agent.search` | `skill` | `skills` | `ai_agent_skill.web.search` | 绑定技能 |
| `slot.prompt.agent.system` | `prompt` | `prompts` | `ai_prompt.system.v2` | 绑定系统提示词 |
| `slot.mcp.agent.tools` | `mcp` | `mcp` | `ai_mcp_server.toolset` | 绑定 MCP 服务器 |

## 4. Crate Layout

```text
crates/
  sdkwork-agents-contract/                    # 运行时环境辅助 (env_test_lock + dev-auth bypass)
    src/
      lib.rs             # 环境检测函数 + env_test_lock 导出
      runtime_env.rs     # 测试环境互斥锁
  sdkwork-agents-kernel-bridge/               # kernel 组合边界
  sdkwork-agents-standalone-gateway/          # 可运行 HTTP 服务器二进制
  sdkwork-agents-database-host/               # 数据库主机集成
  sdkwork-agents-gateway-assembly/            # 网关装配
  sdkwork-agents-runtime-facade/              # code-engine provider 门面 (bootstrap · catalog · turn)
  sdkwork-intelligence-agents-service/        # 核心服务 (domain + ports + persistence + http + 契约)
    src/
      domain.rs          # 领域模型 + 公共枚举 (status/visibility/role/kind...)
      ports.rs           # 端口定义 (Repository, AuditSink)
      persistence.rs     # PostgreSQL 持久化适配器
      infrastructure.rs  # 内存实现 (测试用)
      application.rs     # 应用服务 (命令处理 · send_chat_message)
      chat_runtime.rs    # 托管 chat turn 完成 (contract mode → runtime-facade)
      dto.rs             # API 请求/响应 DTO + 信封包装类型
      response.rs        # ApiProblem (numeric code) + ResourceData/PageData 信封助手
      http.rs            # HTTP 路由 (axum) + AgentRequestContext + WebRequestContext 桥接
      api.rs             # API 操作元数据 + OpenAPI 校验
      validation.rs      # 输入验证
      id.rs              # Snowflake ID 生成
      code_engine_catalog.rs  # Code engine catalog 投影 (ResourceData<CodeEngineCatalog>)
      mcp_marketplace.rs      # MCP marketplace 投影 (PageData<McpServerMarketplaceRecord>)
  sdkwork-routes-agents-app-api/              # App API 路由
  sdkwork-routes-agents-backend-api/          # Backend API 路由
  sdkwork-routes-agents-open-api/             # Open API 路由
  sdkwork-routes-agents-http-shared/          # 共享 HTTP 工具
  sdkwork-agents-integration-tests/           # 集成测试
```

### 4.1 契约与信封归属

`sdkwork-intelligence-agents-service` 是 API 契约与信封的唯一真相源，
遵循 `CODE_STYLE_SPEC.md` 的职责分离原则：

| 模块 | 职责 | 依赖 |
| --- | --- | --- |
| `domain` | 领域模型 + 公共枚举 + DB 编码映射 | `serde` |
| `dto` | API 请求/响应序列化类型 | `serde`, `domain` |
| `response` | `ApiProblem` (numeric int32 code) + `ResourceData<T>`/`PageData<T>` 信封 + `finish_api_json`/`created_json` 助手 | `sdkwork-web-core`, `sdkwork-utils-rust` |
| `http` | axum 路由 + `AgentRequestContext` + `WebRequestContext` 桥接 + 中间件 | `axum`, `sdkwork-web-core` |

`sdkwork-agents-contract` 仅保留运行时环境辅助（`env_test_lock`、
`agents_is_production_like_environment`、`agents_use_dev_inline_auth_resolver`），
供路由 crate 和集成测试共享。所有 DTO、枚举、路径、错误模型均由 service crate
单一持有，避免重复定义。

完整 API 规范见 [API_SPECIFICATION.md](API_SPECIFICATION.md)。

## 5. Database Design

### 5.1 设计原则

1. **所有表使用 `ai_` 前缀** — 遵循 DATABASE_SPEC.md 智能体域前缀标准
2. **仅拥有 6 张表** — Agent 组合平面 + 托管会话/消息的最小完备集
3. **组合槽引用** — 所有外部模块资源通过 `ai_agent_composition_slot` 引用，不复制域数据
4. **Snowflake ID** — 应用层分配 ID，不依赖数据库自增
5. **int64 内部 / string API** — 避免 JavaScript 精度问题
6. **软删除 + 审计** — 管理操作可追溯
7. **多租户隔离** — `tenant_id` + `organization_id` 显式列
8. **无明文密钥** — 仅存储引用 (`profile.*`, `endpoint.*`)

### 5.2 ER 概览

```text
                    ai_agent (身份、清单、生命周期)
                                │
        ┌───────────────────────┼───────────────────────────┐
        │                       │                           │
ai_agent_runtime      ai_agent_composition        ai_agent_audit_event
    _binding               _slot                   (不可变)
  (供应商)            (引用外部模块)
        │
        └── ai_agent_session ── ai_agent_message
              (托管会话)          (chat turn 消息)
```

### 5.3 索引策略

| Table | Indexes |
| --- | --- |
| `ai_agent` | `(tenant_id, organization_id, status, updated_at DESC)`, `(tenant_id, owner_user_id, status)` |
| `ai_agent_runtime_binding` | unique partial `(tenant_id, agent_id) WHERE active=TRUE`, `(tenant_id, agent_id, active, updated_at, binding_id)` |
| `ai_agent_composition_slot` | `(tenant_id, agent_id, slot_kind, enabled, priority, slot_id)` |
| `ai_agent_audit_event` | `(tenant_id, agent_id, created_at DESC)`, `(tenant_id, action, created_at DESC)` |
| `ai_agent_session` | `(tenant_id, agent_id, owner_user_id, status, updated_at DESC)`, unique `(tenant_id, session_id)` |
| `ai_agent_message` | `(tenant_id, session_id, sequence)`, unique `(tenant_id, message_id)` |

### 5.4 约束策略

- **CHECK 约束**：枚举值在数据库层强制（status、visibility、implementation_kind、slot_kind、target_module）
- **UNIQUE 约束**：业务键唯一（tenant_id + agent_id, tenant_id + code, uuid）
- **标准 ID 格式**：正则约束 (`^provider\.`, `^binding\.`, `^slot\.`, `^profile\.`)
- **能力 JSON 验证**：PL/pgSQL 函数验证 capabilities JSON 格式

## 6. API Surfaces

| Surface | Prefix | Audience |
| --- | --- | --- |
| Open API | `/agent/v3/api` | 第三方集成方 |
| App API | `/app/v3/api` | 前端应用 (H5/PC/Mini Program/Flutter) |
| Backend API | `/backend/v3/api` | 管理后台 |

完整 API 列表（68 个 HTTP 操作，含 session/message chat completion）参见 [API 参考文档](TECH-API-REFERENCE.md)。

### 6.1 Chat Completion

`POST .../sessions/{sessionId}/messages` 是 canonical chat turn 入口：

1. 校验 agent/session 归属与 active 状态
2. 持久化 user message
3. 调用 `AgentsService::send_chat_message` → 可注入 `ChatCompleter`（默认 `ContractChatCompleter`；生产由网关通过 `AgentHttpState::with_chat_completer(KernelModelChatCompleter::new(...))` 挂载 kernel `ModelProvider`）
4. 持久化 assistant message 并更新 session counters
5. 返回 `AgentChatCompletionResponse`（session + userMessage + assistantMessage）

`?stream=true` 时返回 `text/event-stream`，包含一个 `completion` 事件（JSON 与上述响应体相同）。逐 token 流式输出待 kernel `ModelProvider::stream` 集成后扩展。

PC 管理面 `managementProfile` 通过 `defaultCodeTaskIntent.constraints` 中的
`sdkwork.agent.pc.config:{json}` 与 OpenAPI 对齐，包含 `knowledgeBaseIds`、`skillIds`、
`toolIds`、`voiceIds`、`memoryEnabled` 等字段。

所有 API 操作通过 `api.rs` 中定义的 `ApiOperation` 元数据驱动路由注册和 OpenAPI 生成。

## 7. Agent Lifecycle

```text
Draft (0) → Active (1) → Disabled (2) → Archived (3)
                ↓                          ↓
            Deleted (4) ← ────────────────┘
```

- **Draft**: 刚创建，未发布
- **Active**: 已发布，可运行
- **Disabled**: 暂停运行
- **Archived**: 归档，不可运行但保留数据
- **Deleted**: 软删除，可恢复

## 8. Implementation Type

Agent 支持多种运行时实现框架：

| Type | Description |
| --- | --- |
| `sdkwork-native` | SDKWork 原生运行时 (默认) |
| `rig-rust` | Rig Rust 框架 |
| `openai-agents` | OpenAI Agents SDK |
| `langchain` | LangChain |
| `langgraph` | LangGraph |
| `crewai` | CrewAI |
| `autogen` | AutoGen |
| `semantic-kernel` | Semantic Kernel |
| `custom` | 自定义实现 |

## 9. Verification

```powershell
pnpm verify
pnpm check
pnpm topology:validate
pnpm db:validate
```

## 10. Launch Readiness

Pre-launch P0/P1 alignment is complete. Remaining items are post-launch platform
capabilities owned by sibling SDKWork layers, not agents-application blockers.

### Completed (pre-launch)

| Area | Status |
| --- | --- |
| IAM-backed policy + Postgres audit sink | Done |
| 70-operation HTTP surface (22/25/23) + OpenAPI/SDK sync | Done |
| Session/message chat + SSE completion event | Done |
| `code_engine_catalog` + `mcp_marketplace` + `runtime_facade_bridge` | Done |
| HTTP-only DTOs colocated in `http.rs` (no feature-gate dead code) | Done |
| Open API / Open SDK surface boundary (no restore/catalog drift) | Done |
| HTTP route trees aligned to OpenAPI authority (22/25/23, no phantom routes) | Done |
| Interaction domain in application/repository only (no public HTTP until OpenAPI) | Done |
| Legacy MCP/Memory/Knowledge inline types removed | Done |
| Structured audit payloads (agent, binding, runtime, marketplace) | Done |

### Post-launch (platform-owned)

| Priority | Item | Owner |
| --- | --- | --- |
| P1 | Token-level SSE streaming | kernel `ModelProvider::stream` |
| P1 | Rate limit + CORS middleware | sdkwork-web-framework |
| P2 | Prometheus metrics / dashboards | ops + web-framework |
| P2 | MCP marketplace federation HTTP | sdkwork-mcp sibling mount |
| P2 | Mini program SDK client surface | apps/sdkwork-agents-mini-program |
| P2 | Direct open SDK sdkgen (`/agent/v3/api` profile) | sdkwork-sdk-generator |
