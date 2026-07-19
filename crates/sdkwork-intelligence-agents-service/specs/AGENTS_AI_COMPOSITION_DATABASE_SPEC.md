# SDKWork Agents AI Composition Database Specification

- Version: `4.0.0`
- Status: active commercial Chat/Project contract
- Domain: `intelligence`
- Capability: `agents`
- Owner: `agents-platform`
- Compliance: L2 (core), L3 (audit)
- Related: `database/contract/schema.yaml`, `DATABASE_SPEC.md`, `DATABASE_FRAMEWORK_SPEC.md`

## 1. Scope

`sdkwork-agents` owns the **agent composition plane** only: agent identity, runtime bindings,
composition slot references, and audit facts.

Content and domain persistence are owned by sibling modules:

| Concern | Owner module | Prefix |
| --- | --- | --- |
| Knowledge / RAG | `sdkwork-knowledgebase` | `kb_` |
| Memory | `sdkwork-memory` | `ai_` (memory-owned) |
| Skills | `sdkwork-skills` | `ai_skill_*` |
| Prompts | `sdkwork-prompts` | `ai_prompt_*` |
| Files | `sdkwork-drive` | `dr_` |
| MCP | `sdkwork-mcp` | `ai_mcp_*` |
| Cross-resource search index/query history | `sdkwork-search` | `search_` |
| Image/video/music/voice generation jobs/results | `sdkwork-generations` | `generation_` |

**Hosted chat persistence** (`ai_agent_session`, `ai_agent_message`, `ai_agent_interaction`)
is owned by `sdkwork-agents` and is the canonical product read model for session/message
query APIs. Kernel may maintain optional in-process runtime state in
`sdkwork-agent-database` for provider execution; that store is not the product authority.

**Task scheduling persistence** (`ai_agent_task`) is owned by `sdkwork-agents` for
product-managed tasks. Kernel `AgentRun` / `AgentStep` projection (`ai_agent_task_run`,
`agents.taskRuns.*`) remains non-GA scope until the kernel run projection is stable;
see `specs/AGENTS_KERNEL_SPI_GAP_ANALYSIS.md`.

Reusable agent/project orchestration configuration references sibling modules
through agent-level `ai_agent_composition_slot` or target project-level
`ai_agent_project_composition_slot`. Runtime model selections, message media,
search publication, and generation source correlation use the dedicated
relations defined by their owning module contracts.
No MCP, knowledge, memory, skills, prompts, or drive tables exist in this repository.

## 2. Design Principles

1. Every agents-owned table uses the `ai_` module prefix per `DATABASE_SPEC.md`.
2. Internal identifiers are `int64`; API serialization uses string.
3. No plaintext secrets in business tables; references only (`profile.*`, `endpoint.*`).
4. Soft-delete plus audit trace for management mutations.
5. `tenant_id` and `organization_id` are explicit columns on all tenant entities.
6. Snowflake IDs are allocated in application code before insert (no `BIGSERIAL` / `RETURNING id`).
7. Cross-module resources are referenced through scoped composition slots, not duplicated tables.
8. The immutable `3.1.0` baseline contains 8 tables; active contract `4.0.0`
   contains 17 tables after versioned migrations. Chat/project tables own Agents
   product state, remove JSON/query debt, and preserve the dependency boundary in
   `specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md`.

## 3. Table Overview

| Table | Profile | Compliance | Responsibility |
| --- | --- | --- | --- |
| `ai_agent` | `tenant_entity` | L2 | Agent identity, manifest snapshot, lifecycle, visibility. |
| `ai_agent_runtime_binding` | `tenant_entity` | L2 | Provider/runtime binding for an agent. |
| `ai_agent_composition_slot` | `tenant_entity` | L2 | Agent → sibling-module resource references. |
| `ai_agent_audit_event` | `audit_log` | L3 | Immutable management audit facts. |
| `ai_agent_session` | `tenant_entity` | L2 | Hosted chat sessions (tenant/agent/owner scope). |
| `ai_agent_message` | `tenant_entity` | L2 | Session messages and chat turn persistence. |
| `ai_agent_interaction` | `tenant_entity` | L2 | Live interaction state (approval / user-question flows). |
| `ai_agent_task` | `tenant_entity` | L2 | Product-managed task scheduling and external task correlation. |

