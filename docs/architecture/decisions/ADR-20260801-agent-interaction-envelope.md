# ADR-20260801 Provider-Neutral Typed Agent Interactions

- Status: accepted
- Date: `2026-08-01`
- Owner: `agents-platform`
- Requirement: [REQ-2026-0801](../../product/requirements/REQ-2026-0801-agent-interaction-envelope.md)

## Context

Kernel already normalizes Codex user-mediated server requests and compiles
typed resolutions back to provider wire responses. Agents persists only a
prompt, flat options, and a reduced resolution, so the normalized contract is
lost before it reaches applications.

## Decision

Extend the existing `AgentInteraction` aggregate instead of creating a Codex
table or product-specific API.

- Keep the existing prompt/options fields as compatibility projections.
- Add a bounded nullable `request_json` object containing a provider-neutral
  versioned envelope.
- Expand the durable interaction category to approval, user input,
  elicitation, and setup.
- Store the canonical typed resolution in the existing bounded
  `resolution_json` object.
- Add one unified claim-fenced resolve operation. Existing approve/answer
  operations adapt only legacy generic interactions and do not accept lossy
  mappings for typed requests.
- Retain `provider_interaction_id` as the only application-visible opaque
  correlation reference. Provider request ids, wire types, protocol methods,
  provider Session ids, and tool callback identities remain server-side Kernel
  state.

The typed envelope contains category, request kind, allowed actions, and one
kind-specific data object. Agents validates the category/kind/action matrix and
bounded data before persistence and resolution. Kernel remains responsible for
provider-specific response compilation and at-most-once callback delivery.

## Alternatives

### Encode The Envelope In `options_json`

Rejected because the column is an option-array projection and using one string
as opaque JSON would evade type validation and generated SDK contracts.

### Expose Raw Codex Requests

Rejected because provider callback ids and protocol methods are adapter-private
security and lifecycle state.

### Add BirdCoder-Local Types

Rejected because Agents owns durable interactions and application SDKs own the
consumer boundary.

## Consequences

- The pre-release PostgreSQL contract gains one bounded JSONB column and two
  category codes.
- App/Open/Backend OpenAPI authorities and generated SDK families must be
  regenerated together from authored sources.
- Provider continuation still requires the separate persistent execution
  registry and cancellation work; this decision removes the payload-loss
  blocker but does not claim runtime continuation by itself.

