# SDKWork Agents Technical Architecture

Status: active
Owner: agents-platform
Updated: 2026-07-07
Specs: [`ARCHITECTURE_DECISION_SPEC.md`](../../../../sdkwork-specs/ARCHITECTURE_DECISION_SPEC.md), [`WEB_FRAMEWORK_SPEC.md`](../../../../sdkwork-specs/WEB_FRAMEWORK_SPEC.md), [`DATABASE_FRAMEWORK_SPEC.md`](../../../../sdkwork-specs/DATABASE_FRAMEWORK_SPEC.md), [`API_SPEC.md`](../../../../sdkwork-specs/API_SPEC.md), [`SDK_SPEC.md`](../../../../sdkwork-specs/SDK_SPEC.md)

## 1. Architecture Overview

SDKWork Agents 是一个智能体组合编排平台应用，遵循"积木式"模块架构设计原则。
本仓库仅拥有 **Agent 组合平面 (Composition Plane)**：Agent 身份、运行时绑定、
组合槽引用、审计事实、托管会话/消息/交互/任务，以及应用注册。所有内容域（知识库、
记忆、技能、提示词、文件、MCP、LLM 模型目录/网关）由独立模块拥有。Agent 通过
`ai_agent_composition_slot` 引用 memory/knowledgebase/skills/prompts/mcp/drive，
通过 runtime binding / provider profile 引用 LLM 能力。运行时机制由 `sdkwork-kernel` 提供；产品应用
（如 BirdCoder）通过本仓库的 HTTP/SDK 和 `sdkwork-agents-runtime-facade` 消费能力。

架构目标是把 `sdkwork-kernel` 的 Linux-kernel-style SPI 封装成可商业化交付的业务层：
kernel 定义机制和 provider SPI，agents 定义业务域、数据库、API、SDK、审计和产品查询模型，
BirdCoder 等产品只使用 agents 暴露的 SDK/facade。

### 架构分层

```text
┌──────────────────────────────────────────────────────────┐
│  Product Apps (BirdCoder PC — coding_session / workbench)  │
│  @sdkwork/agents-app-sdk · sdkwork-agents-runtime-facade   │
├──────────────────────────────────────────────────────────┤
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
│  memory · knowledgebase · skills · prompts · mcp · llm    │
│  drive                                                     │
├──────────────────────────────────────────────────────────┤
│                    PostgreSQL (managed store)               │
└──────────────────────────────────────────────────────────┘
```

### Baseline architecture decisions

| Decision | Current baseline |
| --- | --- |
| Kernel boundary | `sdkwork-kernel` owns SPI, provider plugins, runtime objects, events, policy, telemetry; it does not own `ai_*` tables or business APIs |
| Agents business plane | `sdkwork-agents` owns 8 `ai_*` tables, 95 HTTP operations, TypeScript SDK families, runtime facade, audit, sessions, messages, interactions, tasks |
| Independent module dependency | agents depends on memory, knowledgebase, skills, prompts, mcp, llm, and drive; those modules do not depend on agents |
| Sibling composition | memory, knowledgebase, skills, prompts, drive, and mcp are referenced through `ai_agent_composition_slot`; LLM is referenced through runtime binding / provider profile; no independent-module tables are duplicated |
| Product consumption | BirdCoder, PC, H5, mini program, and Flutter consume agents SDK/facade; direct kernel/provider dependencies are forbidden |
| Provider integration | T1 code engines are canonical; T2 autonomous engines are opt-in until conformance, health, and policy gates pass |

## 2. Module Boundaries

本仓库严格遵循高内聚、低耦合原则。每个模块拥有自己的数据库表和业务逻辑。

### 2.1 本仓库拥有的表 (8 tables, all `ai_` prefix)

| Table | Responsibility | Compliance |
| --- | --- | --- |
| `ai_agent` | Agent 身份、manifest 快照、生命周期 | L2 |
| `ai_agent_runtime_binding` | 供应商/运行时绑定 | L2 |
| `ai_agent_composition_slot` | Agent → 外部模块资源引用 | L2 |
| `ai_agent_audit_event` | 不可变管理审计日志 | L3 |
| `ai_agent_session` | 托管会话（tenant/agent/owner 作用域） | L2 |
| `ai_agent_message` | 会话消息与 chat turn 持久化 | L2 |
| `ai_agent_interaction` | 实时交互（code-engine 审批流） | L2 |
| `ai_agent_task` | 计划任务（kernel `AgentTask` 投影） | L2 |