## 3.1 Active Chat And Project Contract

The following `4.0.0` contract is active design authority. Its DDL is applied by
versioned migrations, and repository, OpenAPI, SDK, frontend, IM consumer,
manifest, schema registry, and PostgreSQL evidence are verified together.

| Table | Change | Responsibility |
| --- | --- | --- |
| `ai_agent_session` | extend | Add project ownership, stable keyset ordering, atomic message sequence allocation, idempotency, audit, archive/delete, and retention fields. It MUST NOT store IM identifiers. |
| `ai_agent_message` | extend | Add organization/owner/sender/turn scope, lifecycle fields, parent integrity, and session-agent consistency. Typed Drive relations replace opaque media JSON. |
| `ai_agent_audit_event` | extend | Cover project/session/message/turn/share mutations and remain immutable across resource deletion. |
| `ai_agent_project` | new | Agent chat project identity, owner, visibility, lifecycle, and default orchestration policy. |
| `ai_agent_project_composition_slot` | new | Project references to prompts, memory, knowledgebase, skills, MCP, and Drive without copying sibling-owned content. |
| `ai_agent_chat_turn` | new | Idempotent inference command, running/completed/failed/cancelled state, request/response message links, usage, errors, trace, and timing. |
| `ai_agent_message_drive_ref` | new | Ordered Drive/MediaResource business relations for attachments and generated outputs. |
| `ai_agent_message_feedback` | new | Per-user positive/negative response feedback without mutating the immutable message body. |
| `ai_agent_resource_user_state` | new | Per-user pinned, hidden, last-opened, last-read, and custom-title state for Agents sessions/projects. |
| `ai_agent_project_member` | new | Agents project collaboration only; not an IM group or conversation membership table. |
| `ai_agent_share_link` | new | Hashed, expiring, revocable, least-privilege share grants for Agents resources. |
| `ai_agent_outbox_event` | new | Reliable Agents-owned events for search, notifications, and external consumers. |

The active Agents inventory is 17 tables: the baseline 8 plus 9 migrated tables above.
`sdkwork-im` owns all conversation-agent assignment, dispatch, visible reply, and
IM-to-Agents correlation tables.

### 3.1.1 Common Target Columns And Scope

Every new mutable entity uses application-allocated `BIGINT id`, public `uuid`,
`tenant_id BIGINT`, `organization_id BIGINT`, optimistic `version BIGINT`, and
`created_at`/`updated_at TIMESTAMPTZ`. User/actor columns are `BIGINT` and come
from trusted request context. Stable API ids such as `project_id`, `session_id`,
`message_id`, and `turn_id` remain bounded strings and are unique inside explicit
tenant/organization scope. Public APIs serialize all int64 values as strings.

Soft-deletable rows use `deleted_at`, `deleted_by`, and `retention_until`.
Archivable rows additionally use `archived_at` and `archived_by`. Physical
foreign keys are allowed only between Agents-owned tables and must include the
tenant/organization scope needed to prevent cross-scope attachment. No
`ON DELETE CASCADE` may erase messages, turns, feedback, audit, or outbox facts.

### 3.1.2 `ai_agent_project`

Required columns:

```text
id, uuid, tenant_id, organization_id, project_id
owner_user_id, name, description
visibility                    # private, organization, shared
status                        # active, archived, deleted
drive_access_mode             # none, owner_private, project_shared
default_agent_id, default_model_id
created_by, updated_by, version
created_at, updated_at, archived_at, archived_by
deleted_at, deleted_by, retention_until
```

`default_agent_id` is a scoped Agents reference. `default_model_id` is an opaque
LLM model reference. Project instructions, memory, knowledge, skills, MCP, and
Drive folders are not copied into this row; they use composition slots below.
Required keys and indexes:

- unique `(tenant_id, organization_id, project_id)`;
- owner list `(tenant_id, organization_id, owner_user_id, status,
  updated_at DESC, id DESC)`;
- organization list `(tenant_id, organization_id, visibility, status,
  updated_at DESC, id DESC)`;
- normalized-name lookup scoped by tenant/organization for bounded project-local
  filtering. Cross-resource ranking, query history, suggestions, and semantic
  search are indexed in `sdkwork-search` from Agents outbox events.

