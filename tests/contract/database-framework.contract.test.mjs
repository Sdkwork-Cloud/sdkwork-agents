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
});

test("agents database contract is materialized without placeholders", () => {
  const schemaPath = path.join(repoRoot, "database/contract/schema.yaml");
  const schema = readFileSync(schemaPath, "utf8");
  assert.doesNotMatch(schema, /<module-id>/);
  assert.match(schema, /ai_/);
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
