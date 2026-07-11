import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
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