Changing `drive_access_mode` to `project_shared` requires Drive authorization and
must never expose an owner's personal Drive space implicitly. A shared project
cannot retain `owner_private` access.

### 3.1.3 `ai_agent_project_composition_slot`

Required columns:

```text
id, uuid, tenant_id, organization_id, project_id, slot_id
slot_kind                     # prompt, memory, knowledge, skill, mcp, drive, tool
target_module, target_ref, target_version_ref
priority, enabled, policy_json
created_by, updated_by, version
created_at, updated_at, deleted_at, deleted_by, retention_until
```

Keys: unique `(tenant_id, organization_id, project_id, slot_id)` and lookup index
`(tenant_id, organization_id, project_id, slot_kind, enabled, priority, id)`.
`policy_json` contains only bounded orchestration overrides and no copied content
or credentials. The project instruction editor creates or updates a private
resource through `sdkwork-prompts` and stores its stable prompt/version reference
here. Memory, knowledgebase, skills, MCP, and Drive are reused in the same way.

### 3.1.4 `ai_agent_session` Extension

Add these columns to the existing row:

```text
project_id nullable
title non-null, title_source   # default, generated, manual
last_message_sequence
idempotency_key, payload_hash
created_by, updated_by
archived_at, archived_by
deleted_at, deleted_by, retention_until
```

Session creation idempotency is unique on `(tenant_id, organization_id,
owner_user_id, idempotency_key)` when the key is present. `project_id` references
an active scoped project or is null. Moving a chat between projects updates only
this reference and audit/outbox rows. Project deletion archives its active
sessions in the same service transaction; it does not physically delete them.

List indexes:

- personal history `(tenant_id, organization_id, owner_user_id, status,
  updated_at DESC, id DESC)`;
- project history `(tenant_id, organization_id, project_id, status,
  updated_at DESC, id DESC)`;
- agent history `(tenant_id, organization_id, agent_id, status,
  updated_at DESC, id DESC)`.

All lists use keyset cursors containing the complete sort tuple. Pinned ordering
comes from `ai_agent_resource_user_state`; it is not stored as one shared session
boolean.

### 3.1.5 `ai_agent_chat_turn`

Required columns:

```text
id, uuid, tenant_id, organization_id, turn_id
session_id, agent_id, owner_user_id
client_request_id, idempotency_key, payload_hash
request_message_id, response_message_id
mode                          # chat, image, web_search, deep_research, tool
status                        # pending, running, completed, failed, cancelled
requested_model_id, provider_binding_id, model_id, provider_id
input_tokens, output_tokens, cached_tokens
finish_reason, error_code, error_detail, trace_id
version
created_at, updated_at, started_at, completed_at
cancel_requested_at, cancelled_at, retention_until
```

Required keys and indexes:

- unique `(tenant_id, organization_id, turn_id)`;
- unique `(tenant_id, organization_id, owner_user_id, idempotency_key)`;
- optional unique `(tenant_id, organization_id, owner_user_id,
  client_request_id)` when supplied;
- session timeline `(tenant_id, organization_id, session_id, created_at, id)`;
- worker/reconciliation `(tenant_id, organization_id, status, updated_at, id)`;
- trace lookup `(tenant_id, organization_id, trace_id)` when non-null.

The idempotency key may come from any consumer. Agents stores the opaque key and
payload hash only; it does not decode or persist an IM conversation/message id.
A repeated key with a different hash is a conflict. Error detail is sanitized and
bounded; provider secrets and raw credentials are forbidden.

### 3.1.6 `ai_agent_message` Extension

Add these columns to the existing row:

```text
organization_id, owner_user_id
sender_type                   # user, assistant, system, tool
sender_user_id nullable
turn_id nullable
created_by
deleted_at, deleted_by, retention_until
```

The existing `role` is migrated to `sender_type` and removed only after contract
consumers switch. Existing `artifacts_json` becomes compatibility-read-only and
is removed after every durable file/media value is represented by
`ai_agent_message_drive_ref`. Message content remains immutable after completion;
redaction changes lifecycle/status fields and writes audit rather than silently
rewriting history.

Constraints:

- unique `(tenant_id, organization_id, message_id)`;
- unique `(tenant_id, organization_id, session_id, sequence)`;
- session, agent, owner, parent message, and turn references must share scope;
- `sender_type = user` requires trusted `sender_user_id`; other sender types must
  not impersonate a user;