DDL 权威：`database/ddl/baseline/postgres/0001_agents_baseline.sql`（PostgreSQL only）。

### 2.2 Independent module dependencies

`sdkwork-agents` 是这些模块的消费者和编排方，不是它们的上游依赖。依赖方向固定为
`sdkwork-agents -> independent capability modules`。独立模块暴露自己的 API、SDK、
schema、数据库和运行时契约；agents 只保存引用和策略，不复制业务数据。

| Module | Owned capability | Integration in agents | Dependency direction |
| --- | --- | --- | --- |
| `sdkwork-memory` | 记忆空间、永久/用户/成长型记忆、记忆检索与写入 | `slot_kind=memory`, `target_module=memory`; memory app SDK when mounted | agents → memory |
| `sdkwork-knowledgebase` | 知识库、RAG 索引、文档检索、知识空间 | `slot_kind=knowledge`, `target_module=knowledgebase`; knowledgebase app SDK | agents → knowledgebase |
| `sdkwork-skills` | 技能定义、技能包、技能市场、技能调用元数据 | `slot_kind=skill`, `target_module=skills`; skills app SDK | agents → skills |
| `sdkwork-prompts` | prompt 模板、系统提示词、版本化提示词资产 | `slot_kind=prompt`, `target_module=prompts`; prompts app SDK | agents → prompts |
| `sdkwork-mcp` | MCP server、MCP 工具目录、marketplace/federation | `slot_kind=mcp`, `target_module=mcp`; marketplace projection / future federation | agents → mcp |
| `sdkwork-llm` | LLM 模型目录、供应商配置、模型网关、凭证引用 | `ai_agent_runtime_binding`, `configuration_profile_id`, model provider profile | agents → llm / kernel provider |
| `sdkwork-drive` | Drive Uploader、文件上传、对象存储、下载 | `slot_kind=drive`, `target_module=drive`; Drive Uploader only | agents → drive |

Reverse dependencies are forbidden: `sdkwork-memory`, `sdkwork-knowledgebase`,
`sdkwork-skills`, `sdkwork-prompts`, `sdkwork-mcp`, `sdkwork-llm`, and `sdkwork-drive`
MUST NOT call `sdkwork-agents` for their core domain behavior.
The machine-readable boundary lives in `specs/component.spec.json`; every independent
capability module uses `dependencyMode=independent-capability-module` and
`reverseDependencyPolicy=forbidden`, enforced by `pnpm check:architecture-alignment`.

### 2.3 平台框架依赖

| Framework | Role |
| --- | --- |
| `sdkwork-kernel` | Agent runtime SPI, route crates, policy/event primitives |
| `sdkwork-web-framework` | HTTP interceptor chain via kernel `build_served_combined_router` |
| `sdkwork-utils` | Shared env parsing, ID generation, validation helpers |
| `sdkwork-drive` | Required for all file upload features (Drive Uploader only) |

### 2.4 Kernel SPI Reference (mechanism — owned by sdkwork-kernel)

| SPI family | Spec | Agents usage |
| --- | --- | --- |
| Object model | `AGENT_KERNEL_SPEC.md` | DTO mapping; no duplication in `ai_*` |
| Model provider | `AGENT_MODEL_PROVIDER_SPI_SPEC.md` | runtime-facade turn execution; LLM model/catalog/profile authority remains in `sdkwork-llm` and kernel provider binding |
| Tool / MCP / Skill | `AGENT_*_PROVIDER_SPI_SPEC.md` | composition slots + kernel invoke |
| Memory context | `AGENT_CONTEXT_MEMORY_SPEC.md` | slot → `sdkwork-memory` |
| Planning / tasks | `AGENT_PLANNING_EXECUTION_SPEC.md` | `ai_agent_task` is live; `ai_agent_task_run` waits for kernel run projection |
| Provider integration | `AGENT_PROVIDER_INTEGRATION_SPEC.md` | facade bootstraps T1/T2 plugins |
| Security / policy | `AGENT_SECURITY_POLICY_SPEC.md` | IAM-backed `PolicyProvider` |
| Live interaction | transitioning to kernel SPI | `ai_agent_interaction` persistence |

Full gap analysis: [`specs/AGENTS_KERNEL_SPI_GAP_ANALYSIS.md`](../../../specs/AGENTS_KERNEL_SPI_GAP_ANALYSIS.md).

### 2.5 Code-engine provider tiers

