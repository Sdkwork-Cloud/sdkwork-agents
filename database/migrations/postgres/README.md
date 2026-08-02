# PostgreSQL Migrations

The complete `7.3.0` schema is installed from
`database/ddl/baseline/postgres/0001_agents_baseline.sql` for new environments.
The forward-only `0001_complete_agents_7_0_0_schema.up.sql` migration brings
existing shared development schemas to the `7.0.0` contract. `0002` adds the
provider Session directory contract (`7.1.0`), `0003` adds typed Agent
Interaction request envelopes and categories (`7.2.0`), `0004` adds the
session-activity lateral lookup indexes (`7.3.0`), and `0005` adds the turn
streaming-content checkpoint column (`7.3.0`). These migrations do
not replay the baseline or delete dependency-owned data.

After first production release, add ordered expand/contract migrations without
rewriting the released baseline.
