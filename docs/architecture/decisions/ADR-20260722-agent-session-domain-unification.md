# ADR-20260722-agent-session-domain-unification

- Status: accepted
- Date: 2026-07-22
- Owners: agents-platform
- Requirement: Agents commercial session consolidation

## Context

Agents already owns hosted sessions, but its pre-launch contract used generic
`message` and `chat_turn` names and retained IM-like delivery states. BirdCoder
also maintained `ai_coding_session*` and assistant `chat_*` persistence. This
created duplicate systems of record and made agent execution easy to confuse
with instant messaging.

## Decision

Agents adopts one canonical aggregate:

```text
Project -> Session -> Turn -> Session Item -> Interaction
```

BirdCoder and other products use Agents sessions for coding and assistant
workflows. Agents adds typed runtime binding, fork lineage and checkpoint
support needed by those products. `sdkwork-im` remains the owner of IM
Conversation/Message and consumes Agents in the one-way direction
`sdkwork-im -> sdkwork-agents -> sdkwork-kernel`.

Because all affected applications are pre-launch, authored database, API and
SDK contracts move directly to canonical names without a compatibility facade,
dual write, projection or second baseline.

## Alternatives

- Keep BirdCoder coding sessions: rejected because it creates two session
  authorities and duplicates provider/runtime behavior.
- Move assistant sessions into IM: rejected because agent execution has no IM
  membership, delivery, read-cursor or presence semantics.
- Preserve message/chat aliases: rejected because the system is pre-launch and
  aliases would create permanent terminology debt.

## Consequences

- Agents owns all durable agent sessions, turns, items and checkpoints.
- Public APIs and generated SDKs use session/turn/item terminology.
- BirdCoder migration is a direct cutover into Agents, followed by deletion of
  its local session tables and APIs.
- Existing consumers must update before launch; there is no compatibility
  window or long-term dual write.

## Verification

```powershell
pnpm check:agents-im-boundary
pnpm db:validate
pnpm check:api-operation-patterns
pnpm check:agent-sdk-workspace
cargo test -p sdkwork-intelligence-agents-service
cargo test -p sdkwork-agents-runtime-facade
```

## Supersedes / Superseded By

Supersedes `ADR-20260719-commercial-chat-persistence.md`.

