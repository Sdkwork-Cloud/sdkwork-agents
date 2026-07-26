# Agents Database Schema

Status: active

Contract: `6.0.0`

Managed engine: PostgreSQL

## Ownership

All 20 tables are authored and written by `sdkwork-intelligence-agents-service`.
There are no imported tables, derived read stores, shadow copies, compatibility tables,
or cross-module foreign keys. External capabilities are represented only by
stable reference columns.

## Inventory

| Table | Responsibility |
| --- | --- |
| `ai_agent` | Managed agent identity and lifecycle |
| `ai_agent_runtime_binding` | Agent-level provider configuration |
| `ai_agent_composition_slot` | Agent references to sibling-owned capabilities |
| `ai_agent_audit_event` | Immutable sanitized aggregate audit facts |
| `ai_agent_project` | Reusable orchestration project and access policy |
| `ai_agent_project_composition_slot` | Project references to sibling-owned capabilities |
| `ai_agent_session` | Single durable execution session authority |
| `ai_agent_session_runtime_binding` | Session runtime selection and provider-native lineage |
| `ai_agent_turn` | Idempotent turn, retry, lease, fencing, usage, and terminal state |
| `ai_agent_session_item` | Ordered typed transcript or execution item |
| `ai_agent_item_drive_ref` | Typed relation to Drive-owned resources |
| `ai_agent_item_feedback` | Per-user assistant-output quality feedback |
| `ai_agent_interaction` | Approval or user-question pause point with claim fencing |
| `ai_agent_session_checkpoint` | Provider or Drive-backed resumable checkpoint reference |
| `ai_agent_task` | Product-managed scheduled task |
| `ai_agent_resource_user_state` | Per-user session/project view preferences |
| `ai_agent_project_member` | Project collaboration ACL |
| `ai_agent_share_link` | Hashed, revocable, expiring grant |
| `ai_agent_outbox_event` | Reliable aggregate event publication |

## Composition References

`ai_agent_composition_slot` and `ai_agent_project_composition_slot` share the
single mapping declared by `specs/AGENTS_DOMAIN_SPEC.md` section 3. PostgreSQL
enforces both the enum allow-lists and the exact pair. A document binding is
therefore `slot_kind=document`, `target_module=documents`; the row contains only
stable external references and bounded orchestration policy. Document content,
versions and lifecycle remain in `sdkwork-documents`.

## Session Aggregate

`ai_agent_session` stores only stable business scope, lineage, lifecycle, and
counters. Runtime/provider/model selection belongs to
`ai_agent_session_runtime_binding`. Ordered results belong to
`ai_agent_session_item`; artifacts are Drive references rather than copied
resource snapshots. Turns own idempotency and worker concurrency state.

Checkpoints store exactly one opaque provider checkpoint reference or one
Drive space/node pair. They never store plaintext resume tokens or unconstrained
provider state documents.

## Isolation And Concurrency

Every session-aggregate unique key and foreign key includes tenant and
organization scope. Application-allocated `BIGINT` primary keys are used.
Mutating aggregates use optimistic versions. Turn workers, interaction
claimers, and outbox publishers use bounded attempts, leases, opaque lease
tokens, expirations, and monotonic fencing tokens.

PostgreSQL list paths use tenant-leading indexes with stable `id` tie-breakers.
Retention scans use partial indexes. There are no IM delivery/read columns in
the execution aggregate; `last_read_item_sequence` is only a local Agents
user-view preference.