- request and response messages linked from a turn must belong to that turn's
  session and agent.

Message history uses `(tenant_id, organization_id, session_id, sequence, id)`.
Database-side scoped content lookup may serve a bounded fallback, but the target
cross-resource search authority is `sdkwork-search`, fed from outbox events.
Loading every session into application memory and filtering is forbidden.

### 3.1.7 `ai_agent_message_drive_ref`

Required columns follow `MEDIA_RESOURCE_SPEC.md`:

```text
id, uuid, tenant_id, organization_id
message_id, media_role        # attachment, image, voice, generated_output, artifact
drive_space_id, drive_node_id, drive_uri
media_resource_id, object_blob_id nullable
resource_snapshot, resource_hash
alt_text, sort_order, status
created_by, created_at, updated_at, deleted_at, retention_until
```

Keys: unique `(tenant_id, organization_id, message_id, drive_node_id,
media_role)` and list index `(tenant_id, organization_id, message_id, status,
sort_order, id)`. `resource_snapshot` is a bounded `MediaResource` projection;
Drive remains authoritative for bytes, versions, scanning, grants, access, and
retention. Object keys, bucket names, credentials, provider URLs, and presigned
URLs are forbidden. Generated or edited artifacts are uploaded through Drive and
linked with `generated_output` or `artifact` role.

### 3.1.8 `ai_agent_message_feedback`

Required columns:

```text
id, uuid, tenant_id, organization_id
message_id, user_id
rating                        # up, down
reason_code nullable, comment nullable
version, created_at, updated_at, deleted_at
```

Unique `(tenant_id, organization_id, message_id, user_id)` supports idempotent
toggle/update. List/analytics index `(tenant_id, organization_id, rating,
created_at DESC, id DESC)`. Feedback is user-owned product data, not embedded in
message metadata and not modeled as an IM reaction.

### 3.1.9 `ai_agent_resource_user_state`

Required columns:

```text
id, uuid, tenant_id, organization_id
user_id, resource_type, resource_id   # project or session
pinned_at, hidden_at, last_opened_at
last_read_message_sequence nullable
custom_title nullable
version, created_at, updated_at
```

Unique `(tenant_id, organization_id, user_id, resource_type, resource_id)`.
Pinned/recent list index `(tenant_id, organization_id, user_id, resource_type,
pinned_at DESC, last_opened_at DESC, id DESC)`. The service validates that the
referenced project/session exists and is visible to the user. This table replaces
browser-only pinned state for signed-in users; local state may remain only as an
offline cache.

### 3.1.10 `ai_agent_project_member`

Required columns:

```text
id, uuid, tenant_id, organization_id
project_id, member_user_id
role                          # owner, editor, viewer
status                        # invited, active, suspended, removed
invited_by, joined_at, removed_at
version, created_at, updated_at, retention_until
```

Unique `(tenant_id, organization_id, project_id, member_user_id)` and member list
index `(tenant_id, organization_id, member_user_id, status, updated_at DESC, id
DESC)`. `ai_agent_project.owner_user_id` is the ownership authority; an owner
member row is maintained transactionally. Ownership transfer is audited. This is
project collaboration only and must not add presence, read receipts, reactions,
channels, or generic group messaging.

### 3.1.11 `ai_agent_share_link`

Required columns:

```text
id, uuid, tenant_id, organization_id, link_id
target_type, target_id        # project or session
permission                    # view or use
token_hash, token_prefix
status                        # active, revoked, expired
created_by, expires_at, revoked_at, revoked_by
max_uses nullable, use_count, last_used_at
created_at, updated_at, retention_until
```

Unique scoped `link_id` and unique `token_hash`; active-target index `(tenant_id,
organization_id, target_type, target_id, status, expires_at, id)`. Only the hash
is persisted. Raw tokens are returned once, never logged, and never placed in an
outbox payload. Use-count changes are atomic and authorization always rechecks
target lifecycle and permission.

This grant authorizes only an Agents project/session. It cannot grant access to a
Drive node. Shared project files require the corresponding Drive permission or
Drive-owned `dr_drive_node_share_link` flow.

### 3.1.12 `ai_agent_outbox_event`

Required columns:

