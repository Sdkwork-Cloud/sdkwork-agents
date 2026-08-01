# REQ-2026-0801 Lossless Agent Interaction Envelope

- Owner: `agents-platform`
- Status: accepted
- Priority: P0
- Source: product parity goal
- Updated: `2026-08-01`

## Problem

Agents currently reduces every provider pause to an approval boolean or one
question answer. Codex app-server requests carry approval scope, policy
amendments, permission profiles, multiple keyed questions, automatic
resolution, MCP elicitation, option pickers, context selection, and setup
steps. The reduction prevents provider continuation and makes a product render
controls whose meaning cannot be reconstructed by the owner service.

## Requirements

1. `AgentInteraction` preserves one provider-neutral typed request envelope and
   one typed resolution without exposing provider wire request ids to clients.
2. Categories are `approval`, `user_input`, `elicitation`, and `setup`.
3. Request kinds cover command execution, file change, permission profile,
   question set, onboarding question set, option picker, context source picker,
   setup step, and MCP elicitation.
4. The request preserves bounded allowed actions and kind-specific typed data.
5. Question sets preserve stable ids, headers, prompts, other/secret flags,
   nullable options, answer arrays keyed by question id, and
   `autoResolutionMs`.
6. Approval requests preserve command, working-directory, file-change,
   proposed exec-policy amendment, and proposed network-policy amendment data;
   resolutions preserve scope, policy amendments, permission profiles, and
   strict auto-review.
7. MCP elicitation preserves form/OpenAI-form/URL mode, schema or URL,
   elicitation id, structured content, action, and metadata.
8. Existing generic approval and user-question clients remain compatible while
   typed clients migrate to the unified resolve operation.
9. PostgreSQL remains the durable authority. Typed request and resolution JSON
   are bounded, validated, and tenant/session/runtime-binding scoped.
10. Kernel owns provider wire compilation. BirdCoder uses only the generated
    Agents App SDK and never receives raw provider correlation fields.

## Acceptance Evidence

- Domain, DTO, persistence, PostgreSQL, OpenAPI, and generated SDK contracts
  round-trip every canonical Kernel request kind and resolution.
- Invalid category/kind/action combinations, unknown question ids, malformed
  amendments, invalid scopes, and oversized payloads fail closed.
- Existing approve/answer clients continue to pass compatibility tests.
- BirdCoder renders and submits every typed request through the generated App
  SDK, including multi-question and scoped approval cases.
- A real Codex app-server request pauses, persists, resolves, continues, and
  completes the same canonical Session Turn.

## Traceability

- [ADR-20260801](../../architecture/decisions/ADR-20260801-agent-interaction-envelope.md)
- [REVIEW-20260801](../../engineering/reviews/REVIEW-20260801-agent-interaction-envelope.md)
- [agent-interaction-envelope.contract.json](../../../specs/agent-interaction-envelope.contract.json)
