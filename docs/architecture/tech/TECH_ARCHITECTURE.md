# SDKWork Agents Technical Architecture

Status: active
Owner: agents-platform
Updated: 2026-06-26

## 1. Architecture Overview

SDKWork Agents 是一个智能体组合编排平台应用，遵循"积木式"模块架构设计原则。
本仓库仅拥有 **Agent 组合平面 (Composition Plane)**：Agent 身份、运行时绑定、部署快照、
组合槽引用、审计事实、出箱事件和应用注册。所有内容域（知识库、记忆、技能、提示词、
文件、MCP）由独立的 sibling 模块拥有，Agent 通过 `ai_agent_composition_slot` 引用它们。

### 架构分层

```text
┌──────────────────────────────────────────────────────────┐
│                    API Surfaces                           │
│  /open/v3/api   /app/v3/api   /backend/v3/api            │
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
│        (agent runtime SPI · session persistence)          │
├──────────────────────────────────────────────────────────┤
│     Sibling Modules (referenced via composition slot)     │
│  memory · knowledgebase · skills · prompts · drive · mcp  │
├──────────────────────────────────────────────────────────┤
│                    PostgreSQL / SQLite                    │
└──────────────────────────────────────────────────────────┘
```

## 2. Module Boundaries

本仓库严格遵循高内聚、低耦合原则。每个模块拥有自己的数据库表和业务逻辑。

### 2.1 本仓库拥有的表 (4 tables, all `ai_` prefix)

| Table | Responsibility | Compliance |
| --- | --- | --- |
| `ai_agent` | Agent 身份、manifest 快照、生命周期 | L2 |
| `ai_agent_runtime_binding` | 供应商/运行时绑定 | L2 |
| `ai_agent_composition_slot` | Agent → 外部模块资源引用 | L2 |
| `ai_agent_audit_event` | 不可变管理审计日志 | L3 |

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
| `sdkwork-kernel` | Agent runtime SPI, route crates, session persistence |
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
  sdkwork-agents-contract/                    # 运行时环境辅助 (utils)
  sdkwork-agents-kernel-bridge/               # kernel 组合边界
  sdkwork-agents-standalone-gateway/                  # 可运行 HTTP 服务器二进制
  sdkwork-agents-database-host/               # 数据库主机集成
  sdkwork-agents-gateway-assembly/            # 网关装配
  sdkwork-agents-runtime-facade/              # code-engine provider 门面 (bootstrap · catalog · turn)
  sdkwork-intelligence-agents-service/        # 核心服务 (domain + ports + persistence)
    src/
      domain.rs          # 领域模型 (Agent 身份、绑定、部署、组合槽)
      ports.rs           # 端口定义 (Repository, AuditSink)
      persistence.rs     # PostgreSQL 持久化适配器
      infrastructure.rs  # 内存实现 (测试用)
      application.rs     # 应用服务 (命令处理)
      dto.rs             # 数据传输对象
      http.rs            # HTTP 路由 (axum)
      api.rs             # API 操作元数据
      validation.rs      # 输入验证
      id.rs              # Snowflake ID 生成
  sdkwork-routes-agents-app-api/              # App API 路由
  sdkwork-routes-agents-backend-api/          # Backend API 路由
  sdkwork-routes-agents-open-api/             # Open API 路由
  sdkwork-routes-agents-http-shared/          # 共享 HTTP 工具
  sdkwork-agents-integration-tests/           # 集成测试
```

## 5. Database Design

### 5.1 设计原则

1. **所有表使用 `ai_` 前缀** — 遵循 DATABASE_SPEC.md 智能体域前缀标准
2. **仅拥有 4 张表** — 不过度设计，Agent 组合平面的最小完备集
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
                    ┌───────────┼───────────────┐
                    │           │               │
        ai_agent_runtime   ai_agent_composition   ai_agent_audit_event
            _binding          _slot              (不可变)
            (供应商)          (引用外部模块)
```

### 5.3 索引策略

| Table | Indexes |
| --- | --- |
| `ai_agent` | `(tenant_id, organization_id, status, updated_at DESC)`, `(tenant_id, owner_user_id, status)` |
| `ai_agent_runtime_binding` | unique partial `(tenant_id, agent_id) WHERE active=TRUE`, `(tenant_id, agent_id, active, updated_at, binding_id)` |
| `ai_agent_composition_slot` | `(tenant_id, agent_id, slot_kind, enabled, priority, slot_id)` |
| `ai_agent_audit_event` | `(tenant_id, agent_id, created_at DESC)`, `(tenant_id, action, created_at DESC)` |

### 5.4 约束策略

- **CHECK 约束**：枚举值在数据库层强制（status、visibility、implementation_kind、slot_kind、target_module）
- **UNIQUE 约束**：业务键唯一（tenant_id + agent_id, tenant_id + code, uuid）
- **标准 ID 格式**：正则约束 (`^provider\.`, `^binding\.`, `^slot\.`, `^profile\.`)
- **能力 JSON 验证**：PL/pgSQL 函数验证 capabilities JSON 格式

## 6. API Surfaces

| Surface | Prefix | Audience |
| --- | --- | --- |
| Open API | `/open/v3/api` | 第三方集成方 |
| App API | `/app/v3/api` | 前端应用 (H5/PC/Mini Program/Flutter) |
| Backend API | `/backend/v3/api` | 管理后台 |

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

## 10. Pending Technical Debt

以下项目已在 PRD Phase 2/3 中规划，按优先级排列：

| Priority | Item | Status |
| --- | --- | --- |
| P0 | 移除生产环境 AllowAllPolicyProvider, 实现 IAM-backed PolicyProvider | Done |
| P0 | 实现 PostgresAgentAuditSink 替换内存审计 | Done |
| P0 | 确保 sdkwork-agents-runtime-facade 正确集成 | Done |
| P1 | 修复 std::sync::Mutex 阻塞异步执行器 | Done |
| P1 | 拆分超大文件 http.rs 和 persistence.rs | Pending (http-axum feature cannot be compiled on Windows toolchain; deferring to avoid unverifiable refactor) |
| P1 | 添加限流 / CORS / 请求追踪中间件 | Partial (request tracing done; rate-limit/CORS deferred to web-framework layer) |
| P1 | 修复 tenant_id 空默认值安全问题 | Done |
| P1 | 清理 domain.rs 中 MCP/Memory/Knowledge 遗留类型 | Done |
| P1 | 清理 application.rs/dto.rs/http.rs 中 MCP/Memory/Knowledge 遗留 | Done |
| P2 | 补充集成测试覆盖 | Pending |
| P2 | 添加 Prometheus metrics 和可观测性 | Pending |
| P2 | 实现小程序 SDK 客户端 | Pending |
