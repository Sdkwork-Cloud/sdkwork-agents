# REVIEW-20260801 Agent Interaction Envelope

- Status: approved
- Outcome: Go
- Date: `2026-08-01`
- Owner: `agents-platform`
- Approval: repository user authorized complete ChatGPT/Codex parity across sdkwork-birdcoder, sdkwork-agents, and sdkwork-kernel
- Requirement: [REQ-2026-0801](../../product/requirements/REQ-2026-0801-agent-interaction-envelope.md)
- Decision: [ADR-20260801](../../architecture/decisions/ADR-20260801-agent-interaction-envelope.md)

## Approved Contract

- Backward-compatible pre-release API and PostgreSQL expansion is approved.
- Provider-neutral typed request and resolution data belongs to Agents.
- Provider wire request ids, methods, and connection state remain Kernel-only.
- Existing generic approve/answer operations remain available only as legacy
  adapters; they may not collapse typed scoped decisions.
- Authored OpenAPI is the SDK generation authority. Generated output is never
  hand edited and BirdCoder continues to use the application-root App SDK.

## Prohibited Shortcuts

- BirdCoder-local provider Interaction DTOs.
- Raw HTTP or manual authorization headers from product UI code.
- Encoding structured requests inside option labels or prompt strings.
- Mapping Session-scoped approval, cancellation, amendments, permission
  profiles, or multi-question answers to boolean/string compatibility fields.
- Exposing provider request ids, protocol methods, provider Session ids, or
  callback payloads through public application records.

## Release Conditions

- PostgreSQL baseline and forward migration, domain, repository SQL, API
  authorities, generated SDKs, and all contract versions agree.
- Compatibility and typed conformance tests pass across Kernel, Agents, and
  BirdCoder.
- Real provider continuation evidence is recorded separately before product
  parity is declared complete.

