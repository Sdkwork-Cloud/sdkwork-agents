# 聊天文件库：sdkwork-drive 打标 + 按标记查询

## 目标
1. 聊天上传的文件（图片/附件/视频/语音）在通过 sdkwork-drive 上传后打上标记（property `agents.chat_file_library` = "1"，visibility `app_public`）
2. sdkwork-drive 新增 App API「按标记反查节点列表」，供文件库（资料库）页面拉取真实数据
3. `FileLibraryView.tsx` 从 mock 改为真实数据（drive 标记文件列表 + 搜索 + 图片/文件筛选 + 下载预览）

## 一、sdkwork-drive 仓库改动（跨仓库，遵守 drive 仓库 AGENTS.md 与验证管线）

### 1. 新增 App API：按属性标记反查节点列表
- 端点：`GET /app/v3/api/drive/properties/{property_key}/nodes`，operationId `propertyNodes.list`
- 语义：租户范围内返回满足 `dr_drive_node_property.property_key = {key} AND visibility='app_public' AND lifecycle_status='active'` 且调用者具备 reader 权限的节点列表（复用 `present_node_list` + `acl_sql::reader_inherited_permission_exists_sql` 的 ACL 模式，参照 `library_handlers.rs::list_recent_nodes`；排序 `updated_at DESC, id ASC`；分页 cursor 与现有 list 端点一致）
- 改动文件：
  - `crates/sdkwork-routes-drive-app-api/src/dto.rs`：新增 `PropertyNodeListQuery`（page_size/cursor）
  - `crates/sdkwork-routes-drive-app-api/src/metadata_handlers.rs`：新增 `list_property_nodes` handler（JOIN `dr_drive_node_property`，复用 `NODE_API_SELECT_COLUMNS`）
  - `crates/sdkwork-routes-drive-app-api/src/routes.rs`：注册路由
  - `database/ddl/baseline/postgres/0001_drive_baseline.sql`：新增反向索引 `ix_dr_drive_node_property_key (tenant_id, property_key, visibility, lifecycle_status, node_id)`
  - `apis/app-api/drive/drive-app-api.openapi.json`：新增操作（复用 DriveNode 页面信封 schema）
- 生成与验证（drive 仓库命令）：`pnpm api:envelope:check`（对齐 openapi 信封 + materialize）、`pnpm sdk:generate`（重新生成 TS/Rust/Java/Python/Go SDK；route manifest 由 openapi 生成）、`cargo check`、相关 cargo 测试（`command_routes.rs` 增补端点测试）、`pnpm db:materialize:contract` + `pnpm db:validate`、`pnpm check:app-sdk-consumers`
- 兼容性：纯新增端点 + 新增索引，非破坏性；`@sdkwork/drive-app-sdk` 经 pnpm workspace 链接，agents 前端自动获得新方法

### 2. 打标机制（drive 现有能力，无需改动）
现有 `PUT /app/v3/api/drive/nodes/{nodeId}/properties/{propertyKey}`（upsert、`app_public`）即可完成打标。

## 二、sdkwork-agents 仓库改动（前端）

### 1. 上传后自动打标
- `apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-core/src/sdk/driveUploadService.ts`：
  - `UPLOAD_POLICIES` 为 `agent-chat-attachment/image/video/voice` 增加标记契约字段（key `agents.chat_file_library`）
  - `upload()` 在 `uploadByProfile` 成功后调用 `client.drive.nodeProperties.update(nodeId, "agents.chat_file_library", { value: "1", visibility: "app_public" })`
  - 打标失败仅 console.warn（best-effort，不影响上传与发消息）；标记契约以常量 + 注释声明

### 2. 文件库数据服务
- 新增 `apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-chat/src/services/chatFileLibraryService.ts`：
  - `listChatLibraryFiles({ pageSize, cursor })` → `client.drive.propertyNodes.list("agents.chat_file_library", ...)`（方法名以重新生成的 SDK 为准）
  - 映射为展示项：nodeId / name / mimeType / sizeBytes / updatedAt / spaceId
  - `resolvePreviewUrl(nodeId)` 复用现有 `downloadUrls.retrieve` 模式

### 3. FileLibraryView 真实数据
- `apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-chat/src/components/FileLibraryView.tsx`：
  - 移除 mockFiles；挂载时从 drive 拉取（loading / error / empty 状态 + 刷新按钮）
  - 保留「全部/图片/文件」tab（按 mimeType 过滤）与本地搜索
  - 名称/修改时间/大小映射自节点元数据；点击文件下载/预览

### 4. 文档
- `specs/README.md` 集成表补充文件库标记契约说明（`agents.chat_file_library`，app_public）；drive 侧 openapi 权威由生成管线同步

## 三、验证
- drive：`pnpm api:envelope:check`、`pnpm sdk:generate`、`cargo check` + 相关测试、`pnpm db:validate`、`pnpm check:app-sdk-consumers`
- agents：pc-core / pc-chat 类型检查与构建（pnpm 对应包构建命令），`pnpm verify` 窄范围跑相关检查
- 手工验证路径（如环境允许）：上传聊天文件 → 检查 drive 节点出现 `agents.chat_file_library` 属性 → 文件库列表出现该文件

## 四、范围外（后续可选）
- 存量聊天文件（本功能上线前上传的）不回填标记；如需回填可另立任务（扫描 `ai_agent_item_drive_ref` 补标）
- 文件库不做会话维度分组（当前为全局资料库，与 mock 语义一致）

## 风险与注意
- drive SDK 重新生成会被 sdkwork-im 等其他 workspace 消费者共享 —— 纯新增方法，非破坏性
- 两个仓库分开提交；执行前确认 drive 仓库 git 状态
- 新端点方法名/响应字段以重新生成的 SDK 实际签名为准