# PostgreSQL Migrations

The directory is intentionally empty while `sdkwork-agents` is pre-launch.

The complete `6.0.0` schema is installed from
`database/ddl/baseline/postgres/0001_agents_baseline.sql`. There is no released
schema to upgrade, so retaining a historical compatibility migration here
would make every fresh baseline appear to have a pending migration and would
violate the SDKWork pre-launch migration rules. Add ordered forward migrations
only after the first release, without rewriting the baseline.
