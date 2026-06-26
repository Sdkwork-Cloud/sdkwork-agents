import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("agents database manifest declares postgres and sqlite engines", () => {
  const manifest = JSON.parse(
    readFileSync(path.join(repoRoot, "database/database.manifest.json"), "utf8"),
  );
  assert.equal(manifest.moduleId, "agents");
  assert.deepEqual(manifest.engines, ["postgres", "sqlite"]);
  assert.equal(manifest.tablePrefix, "agents_");
});

test("agents database contract is materialized without placeholders", () => {
  const schemaPath = path.join(repoRoot, "database/contract/schema.yaml");
  const schema = readFileSync(schemaPath, "utf8");
  assert.doesNotMatch(schema, /<module-id>/);
  assert.match(schema, /agents_/);
});
