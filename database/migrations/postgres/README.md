# PostgreSQL Migrations

`0001_add_agent_workspaces.up.sql` upgrades existing `5.0.0` installations to
the `6.0.0` Workspace-scoped Project contract. It is an idempotent, transactional
forward migration with an explicit default-Workspace backfill and verification
gate before constraints are enabled. Rollback is by compatible application
rollback or forward-fix; the migration does not delete historical data.
