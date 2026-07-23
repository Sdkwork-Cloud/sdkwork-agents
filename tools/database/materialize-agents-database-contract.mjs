#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const defaultRoot = path.resolve(scriptDirectory, "../..");
const root = parseRoot(process.argv.slice(2));
const standardMaterializer = path.resolve(
  defaultRoot,
  "../sdkwork-specs/tools/materialize-database-contract-from-baseline.mjs",
);

if (!existsSync(standardMaterializer)) {
  throw new Error(`standard database materializer not found: ${standardMaterializer}`);
}

const databaseRoot = path.join(root, "database");
const contractRoot = path.join(databaseRoot, "contract");
const paths = {
  manifest: path.join(databaseRoot, "database.manifest.json"),
  prefixRegistry: path.join(contractRoot, "prefix-registry.json"),
  schema: path.join(contractRoot, "schema.yaml"),
  tableRegistry: path.join(contractRoot, "table-registry.json"),
};

const authored = {
  manifest: readJson(paths.manifest),
  prefixRegistry: readJson(paths.prefixRegistry),
  schema: parseSchema(paths.schema),
  tableRegistry: readJson(paths.tableRegistry),
};

validateAuthoredContract(authored);

const tempBase = path.resolve(os.tmpdir());
const stagingRoot = mkdtempSync(path.join(tempBase, "sdkwork-agents-db-contract-"));

try {
  cpSync(databaseRoot, path.join(stagingRoot, "database"), { recursive: true });
  const materialized = runStandardMaterializer(stagingRoot);
  validateMaterializedDiscovery(authored, materialized);

  const tableByName = new Map(
    authored.tableRegistry.tables.map((entry) => [entry.table_name, entry]),
  );
  const prefixByName = new Map(
    authored.prefixRegistry.prefixes.map((entry) => [entry.prefix, entry]),
  );
  const tableNames = materialized.tableRegistry.tables.map((entry) => entry.table_name);
  const prefixes = materialized.prefixRegistry.prefixes.map((entry) => entry.prefix);

  writeJsonIfChanged(paths.tableRegistry, {
    ...authored.tableRegistry,
    tables: tableNames.map((tableName) => tableByName.get(tableName)),
  });
  writeJsonIfChanged(paths.prefixRegistry, {
    ...authored.prefixRegistry,
    prefixes: prefixes.map((prefix) => prefixByName.get(prefix)),
  });
  writeJsonIfChanged(paths.manifest, {
    ...authored.manifest,
    lifecycle: {
      ...authored.manifest.lifecycle,
      autoMigrate: materialized.manifest.lifecycle.autoMigrate,
    },
  });

  process.stdout.write(
    `materialized ${tableNames.length} Agents tables without replacing authored semantic metadata\n`,
  );
} finally {
  if (path.dirname(stagingRoot) !== tempBase) {
    throw new Error(`refusing to remove unexpected staging directory: ${stagingRoot}`);
  }
  rmSync(stagingRoot, { recursive: true, force: true });
}

function parseRoot(argv) {
  let resolvedRoot = defaultRoot;
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === "--root") {
      const value = argv[index + 1];
      if (!value) {
        throw new Error("--root requires a directory");
      }
      resolvedRoot = path.resolve(value);
      index += 1;
    } else {
      throw new Error(`unknown argument: ${token}`);
    }
  }
  return resolvedRoot;
}

function runStandardMaterializer(targetRoot) {
  const result = spawnSync(
    process.execPath,
    [
      standardMaterializer,
      "--root",
      targetRoot,
      "--baseline",
      "database/ddl/baseline/postgres/0001_agents_baseline.sql",
      "--module-id",
      "agents",
      "--owner",
      "agents-platform",
      "--prefixes",
      "ai_",
      "--engines",
      "postgres",
    ],
    { encoding: "utf8" },
  );
  if (result.status !== 0) {
    throw new Error(
      `standard database materializer failed\n${result.stdout}\n${result.stderr}`,
    );
  }

  const stagedContractRoot = path.join(targetRoot, "database", "contract");
  return {
    manifest: readJson(path.join(targetRoot, "database", "database.manifest.json")),
    prefixRegistry: readJson(path.join(stagedContractRoot, "prefix-registry.json")),
    schema: parseSchema(path.join(stagedContractRoot, "schema.yaml")),
    tableRegistry: readJson(path.join(stagedContractRoot, "table-registry.json")),
  };
}

