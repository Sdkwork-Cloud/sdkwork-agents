# ADR-20260719 Commercial Chat Persistence And IM Boundary

> Status: superseded by
> [ADR-20260722-agent-session-domain-unification.md](./ADR-20260722-agent-session-domain-unification.md).
> This record is historical decision evidence and is not current architecture
> authority.

- Status: accepted
- Date: 2026-07-19
- Owner: agents-platform

## Context

The PC Chat UI exposes project, session, message, attachment, feedback, sharing,
search, generation, archive, and pin workflows that the active eight-table
Agents contract cannot fully persist. IM also invokes Agents, but owns its own
conversation timeline and correlation.

## Decision

Adopt the 17-table Agents `4.0.0` target through an expand/contract rollout. Keep
existing session/message fields during compatibility, add dedicated project,
turn, Drive relation, feedback, user state, project member, share, and outbox
tables, then switch API/SDK/UI consumers before removing compatibility fields.

Keep the dependency direction
`sdkwork-im -> sdkwork-agents -> sdkwork-kernel`. IM stores opaque Agents ids in
IM-owned binding/dispatch rows. Neither module writes or foreign-keys the other
module's tables.

Reuse Prompts, Memory, Knowledgebase, Skills, MCP, Drive, Search, Generations,
LLM, IAM, and Membership through their public SDK/facade boundaries. Generated
SDK transport remains owner-only and is regenerated through `sdkgen`.

## Consequences

- Database rollout is additive first and requires compatibility reads/writes.
- Message and audit history cannot be cascade-deleted.
- Search/generation availability requires explicit dependency SDK runtime
  surfaces; no local substitute tables are allowed.
- IM and Agents can deploy independently; timeout recovery uses stable ids and
  idempotency rather than cross-database transactions.

## Verification

Migration smoke/rollback tests, repository/service transaction tests, OpenAPI
and SDK regeneration checks, frontend service/UI integration tests, IM dispatch
reconciliation tests, and release preflight are mandatory.
