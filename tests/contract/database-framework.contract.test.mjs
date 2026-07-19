import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { readdirSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("agents database manifest declares postgres-only production engine", () => {
  const manifest = JSON.parse(
    readFileSync(path.join(repoRoot, "database/database.manifest.json"), "utf8"),
  );
  assert.equal(manifest.moduleId, "agents");
  assert.deepEqual(manifest.engines, ["postgres"]);
  assert.equal(manifest.tablePrefix, "ai_");
  assert.equal(manifest.contractVersion, "4.0.0");
});

test("agents database contract is materialized without placeholders", () => {
  const schemaPath = path.join(repoRoot, "database/contract/schema.yaml");
  const schema = readFileSync(schemaPath, "utf8");
  assert.doesNotMatch(schema, /<module-id>/);
  assert.match(schema, /ai_/);
  assert.match(schema, /contract_version: 4\.0\.0/u);
  assert.equal((schema.match(/lifecycle_status: expanding/gu) ?? []).length, 0);
  assert.equal((schema.match(/lifecycle_status: active/gu) ?? []).length, 17);

  const registry = JSON.parse(
    readFileSync(path.join(repoRoot, "database/contract/table-registry.json"), "utf8"),
  );
  assert.equal(registry.tables.length, 17);
  assert.ok(
    registry.tables.every((entry) => entry.lifecycle_status === "active"),
    "every Agents 4.0 table must be active in the contract registry",
  );
});

test("sqlite managed-store baseline is a real eight-table schema", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/sqlite/0001_agents_baseline.sql"),
    "utf8",
  );
  const tables = Array.from(
    baseline.matchAll(/CREATE TABLE IF NOT EXISTS (ai_agent(?:_[a-z_]+)?)/giu),
    (match) => match[1],
  );

  assert.deepEqual(
    [...new Set(tables)].sort(),
    [
      "ai_agent",
      "ai_agent_audit_event",
      "ai_agent_composition_slot",
      "ai_agent_interaction",
      "ai_agent_message",
      "ai_agent_runtime_binding",
      "ai_agent_session",
      "ai_agent_task",
    ].sort(),
  );
  assert.match(baseline, /PRAGMA foreign_keys = ON/u);
  assert.doesNotMatch(baseline, /\bJSONB\b|\bTIMESTAMPTZ\b|\bCREATE EXTENSION\b|\bGIN\b/iu);
  assert.match(
    baseline,
    /FOREIGN KEY \(tenant_id, agent_id\) REFERENCES ai_agent/iu,
  );
  assert.match(
    baseline,
    /UNIQUE INDEX IF NOT EXISTS uk_ai_agent_runtime_binding_active_default/iu,
  );
});

test("greenfield baseline structures are not replayed by incremental migrations", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const migrations = readdirSync(migrationRoot)
    .filter((fileName) => fileName.endsWith(".up.sql"))
    .map((fileName) => readFileSync(path.join(migrationRoot, fileName), "utf8"))
    .join("\n");

  for (const constraint of Array.from(
    baseline.matchAll(/CONSTRAINT\s+([a-z0-9_]+)/giu),
    (match) => match[1],
  )) {
    assert.doesNotMatch(
      migrations,
      new RegExp(`ADD\\s+CONSTRAINT\\s+${constraint}\\b`, "iu"),
      `incremental migrations must not replay baseline constraint ${constraint}`,
    );
  }
});