| Tier | Engines | Default in `AgentsCodeEngineHost` | App API catalog |
| --- | --- | --- | --- |
| T1 Code | codex, claude-code, gemini, opencode | Yes | `GET /app/v3/api/ai/code_engines` |
| T2 Autonomous | openclaw, hermes | On-demand bootstrap | Non-GA catalog until conformance gates pass |
| T3 Framework | rig (`implementation_type=rig-rust`) | Kernel plugin | Not in code-engine catalog |

Taxonomy authority: [`specs/AGENTS_PROVIDER_TAXONOMY_SPEC.md`](../../../specs/AGENTS_PROVIDER_TAXONOMY_SPEC.md).

Provider onboarding rules:

1. If a public, maintained SDK exists, the kernel provider uses that SDK or a typed local adapter.
2. If no stable SDK exists, integration remains plugin-bound and opt-in; product code must not add raw HTTP shortcuts.
3. Every provider must expose health, capability, policy, and conformance evidence before entering the default catalog.
4. Product applications may select engines through agents catalog and runtime binding only.

### 2.6 BirdCoder integration boundary

```text
BirdCoder PC (coding_session*, workbench projection)
        │
        ▼
sdkwork-birdcoder-kernel-bridge  ── MUST NOT depend on sdkwork-agent-kernel
        │
        ▼
sdkwork-agents-runtime-facade (host / turn / catalog / live_interaction)
        │
        ▼
sdkwork-kernel provider plugins
```

Alignment tracker: [`specs/agents-birdcoder-alignment.spec.json`](../../../specs/agents-birdcoder-alignment.spec.json).

BirdCoder ownership remains narrow: `coding_session*`, workbench projection, repository state,
and code-task UI stay in BirdCoder. Shared agent lifecycle, provider catalog, messages,
interactions, tasks, and runtime turns go through `sdkwork-agents`.

## 3. Composition Slot Pattern

`ai_agent_composition_slot` 是跨模块资源引用的核心机制。Agent 不拥有外部资源的数据，
而是通过组合槽引用 independent capability modules 的资源。LLM 不是 composition slot
的默认数据域；LLM 模型选择、供应商、凭证和网关通过 runtime binding、configuration
profile 和 kernel model provider 接入。

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

LLM 示例不进入 composition slot：

| Field | Example | Owner |
| --- | --- | --- |
| `ai_agent_runtime_binding.provider_id` | `provider.model.openai` | `sdkwork-llm` / kernel provider catalog |
| `ai_agent_runtime_binding.configuration_profile_id` | `profile.llm.production` | `sdkwork-llm` |
| `ai_agent_session.model_id` | `gpt-5.1` | LLM model catalog / provider binding |

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

完整 API 规范见 [TECH-api-specification.md](TECH-api-specification.md)。

## 5. Database Design

### 5.1 设计原则

1. **所有表使用 `ai_` 前缀** — 遵循 DATABASE_SPEC.md 智能体域前缀标准
2. **拥有 8 张表** — Agent 组合平面 + 托管会话/消息/交互/任务
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
        ├── ai_agent_session ── ai_agent_message
        │     (托管会话)          (chat turn 消息)
        ├── ai_agent_interaction
        │     (审批 / 用户问答暂停点)
        └── ai_agent_task
              (计划任务 / 外部任务关联)
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
| `ai_agent_interaction` | `(tenant_id, session_id, status, created_at DESC)`, unique `(tenant_id, session_id, interaction_id)` |
| `ai_agent_task` | `(tenant_id, agent_id, owner_user_id, status, updated_at DESC)`, unique `(tenant_id, task_id)` |

### 5.4 约束策略

- **CHECK 约束**：枚举值在数据库层强制（status、visibility、implementation_kind、slot_kind、target_module）
- **UNIQUE 约束**：业务键唯一（tenant_id + agent_id, tenant_id + code, uuid）
- **标准 ID 格式**：正则约束 (`^provider\.`, `^binding\.`, `^slot\.`, `^profile\.`)
- **能力 JSON 验证**：PL/pgSQL 函数验证 capabilities JSON 格式

## 6. API Surfaces

| Surface | Prefix | Audience | Operations | SDK |
| --- | --- | --- | --- | --- |
| Open API | `/agent/v3/api` | 第三方集成方 | 27 | `@sdkwork/agents-sdk` |
| App API | `/app/v3/api` | 前端应用 (H5/PC/Mini Program/Flutter) | 35 | `@sdkwork/agents-app-sdk` |
| Backend API | `/backend/v3/api` | 管理后台 | 33 | `@sdkwork/agents-backend-sdk` |

