# PostgreSQL Baseline

`0001_agents_baseline.sql` is the complete `6.0.0` greenfield authority for the
Agents managed store. Apply it only to an empty installation through the
database lifecycle orchestrator. Future released-schema changes belong in
paired files under `database/migrations/postgres/`; they must not rewrite this
baseline after production release.
