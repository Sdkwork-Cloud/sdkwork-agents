# PostgreSQL Migrations

The complete `7.0.0` schema is installed from
`database/ddl/baseline/postgres/0001_agents_baseline.sql` for every supported
pre-launch environment. No compatibility migration is retained before the
first production release. Development databases created from earlier drafts
are rebuilt from the consolidated baseline.

After first production release, add ordered expand/contract migrations without
rewriting the released baseline.
