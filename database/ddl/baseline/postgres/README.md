# PostgreSQL Baseline

`0001_agents_baseline.sql` is the complete `6.0.0` greenfield authority for the
Agents managed store. Apply it only to an empty installation through the
database lifecycle orchestrator. The application is pre-launch, so this
baseline is the complete initial state and no compatibility migration is
active. Future released-schema changes belong in paired files under
`database/migrations/postgres/`; they must not rewrite this baseline after the
first production release.
