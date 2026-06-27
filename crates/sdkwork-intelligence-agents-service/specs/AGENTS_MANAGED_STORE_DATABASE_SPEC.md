# SDKWork Agents Managed Store Database Specification (Deprecated)

> **Deprecated:** This document is retained only as a compatibility redirect.
> The canonical agents database contract is
> [`AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`](./AGENTS_AI_COMPOSITION_DATABASE_SPEC.md).

`sdkwork-agents` owns the **agent composition plane** (`ai_*` tables). Inline
knowledge-base and memory-store tables (`a_agent_knowledge_*`, `a_agent_memory_*`)
were removed; those domains belong to `sdkwork-knowledgebase` and `sdkwork-memory`.

Cross-module bindings use `ai_agent_composition_slot` and the HTTP surface
`agents.compositionSlots.*`.

See [`AGENTS_AI_COMPOSITION_DATABASE_SPEC.md`](./AGENTS_AI_COMPOSITION_DATABASE_SPEC.md)
for table definitions, migration notes, and audit actions.
