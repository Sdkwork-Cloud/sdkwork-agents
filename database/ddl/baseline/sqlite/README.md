# SQLite Managed-Store Status

SDKWork Agents provides a native eight-table SQLite baseline at
`0001_agents_baseline.sql`. The baseline executes on SQLite with foreign keys enabled and avoids
PostgreSQL-only types, casts, extensions, and index methods. The service also provides a validated
SQLite pool facade behind the `sqlite-sync` feature.

SQLite is not yet an advertised managed-store engine because repository and audit adapters,
transaction coverage, lifecycle integration, server-side pagination, and parity tests against
PostgreSQL remain required. Do not add SQLite to `database.manifest.json#engines` before those
runtime artifacts and tests exist.