test("commercial chat expand migrations are paired and own the approved target tables", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const expandName = "0002_chat_project_commercial_expand";
  const up = readFileSync(path.join(migrationRoot, `${expandName}.up.sql`), "utf8");
  const down = readFileSync(path.join(migrationRoot, `${expandName}.down.sql`), "utf8");

  for (const table of [
    "ai_agent_project",
    "ai_agent_project_composition_slot",
    "ai_agent_chat_turn",
    "ai_agent_message_drive_ref",
    "ai_agent_message_feedback",
    "ai_agent_resource_user_state",
    "ai_agent_project_member",
    "ai_agent_share_link",
    "ai_agent_outbox_event",
  ]) {
    assert.match(up, new RegExp(`CREATE\\s+TABLE\\s+${table}\\b`, "iu"));
    assert.match(down, new RegExp(`DROP\\s+TABLE\\s+${table}\\b`, "iu"));
  }

  assert.match(up, /last_message_sequence\s+BIGINT\s+NOT NULL/iu);
  assert.match(up, /DEFERRABLE INITIALLY DEFERRED/iu);
  assert.match(
    up,
    /CREATE\s+UNIQUE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+uk_ai_agent_tenant_id/iu,
    "expand migration must repair the baseline tenant-scoped internal key",
  );
  assert.match(
    up,
    /DROP\s+CONSTRAINT\s+IF\s+EXISTS\s+fk_ai_agent_audit_event_agent/iu,
    "expand migration must tolerate baselines created before the audit agent FK existed",
  );
  assert.match(down, /rollback refused/iu);
  assert.doesNotMatch(up, /\b(?:CREATE|ALTER|REFERENCES|INSERT|UPDATE|DELETE)\b[^;]*\bim_/iu);
});

test("every PostgreSQL up migration has a reviewed down pair", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const files = readdirSync(migrationRoot);
  for (const upName of files.filter((fileName) => fileName.endsWith(".up.sql"))) {
    const downName = upName.replace(/\.up\.sql$/u, ".down.sql");
    assert.ok(files.includes(downName), `${upName} must have ${downName}`);
  }
});

test("outbox dedupe is corrected through an immutable follow-up migration", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const correction = readFileSync(
    path.join(migrationRoot, "0003_scope_agents_outbox_dedupe.up.sql"),
    "utf8",
  );
  assert.match(
    correction,
    /UNIQUE\s*\(tenant_id,\s*organization_id,\s*dedupe_key\)/iu,
  );
});

test("audit action constraint accepts every action emitted by the runtime", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const correction = readFileSync(
    path.join(migrationRoot, "0004_audit_action_runtime_compatibility.up.sql"),
    "utf8",
  );
  const domain = readFileSync(
    path.join(
      repoRoot,
      "crates/sdkwork-intelligence-agents-service/src/domain.rs",
    ),
    "utf8",
  );
  const actionCodes = Array.from(
    domain
      .match(/pub fn action_code[\s\S]*?\n    \}\n\}/u)?.[0]
      ?.matchAll(/=>\s*"([a-z_]+)"/gu) ?? [],
    (match) => match[1],
  );

  assert.ok(actionCodes.length > 0, "runtime audit action codes must be discoverable");
  for (const actionCode of actionCodes) {
    assert.match(
      correction,
      new RegExp(`'${actionCode}'`, "u"),
      `audit constraint must accept runtime action ${actionCode}`,
    );
  }
});

test("audit storage supports project aggregates without fabricated agent foreign keys", () => {
  const migration = readFileSync(
    path.join(
      repoRoot,
      "database/migrations/postgres/0005_generalize_agents_audit_aggregate.up.sql",
    ),
    "utf8",
  );
  const rollback = readFileSync(
    path.join(
      repoRoot,
      "database/migrations/postgres/0005_generalize_agents_audit_aggregate.down.sql",
    ),
    "utf8",
  );
  assert.match(migration, /aggregate_type\s+VARCHAR\(64\)/iu);
  assert.match(migration, /agent_internal_id\s+DROP NOT NULL/iu);
  assert.match(migration, /agent_id\s+DROP NOT NULL/iu);
  assert.match(migration, /aggregate_type\s+<>\s+'agent'/iu);
  assert.match(rollback, /rollback refused/iu);
});

const databaseContractDocumentationFiles = [
  "database/README.md",
  "database/SCHEMA_DESIGN.md",
  "database/MIGRATION_SUMMARY.md",
];

const forbiddenDatabaseContractPhrases = [
  /\bpost-GA\b/iu,
  /\bunimplemented\b/iu,
  /\bdeferred until\b/iu,
  /\buntil\b[\s\S]{0,80}\bimplemented\b/iu,
  /\bTBD\b/iu,
];

test("database contract docs describe current scope without deferred debt wording", () => {
  for (const relativePath of databaseContractDocumentationFiles) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    for (const pattern of forbiddenDatabaseContractPhrases) {
      assert.doesNotMatch(
        content,
        pattern,
        `${relativePath} must describe current database scope and entry criteria instead of deferred or unimplemented work`,
      );
    }
  }
});
