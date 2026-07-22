# PostgreSQL Migrations

The pre-launch `5.0.0` schema is fully consolidated in the canonical baseline,
so this directory has no active migration files.

After the first production schema release, add sortable paired files named
`{version}_{name}.up.sql` and `{version}_{name}.down.sql` with the metadata,
expand/backfill/verify/contract, checksum, and rollback evidence required by
`DATABASE_FRAMEWORK_SPEC.md` and `MIGRATION_SPEC.md`.