完整 API 列表（95 个 HTTP 操作）参见 [TECH-api-specification.md](TECH-api-specification.md)，
分组参考与生产契约说明见 [TECH-api-reference.md](TECH-api-reference.md)。
所有成功响应使用 `SdkWorkApiResponse`，错误响应使用 `application/problem+json`
(`ProblemDetail`) 并携带 numeric `code` 和 `traceId`。

### 6.1 Chat Completion

`POST .../sessions/{sessionId}/messages` 是 canonical chat turn 入口：

1. 校验 agent/session 归属与 active 状态
2. 持久化 user message
3. 调用 `AgentsService::send_chat_message` → 可注入 `ChatCompleter`（默认 `ContractChatCompleter`；生产由 gateway 注入 `RuntimeFacadeChatCompleter`，通过 code-engine facade 执行 provider turn）
4. 持久化 assistant message 并更新 session counters
5. 返回 `AgentChatCompletionResponse`（session + userMessage + assistantMessage）

`?stream=true` 时返回 `text/event-stream`。当前 provider 若只支持 invoke，SSE 至少返回一个
`completion` 事件；当 kernel provider 暴露 stream chunks 时，agents 将映射为
`message.delta` 分片并在结束时返回 completion 信封。逐 token streaming 是 P1 kernel/provider
缺口，不改变 chat turn 的持久化权威。

PC 管理面 `managementProfile` 通过 `defaultCodeTaskIntent.constraints` 中的
`sdkwork.agent.pc.config:{json}` 与 OpenAPI 对齐，包含 `knowledgeBaseIds`、`skillIds`、
`toolIds`、`voiceIds`、`memoryEnabled` 等字段。

所有 API 操作通过 `api.rs` 中定义的 `ApiOperation` 元数据驱动路由注册和 OpenAPI 生成。

### 6.2 SDK ownership and consumer policy

