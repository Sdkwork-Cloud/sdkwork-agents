## 修复 Playground 生成页面两类报错

### 根因分析（已确认）

**错误 1：`generations app SDK base URL is not configured`（CreativeView 生成页加载即报）**
- `generationsAppSdkClient.ts:56-58`：`resolveGenerationsAppSdkBaseUrl()` 只读取两个 VITE_ 环境变量，缺任一即返回 null → 抛错。
- 对比 `agentsAppSdkClient.ts:35-46`：agents SDK 有 `window.location.origin` 回退，所以在当前 3901 网关环境（cloudrouter 统一入口，与 cloud 拓扑 `api-dev.sdkwork.com` 网关模式一致）下 agents API 正常，而 generations SDK 因无回退直接崩溃。
- 修复：无环境变量时回退到 `resolveAgentsAppSdkBaseUrl()`（同网关入口，含 `VITE_SDKWORK_AGENTS_PC_APPLICATION_PUBLIC_HTTP_URL` → `window.location.origin` 回退链）。

**错误 2：`403 iam.permission.missing:ai.agents.manage`（聊天发送消息即报）**
- 未提交的新代码 `ensureSessionChatRuntime`（PC + H5 AgentChatService）在聊天路径上主动调用 `providerBindings.create`。
- 后端刻意将 `provider_binding.add` 限制为 `ai.agents.manage`（`infrastructure.rs` 的 `SELF_SERVICE_POLICY_ACTIONS` 不含它，且 `iam_gated_provider_keeps_management_actions_behind_manage_permission` 测试明确断言）。当前演示用户只有 use 级 scope → 403 → 整条消息发送失败。
- 注意：`session_runtime_binding.create` 是自助操作（`ai.agents.use`），聊天路径创建会话运行时绑定本身没问题。
- 修复：聊天路径对 provider binding 创建改为 best-effort（容忍 403/409）；创建后重新拉取，若仍无目标模型的 active binding，则抛出明确的用户可操作错误（不再让 403 裸崩）；有 binding 时继续原有的自助式 session runtime binding 逻辑。

### 改动文件

1. `apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/sdk/generationsAppSdkClient.ts`
   - `resolveGenerationsAppSdkBaseUrl()`：env 读取失败后回退 `resolveAgentsAppSdkBaseUrl()`（从 `./agentsAppSdkClient` 导入；无循环依赖）。

2. `apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/src/services/AgentChatService.ts`（`ensureSessionChatRuntime`）
   - 扩展错误容忍：403（无 manage 权限）与 409（已存在）均跳过创建。
   - 创建后重新 `list` 校验 active binding；仍缺失时抛出明确中文错误（如“该 Agent 尚未绑定模型引擎，请先在编辑器中发布/绑定模型后重试”）。
   - `isProviderBindingConflict` 改造为同时识别 403/409 的判定函数。

3. `apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents/src/services/AgentChatService.ts`
   - 与 PC 完全一致的修复（h5 有同款未提交改动）。

4. `apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/src/pages/HomeAgentConversation.tsx`
   - `sendMessage` 失败时展示 `sendError.message`（服务层已抛用户可读中文错误），保留通用文案兜底。

5. `apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents/src/pages/AgentChatView.tsx`
   - toast 改为展示具体错误信息（有 Error message 时）。

### 验证
- PC：`pnpm typecheck`（tsc --noEmit）+ `pnpm test:agent-contract`（`tests/agent-catalog.test.ts`、`sdk-services.contract.test.ts` 等，确认无契约回归）。
- H5：对应 package `pnpm lint` / typecheck。
- 手工确认生成页（CreativeView 会话列表）不再抛 base URL 错误；聊天在无 binding 时给出明确提示而非 403。

### 不做的事
- 不改后端 IAM 边界（`provider_binding.add` 保持 manage-only，有测试钉住）。
- 不改 dev 环境的权限授予（IAM 授权属于 sdkwork-iam/cloudrouter 域，超出本仓库）。