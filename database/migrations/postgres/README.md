# PostgreSQL Migrations

Pre-launch the agents schema is consolidated on the single greenfield baseline:
`database/ddl/baseline/postgres/0001_agents_baseline.sql`. It contains the
complete `7.3.0` schema (canonical Agent, Session, Turn, Task Run/Attempt,
Turn input queue, provider Session directory, typed Interaction envelope,
session-activity lateral indexes, turn streaming-content checkpoint, and
simplified `agent.`/`binding.`/`provider.` identity namespaces).

No ordered post-baseline migrations exist while the app is pre-launch; the
lifecycle orchestrator applies the baseline once on an empty schema
(`baseline-plus-migrations`, `lifecycle.autoMigrate=false`). The drift gate
then verifies the live schema against `database/contract/`.

After first production release, add ordered expand/contract migrations without
rewriting the released baseline.