| Consumer | Allowed SDK path | Forbidden path |
| --- | --- | --- |
| Third-party integrator | `@sdkwork/agents-sdk` | generated transport package names |
| PC/H5/MP/Flutter | `@sdkwork/agents-app-sdk` through app core SDK exports | deep imports into `generated/server-openapi/src/*` |
| Backend admin | `@sdkwork/agents-backend-sdk` | app/open SDK for admin-only operations |
| BirdCoder runtime bridge | `sdkwork-agents-runtime-facade` | `sdkwork-agent-kernel`, `sdkwork-agent-provider-*` |
| Product UI | app SDK + typed service adapters | raw HTTP to `/internal/v3/api` |

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
pnpm check:api-envelope
pnpm check:api-operation-patterns
pnpm check:route-path-collisions
pnpm check:pagination
pnpm check:app-sdk-consumer-imports
pnpm check:component-port-bindings
pnpm check:frontend-composition
pnpm check:permission-composition
pnpm check:composition-resolver
pnpm check:rust-backend-composition
pnpm check:production-security
node ../sdkwork-birdcoder/scripts/birdcoder-agents-integration-contract.test.mjs
```

`pnpm check:production-security` validates production-like profile gating, IAM/Postgres/runtime-facade bootstrap, standalone gateway shutdown handling, in-memory repository/audit lock recovery, and strict repository ports. Signal handler installation failures and poisoned in-memory locks must be logged for operators and must not panic the running gateway; incomplete persistence adapters must fail at compile time instead of inheriting default empty stubs.

## 10. Launch Readiness

Pre-launch P0/P1 alignment is complete for PC/H5 and BirdCoder code-agent workflows.
Remaining items are commercial GA hardening and kernel/sibling platform capabilities, not a
reason to re-own kernel or sibling module responsibilities inside `sdkwork-agents`.

### Completed (pre-launch)

| Area | Status |
| --- | --- |
| IAM-backed policy + Postgres audit sink | Done |
| 95-operation HTTP surface (27/35/33) + OpenAPI/SDK sync | Done |
| Session/message chat + SSE completion event | Done |
| `code_engine_catalog` + `mcp_marketplace` + `runtime_facade_bridge` | Done |
| HTTP-only DTOs colocated in `http.rs` (no feature-gate dead code) | Done |
| Open API / Open SDK surface boundary (no restore/catalog drift) | Done |
| HTTP route trees aligned to OpenAPI authority (27/35/33, no phantom routes) | Done |
| Interaction HTTP on App/Backend (`agents.interactions.*`; Open API excluded) | Done |
| Legacy MCP/Memory/Knowledge inline types removed | Done |
| Structured audit payloads (agent, binding, runtime, marketplace) | Done |
| PC/H5/MP core `sdkDependencies` + agents-app-sdk wiring | Done |
| PC/H5 core knowledgebase-app-sdk via `*-core/sdk` (capability packages import core only) | Done |
| `pnpm check` gates (composition, component ports, frontend, permissions, Rust backend, API envelope, operation patterns, route collisions, pagination, SDK imports, apps index, production security, deploy, docs, scripts, workflow, topology, database) | Done |
| `pnpm verify` includes SDK build, `--all-features` Rust tests, mini-program runtime build, client typecheck, PC agent + e2e flow contracts, Node platform contracts | Done |
| CI packaging `validate` lifecycle mirrors `pnpm verify` | Done |
| Archive docs trimmed to redirect stubs (no historical body) | Done |
| Flutter core `sdk_inventory.dart` + `component.spec.json` non-GA mobile tracking contract | Done |
| Agents managed-store Prometheus metrics (`/metrics/agents`, RPS gauge) | Done |
| Postgres interaction persistence + fail-closed HTTP state bootstrap | Done |
| PC/H5 production chat UI (`AgentChatView` + sessions/messages API) | Done |
| PC Auth Gate + knowledge bootstrap + runtime catalog + composition slot sync | Done |
| Optional skills/voice/knowledge catalog via sibling app SDKs | Done (server-paged pickers; cursor/offset per authority) |
| Mini-program runtime bundle rebuild in verify (`agents-mini-program build` + runtime contract) | Done |

### List pagination alignment (`PAGINATION_SPEC.md`)

| Layer | Status | Notes |
| --- | --- | --- |
| SQL list authorities | Done | `LIMIT/OFFSET` + `COUNT(*)` per `AGENTS_AI_COMPOSITION_DATABASE_SPEC.md` §8 |
| In-memory test repository | Done | `BTreeMap` indexes + `offset_limit_page_from_iter` (`in_memory_pagination.rs`) |
| HTTP `PageInfo` | Done | `mode=offset`, `totalItems`, `totalPages`, `hasMore` via `sdkwork-utils-rust::http_api` |
| App API `scope` + `q` | Done | OpenAPI `ListScope` + `SearchQ` on `agents.list`; market vs owned lists |
| PC/H5 interactive UI | Done | `listAgentsPage()` one page at a time; chat uses single message page (`pageSize=50`) |
| PC/H5 export/sync | Done | `syncAllOffsetPages()` in `*-core/sdk/pagination` (not for UI tables) |
| Mini program native list | Done | `pageSize=20`, follows `pageInfo.hasMore`, load-more button |

Messages remain **offset mode** by contract today. Cursor/keyset support belongs to a future requirement only when very long sessions require it, and it is not part of the current GA evidence bundle.

### Client surfaces (commercial MVP)

| Surface | Auth + CRUD + Chat | Catalog / composition | Notes |
| --- | --- | --- | --- |
| PC | Done | Done | Production path |
| H5 | Done | Done | Synced from PC via `workflow:sync-agent-h5-from-pc` |
| Mini program | Runtime SDK bootstrap | Native agents list + WebView editor bridge | Native list via App API; full editor in WebView |
| Flutter | Scaffold only | Out of GA scope | No owned Dart agents-app-sdk facade yet |

### Non-GA Scope (Owned Outside Current Release)

| Priority | Item | Owner |
| --- | --- | --- |
| P1 | Token-level SSE streaming | kernel `ModelProvider::stream` → agents SSE |
| P1 | Task run projection (`ai_agent_task_run` + `agents.taskRuns.*`) | kernel `AgentRun` / `AgentStep` → sdkwork-agents projection |
| P1 | Live interaction SPI ownership migration | sdkwork-kernel |
| P1 | Rate limit + CORS middleware | sdkwork-web-framework |
| P2 | T2 engines (openclaw, hermes) in default catalog | agents facade + kernel conformance |
| P2 | Rig live backend (feature-gated) | sdkwork-agent-provider-rig |
| P2 | Split `http.rs` / `persistence.rs` for maintainability | sdkwork-agents |
| P2 | Grafana dashboards wired to `/metrics` + `/metrics/agents` | ops |
| P2 | MCP marketplace federation HTTP | sdkwork-mcp sibling mount |
| P2 | Direct open SDK sdkgen (`/agent/v3/api` profile) | sdkwork-sdk-generator |
| P2 | Flutter Dart agents-app-sdk | sdkwork-agents Flutter app |
