# SQLite Development Subset

This directory contains a four-table agent control-plane subset for local
development. It intentionally excludes projects, sessions, turns, items,
interactions, checkpoints, tasks, sharing, and outbox state.

SQLite is not declared in `database/database.manifest.json#engines` and must not
be reported as managed-store parity with PostgreSQL. Business IDs are explicit
`BIGINT` values supplied by the same application ID provider; no SQLite rowid or
auto-increment allocation is permitted.