```text
id, uuid, tenant_id, organization_id, event_id
aggregate_type, aggregate_id, aggregate_version
event_type, payload_json, headers_json, dedupe_key
status                        # pending, leased, published, failed, dead_letter
attempt_count, max_attempts, available_at
lease_owner, lease_expires_at
published_at, last_error_code, last_error_detail
created_at, updated_at, retention_until
```

Unique `(tenant_id, organization_id, event_id)` and `dedupe_key`; worker index
`(status, available_at, lease_expires_at, id)` supports `FOR UPDATE SKIP LOCKED`.
Outbox rows are committed with their aggregate mutation. Payloads contain stable
Agents references and bounded snapshots only. Search indexing, notification, and
IM consumption are downstream concerns; publishing never writes another module's
tables.

### 3.1.13 Turn And Message Transactions

1. Session creation locks or inserts the idempotency scope and initializes
   `last_message_sequence = 0`.
2. Sending input locks the session row, allocates the next sequence, inserts the
   user message and Drive relations, inserts a `pending`/`running` turn, updates
   session counters/timestamps, and writes audit/outbox in one transaction.
3. Inference runs outside the database transaction.
4. Completion locks the session, allocates the assistant sequence, inserts or
   finalizes the assistant message and Drive relations, marks the turn completed,
   updates usage/session counters, and writes outbox in one transaction.
5. Failure or cancellation preserves the user message and records a terminal turn
   state. Cancellation is compare-and-set and cannot overwrite completion.

Production code MUST NOT use unlocked `MAX(sequence) + 1`. All optimistic updates
include `version`; all retries re-read the terminal state before repeating external
effects.

### 3.1.14 Frontend Capability Traceability

| Frontend capability | Database authority | Reused module / rule |
| --- | --- | --- |
| New chat, history, rename, archive, delete | `ai_agent_session`, audit, outbox | Keyset pagination; soft lifecycle. |
| Pin, recent/open state, optional custom title | `ai_agent_resource_user_state` | Per-user, never one global session flag. |
| Projects, move chat, project home | `ai_agent_project`, `ai_agent_session.project_id` | Move is a scoped reference update. |
| Project instructions | `ai_agent_project_composition_slot` | Content/version owned by `sdkwork-prompts`. |
| Project memory, knowledge, skills, MCP/tools | project composition slot | Stable sibling SDK references only. |
| Project file-library access | project `drive_access_mode` plus drive slot | Files/search/grants owned by `sdkwork-drive`. |
| Text/image/search/research/tool send modes | `ai_agent_chat_turn.mode` | Provider execution remains Agents/kernel boundary. |
| Stop generation, retry reconciliation, errors | `ai_agent_chat_turn` lifecycle/idempotency | Compare-and-set cancellation. |
| Ordered user/assistant/tool messages | `ai_agent_message` | Session row allocates sequence. |
| Image/file/voice input and generated artifacts | `ai_agent_message_drive_ref` | Drive Uploader plus `MediaResource`; no URL persistence. |
| Copy/preview/download | No new product row | Read message/Drive; delivery grants are Drive-owned. |
| Thumbs up/down | `ai_agent_message_feedback` | Not IM reaction state. |
| Search chats/projects/content | `sdkwork-search` projection fed by Agents outbox | Drive file search remains Drive-owned. |
| Project members and share | project member/share link | Not an IM group/chat. |
| Start generic group chat | No Agents table | Optional host port implemented by consuming `sdkwork-im`. |

Input drafts, selected input mode, open panels, scroll position, copied-toast
state, and upload progress are transient client state. They are deliberately not
server tables. IAM profile/security, membership/billing/usage plans, LLM catalog,
prompt content, memory content, knowledge indexes, MCP servers, and Drive files
remain owned by their existing modules.

### 3.1.15 Reuse Decisions

