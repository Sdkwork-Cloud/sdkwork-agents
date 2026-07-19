# Agents Chat 4.0 Migration Plan

## Sequence

1. Expand schema with paired `0002_chat_project_commercial_expand` migrations.
2. Deploy dual-compatible repositories and backfill normalized fields.
3. Add API contracts and regenerate owner-only SDK families.
4. Switch PC Chat and IM consumers to the new contracts.
5. Observe idempotency, sequence, outbox, retention, and error metrics.
6. Activate contract `4.0.0` only after all gates pass.
7. Remove legacy `role`/`artifacts_json` compatibility in a later reviewed
   contract migration.

## Rollback

Application rollback is allowed while compatibility columns remain. Database
down migration is allowed only when no target-only audit/action data or product
rows exist; its preflight fails closed rather than deleting commercial history.

## Compatibility Window

The expand schema supports the active `3.1.0` repository while new repositories
are deployed. The window closes only after generated SDK and frontend adoption
evidence is complete.
