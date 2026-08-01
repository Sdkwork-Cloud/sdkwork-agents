# REQ-2026-0722 Canonical Agent Session Execution

- Owner: `agents-platform`
- Status: accepted
- Priority: P0
- Review: human-approved `2026-07-22`

## Requirement

Deliver tenant-isolated managed agent execution through one Agents-owned
Project, Session, Turn, SessionItem and Interaction model. Public consumers use
generated SDKs. Kernel provider mechanisms, IM communication semantics and
independent capability entities remain outside the Agents persistence boundary.

## Acceptance Criteria

- The 23-table PostgreSQL module is the only Agents business persistence
  authority and passes database framework validation.
- Session and Turn commands enforce trusted context, idempotency, payload hash,
  request time and bounded input.
- Turn completion persists ordered typed Session Items, usage, audit and outbox
  facts consistently.
- Interaction claim and resolution are race-safe and versioned.
- App, Backend and Open OpenAPI authorities expose 102, 58 and 56 operations
  respectively with standard envelopes, problem details and pagination.
- TypeScript and Flutter App SDKs expose Session, Turn, SessionItem, feedback and
  Interaction resources from package roots.
- Products store only stable Agents correlations and no alternate execution
  transcript.
- IM dependency direction remains `sdkwork-im -> sdkwork-agents -> sdkwork-kernel`.
- Skill packages and installations remain owned by `sdkwork-skills`; Agents
  stores only stable references.
- Release verification covers Rust, API, SDK, database, security,
  documentation, deployment and rollback.

## Traceability

- Domain: `specs/AGENTS_DOMAIN_SPEC.md`
- Session model: `specs/AGENTS_SESSION_MODEL_SPEC.md`
- Database:
  `crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`
- Boundary: `specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md`
- Decision:
  `docs/architecture/decisions/ADR-20260722-agent-session-domain-unification.md`
- API inventory: `docs/architecture/tech/TECH-api-specification.md`
- Verification: `docs/runbooks/pre-launch-verification.md`