| Concern | Decision |
| --- | --- |
| Project instructions | Reuse `sdkwork-prompts`; Agents stores only a project composition reference. |
| Memory/RAG/skills/MCP/tools | Reuse their public SDKs and stable refs; no copied tables. |
| File upload/library/download/share | Reuse `sdkwork-drive`; Agents owns only message/project business relations. |
| Chat/project cross-resource search | Reuse `sdkwork-search` documents/indexes/query history through outbox ingestion; no `ai_agent_search_*` tables. |
| Image/video/music/voice generation | Reuse `sdkwork-generations` records/jobs/results. Its source-reference row points to the Agents turn; completed MediaResources are linked to the message through Drive refs. No Agents generation-job table. |
| Model/provider catalog | Reuse `sdkwork-llm`; Agents persists stable selected/runtime references and usage snapshots only. |
| Login/profile/security | Reuse `sdkwork-iam`; trusted subject ids only. |
| Commercial plan/entitlements | Reuse `sdkwork-membership`. That module owns subscriptions and entitlements, not project ACL, so `ai_agent_project_member` remains Agents-owned. |
| Drive node share | Reuse Drive permissions/share links. `ai_agent_share_link` remains necessary only for Agents project/session grants and cannot authorize file bytes. |
| Generic comments/reactions | Reuse `sdkwork-comments` or `sdkwork-im` when that product capability is required. Binary assistant quality feedback remains `ai_agent_message_feedback`. |
| Generic group chat | Host integration through consuming `sdkwork-im`; no Agents IM tables or dependency. |

Target dependencies on Search and Generations become active in
`component.spec.json#contracts.sdkDependencies` only when their SDK/facade
integration is implemented. A design proposal must not claim an inactive runtime
dependency as already wired.

### 3.1.16 Cross-Module Correlation

Agents returns stable session, message, and turn identifiers. It does not persist
the caller's IM conversation/message identifiers. IM owns that mapping and uses
Agents idempotency keys for retry safety. There are no cross-database foreign
keys or cross-module table writes.

## 4. `ai_agent_composition_slot`

Binds an agent to an external module resource without copying domain data.

| Column | Type | Description |
| --- | --- | --- |
| `slot_id` | `VARCHAR(128)` | Stable id `slot.{kind}.{name}` |
| `slot_kind` | `VARCHAR(64)` | `memory`, `knowledge`, `skill`, `prompt`, `drive`, `tool`, `mcp` |
| `target_module` | `VARCHAR(64)` | `memory`, `knowledgebase`, `skills`, `prompts`, `drive`, `mcp` |
| `target_ref` | `VARCHAR(256)` | External stable reference (e.g. mem space id, kb space id, mcp server id) |
| `target_version_ref` | `VARCHAR(128)` | Optional pinned version |
| `policy_json` | `JSONB` | Slot-level overrides (non-secret) |
| `priority`, `enabled` | | Orchestration order |

Indexes:

- `uk_ai_agent_composition_slot_tenant_agent_slot`
- `idx_ai_agent_composition_slot_lookup`

Composition slot management audit actions recorded in `ai_agent_audit_event`:

- `composition_slot_created`
- `composition_slot_updated`
- `composition_slot_deleted`

## 5. Audit Actions

The active baseline records agent-core management actions. No MCP, memory,
knowledge, prompt, or Drive domain action is copied into Agents audit.

| Action | Description |
| --- | --- |
| `created` | Agent created |
| `updated` | Agent updated |
| `deleted` | Agent soft-deleted |
| `restored` | Agent restored |
| `status_changed` | Agent status changed |
| `provider_binding_changed` | Provider binding created/updated/activated |
| `composition_slot_created` | Composition slot created |
| `composition_slot_updated` | Composition slot updated |
| `composition_slot_deleted` | Composition slot deleted |

The `4.0.0` target additionally requires immutable audit facts for:

```text
project_created, project_updated, project_archived, project_deleted
project_member_added, project_member_role_changed, project_member_removed
project_composition_slot_created, project_composition_slot_updated,
project_composition_slot_deleted
session_created, session_renamed, session_moved, session_archived,
session_deleted
turn_requested, turn_cancel_requested, turn_completed, turn_failed,
turn_cancelled
message_redacted, message_feedback_changed
share_link_created, share_link_revoked, share_link_expired
```

Audit payloads contain resource ids, lifecycle transitions, trusted actor scope,
request/trace ids, and bounded change summaries. They do not contain full prompts,
message bodies, file bytes, raw share tokens, credentials, or signed URLs.

## 6. Schema lifecycle

The current greenfield application is governed by the `3.1.0` baseline at
`database/ddl/baseline/postgres/0001_agents_baseline.sql`. The repository has no legacy
installation migration path because it has not been released. Future migrations must be
strictly additive or use an approved expand/contract plan and must not repeat baseline
constraints, indexes, functions, or columns.

