# REQ-2026-0719 Commercial Agents Chat

- Owner: agents-platform
- Status: approved
- Priority: P0
- Review: human-approved 2026-07-19

## Requirement

Deliver the Agents chat frontend as a durable, tenant-isolated commercial
product backed by Agents-owned projects, sessions, turns, messages, typed Drive
relations, feedback, per-user state, project collaboration, share grants, audit,
and reliable outbox publication.

The only IM integration direction is
`sdkwork-im -> sdkwork-agents -> sdkwork-kernel`. Agents must not depend on IM or
persist IM conversation/message correlation.

## Acceptance Criteria

- The active database contract reaches `4.0.0` through reviewed, paired,
  checksum-tracked migrations with rollback preflight.
- Project/session/message lists are database-paginated and tenant/organization
  isolated; message sequence allocation and command idempotency are atomic.
- App/backend/open API authorities, generated SDKs, service facades, and the PC
  Chat UI agree on stable project/session/message/turn contracts.
- UI product state no longer depends on mock arrays, singleton sessions, or
  browser-only pin/history persistence for authenticated users.
- Files use Drive Uploader and stable MediaResource references; search and
  generation lifecycle reuse their owning SDKWork modules.
- Security tests cover trusted actor context, cross-tenant denial, idempotency
  conflict, optimistic concurrency, share-token hashing, redaction, and retention.
- Migration, rollout, rollback, release, and commercial verification evidence is
  reproducible from root commands.

## Traceability

- Database: `crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`
- Boundary: `specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md`
- Decision: `docs/architecture/decisions/ADR-20260719-commercial-chat-persistence.md`
- Verification: `pnpm verify`, `pnpm db:validate`, SDK generation verification,
  Rust tests, frontend tests, and migration smoke tests.
