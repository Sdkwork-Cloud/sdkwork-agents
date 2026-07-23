import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("agents database manifest declares one canonical PostgreSQL engine", () => {
  const manifest = JSON.parse(
    readFileSync(path.join(repoRoot, "database/database.manifest.json"), "utf8"),
  );
  assert.equal(manifest.moduleId, "agents");
  assert.deepEqual(manifest.engines, ["postgres"]);
  assert.equal(manifest.defaultEngine, "postgres");
  assert.equal(manifest.tablePrefix, "ai_");
  assert.equal(manifest.contractVersion, "5.0.0");
  assert.equal(manifest.paths.contract, "contract/schema.yaml");
  assert.equal(
    existsSync(
      path.join(repoRoot, "database/ddl/baseline/sqlite/0001_agents_baseline.sql"),
    ),
    false,
    "Agents must not retain a partial SQLite schema beside its PostgreSQL authority",
  );
  assert.equal(
    existsSync(path.join(repoRoot, "database/migrations/sqlite/README.md")),
    false,
    "Agents must not advertise an inactive SQLite migration path",
  );
});

test("agents database contract is materialized without placeholders", () => {
  const schemaPath = path.join(repoRoot, "database/contract/schema.yaml");
  const schema = readFileSync(schemaPath, "utf8");
  assert.doesNotMatch(schema, /<module-id>/);
  assert.match(schema, /table_prefix: ai_/u);
  assert.match(schema, /contract_version: 5\.0\.0/u);
  assert.match(
    schema,
    /ddl_authority: ddl\/baseline\/postgres\/0001_agents_baseline\.sql/u,
  );
  assert.equal((schema.match(/lifecycle_status: expanding/gu) ?? []).length, 0);
  assert.equal((schema.match(/lifecycle_status: active/gu) ?? []).length, 19);

  const registry = JSON.parse(
    readFileSync(path.join(repoRoot, "database/contract/table-registry.json"), "utf8"),
  );
  assert.equal(registry.contractVersion, "5.0.0");
  assert.equal(registry.tables.length, 19);
  assert.ok(
    registry.tables.every((entry) => entry.lifecycle_status === "active"),
    "every Agents 5.0 table must be active in the contract registry",
  );
});

test("PostgreSQL baseline exactly matches the 19-table contract registry", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  const baselineTables = Array.from(
    baseline.matchAll(/CREATE TABLE IF NOT EXISTS (ai_[a-z_]+)/giu),
    (match) => match[1],
  );
  const registry = JSON.parse(
    readFileSync(path.join(repoRoot, "database/contract/table-registry.json"), "utf8"),
  );
  const registryTables = registry.tables.map((entry) => entry.table_name);

  assert.deepEqual(
    [...new Set(baselineTables)].sort(),
    [...registryTables].sort(),
  );
  assert.doesNotMatch(
    baseline,
    /\bai_(?:coding_session|agent_message|agent_chat_turn|chat_conversation|chat_message)\b/iu,
  );
  assert.doesNotMatch(baseline, /\bim_[a-z_]+\b/iu);
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

test("pre-launch PostgreSQL contract has no incremental migration chain", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const activeMigrations = readdirSync(migrationRoot).filter((fileName) =>
    /\.(?:up|down)\.sql$/u.test(fileName),
  );
  assert.deepEqual(activeMigrations, []);
});

test("every PostgreSQL up migration has a reviewed down pair", () => {
  const migrationRoot = path.join(repoRoot, "database/migrations/postgres");
  const files = readdirSync(migrationRoot);
  for (const upName of files.filter((fileName) => fileName.endsWith(".up.sql"))) {
    const downName = upName.replace(/\.up\.sql$/u, ".down.sql");
    assert.ok(files.includes(downName), `${upName} must have ${downName}`);
  }
});

test("baseline outbox dedupe is tenant and organization scoped", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  assert.match(
    baseline,
    /CONSTRAINT\s+uk_ai_agent_outbox_event_dedupe\s+UNIQUE\s*\(\s*tenant_id,\s*organization_id,\s*dedupe_key\s*\)/iu,
  );
});

test("audit baseline accepts every action emitted by the runtime", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
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
      baseline,
      new RegExp(`'${actionCode}'`, "u"),
      `audit constraint must accept runtime action ${actionCode}`,
    );
  }
});

test("audit baseline supports non-agent aggregates without fabricated agent keys", () => {
  const baseline = readFileSync(
    path.join(repoRoot, "database/ddl/baseline/postgres/0001_agents_baseline.sql"),
    "utf8",
  );
  assert.match(baseline, /aggregate_type\s+VARCHAR\(64\)\s+NOT NULL/iu);
  assert.match(baseline, /agent_internal_id\s+BIGINT,/iu);
  assert.match(baseline, /agent_id\s+VARCHAR\(128\),/iu);
  assert.match(baseline, /aggregate_type\s+<>\s+'agent'/iu);
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