function validateAuthoredContract(contract) {
  assertEqual(contract.manifest.moduleId, "agents", "manifest moduleId");
  assertEqual(contract.schema.module_id, "agents", "schema module_id");
  assertEqual(
    contract.tableRegistry.moduleId,
    contract.manifest.moduleId,
    "table registry moduleId",
  );
  assertEqual(
    contract.tableRegistry.contractVersion,
    contract.manifest.contractVersion,
    "table registry contractVersion",
  );
  assertEqual(
    contract.schema.contract_version,
    contract.manifest.contractVersion,
    "schema contract_version",
  );

  for (const entry of contract.tableRegistry.tables) {
    for (const field of [
      "profile",
      "owner",
      "write_owner",
      "system_of_record",
      "compliance_level",
      "lifecycle_status",
    ]) {
      if (entry[field] === undefined || entry[field] === "") {
        throw new Error(`${entry.table_name} is missing authored registry field ${field}`);
      }
    }
  }

  for (const entry of contract.prefixRegistry.prefixes) {
    for (const field of ["owner", "domain", "capability", "description"]) {
      if (entry[field] === undefined || entry[field] === "") {
        throw new Error(`${entry.prefix} is missing authored prefix field ${field}`);
      }
    }
  }
}

function validateMaterializedDiscovery(authoredContract, materializedContract) {
  const authoredTables = authoredContract.tableRegistry.tables.map(
    (entry) => entry.table_name,
  );
  const schemaTables = authoredContract.schema.tables.map((entry) => entry.name);
  const discoveredTables = materializedContract.tableRegistry.tables.map(
    (entry) => entry.table_name,
  );
  const stagedSchemaTables = materializedContract.schema.tables.map((entry) => entry.name);

  assertSameSet(authoredTables, schemaTables, "authored registry and schema tables");
  assertSameSet(authoredTables, discoveredTables, "authored and baseline tables");
  assertSameSet(authoredTables, stagedSchemaTables, "authored and materialized schema tables");

  const authoredPrefixes = authoredContract.prefixRegistry.prefixes.map(
    (entry) => entry.prefix,
  );
  const discoveredPrefixes = materializedContract.prefixRegistry.prefixes.map(
    (entry) => entry.prefix,
  );
  assertSameSet(authoredPrefixes, discoveredPrefixes, "authored and baseline prefixes");

  assertEqual(
    materializedContract.manifest.contractVersion,
    authoredContract.manifest.contractVersion,
    "materialized manifest contractVersion",
  );
}

function assertSameSet(left, right, label) {
  const normalizedLeft = [...new Set(left)].sort();
  const normalizedRight = [...new Set(right)].sort();
  if (JSON.stringify(normalizedLeft) !== JSON.stringify(normalizedRight)) {
    throw new Error(
      `${label} differ: authored=${normalizedLeft.join(",")} discovered=${normalizedRight.join(",")}`,
    );
  }
}

function assertEqual(actual, expected, label) {
  if (actual !== expected) {
    throw new Error(`${label} must be ${expected}, received ${actual}`);
  }
}

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function parseSchema(filePath) {
  const value = parseYaml(readFileSync(filePath, "utf8"));
  if (!value || !Array.isArray(value.tables)) {
    throw new Error(`${filePath} must contain a tables array`);
  }
  return value;
}

function writeJsonIfChanged(filePath, value) {
  const current = readJson(filePath);
  if (JSON.stringify(current) === JSON.stringify(value)) {
    return;
  }
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}