## 7. Authority

Canonical DDL: `database/ddl/baseline/postgres/0001_agents_baseline.sql`
Contract registry: `database/contract/table-registry.json`
Schema contract: `database/contract/schema.yaml`

## 8. List Query Pagination (DATABASE_SPEC §16)

All agents list/search endpoints **must** push filtering, sorting, `LIMIT`, and `OFFSET` to SQL.
Application and frontend layers must not load full result sets and slice in memory.

| List surface | SQL authority | Notes |
| --- | --- | --- |
| `ai_agent` | `SQL_LIST_AGENT` + `SQL_COUNT_AGENT` | Filters: tenant, org, owner, deleted, `q`, `visibility`. Stable sort: `updated_at DESC, id DESC`. |
| `ai_agent_runtime_binding` | `SQL_LIST_AGENT_PROVIDER_BINDINGS` + `SQL_COUNT_AGENT_PROVIDER_BINDINGS` | Per-agent bindings; sort: active desc, updated_at desc. |
| `ai_agent_composition_slot` | `SQL_LIST_AGENT_COMPOSITION_SLOTS` + `SQL_COUNT_AGENT_COMPOSITION_SLOTS` | Excludes soft-deleted slots. |
| MCP marketplace projection | `SQL_LIST_MCP_MARKETPLACE_SLOTS` + `SQL_COUNT_MCP_MARKETPLACE_SLOTS` | Join `ai_agent` + `slot_kind = mcp`; no N+1 agent scan. |
| `ai_agent_audit_event` | `SQL_LIST_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID` + `SQL_COUNT_AUDIT_EVENTS_BY_TENANT_AND_AGENT_ID` | Filters: `action`, `from`, `to`; sort: `created_at DESC, id DESC`. |
| `ai_agent_session` | `SQL_LIST_AGENT_SESSIONS` + `SQL_COUNT_AGENT_SESSIONS` | Offset-paginated; `PageInfo.totalItems` from count query. |
| `ai_agent_message` | `SQL_LIST_AGENT_MESSAGES` / `SQL_LIST_AGENT_MESSAGES_RECENT_CONTEXT` + `SQL_COUNT_AGENT_MESSAGES` | API lists use ASC + offset today; cursor/keyset migration is required before long-session GA. |
| `ai_agent_interaction` | `SQL_LIST_AGENT_INTERACTIONS` + `SQL_COUNT_AGENT_INTERACTIONS` | Offset-paginated; App/Backend HTTP surfaces expose `agents.interactions.*`. |
| `ai_agent_task` | `SQL_LIST_AGENT_TASKS` + `SQL_COUNT_AGENT_TASKS` | Offset-paginated; Open/App/Backend HTTP surfaces expose `agents.tasks.*`. |

HTTP handlers pass `page` / `page_size` into repository `PaginationParams` and return `PageInfo.totalItems` from `COUNT(*)` queries.
Marketplace scope (`scope=market|public|published`) maps to `visibility = public` in SQL, not client-side filtering.

Active `4.0.0` project/session/chat list surfaces use stable database ordering,
bounded `LIMIT`/`OFFSET`, and paired count/search queries where the public
page/pageSize API needs totals:

| Target surface | Required database ordering |
| --- | --- |
| Project list | `(tenant_id, organization_id, owner/member scope, status, updated_at DESC, id DESC)` |
| Project sessions | `(tenant_id, organization_id, project_id, status, updated_at DESC, id DESC)` |
| Personal sessions | `(tenant_id, organization_id, owner_user_id, status, updated_at DESC, id DESC)` plus per-user pinned state |
| Session messages | `(tenant_id, organization_id, session_id, sequence ASC, id ASC)` |
| Message feedback analytics | `(tenant_id, organization_id, rating, created_at DESC, id DESC)` |
| Project members | `(tenant_id, organization_id, project_id, status, updated_at DESC, id DESC)` |

All page parameters are bounded and tenant/organization scoped. Search pushes
scope, filters, ranking, ordering, and limits into PostgreSQL or an outbox-fed
search authority. Frontends do not load all messages/projects/files and filter
them in memory. Drive file results remain queried through Drive APIs. A cursor
protocol requires a separately versioned OpenAPI and generated SDK contract;
repositories already retain deterministic tie-breaker ordering.
