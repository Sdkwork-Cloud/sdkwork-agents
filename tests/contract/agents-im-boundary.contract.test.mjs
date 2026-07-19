import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const ignoredDirectories = new Set([
  ".git",
  ".runtime",
  "dist",
  "node_modules",
  "target",
]);

function collectFiles(relativeRoots, predicate) {
  const files = [];
  const pending = relativeRoots
    .map((relativeRoot) => path.join(repoRoot, relativeRoot))
    .filter((absoluteRoot) => existsSync(absoluteRoot));

  while (pending.length > 0) {
    const current = pending.pop();
    const currentStat = statSync(current);
    if (currentStat.isDirectory()) {
      if (ignoredDirectories.has(path.basename(current))) {
        continue;
      }
      for (const entry of readdirSync(current)) {
        pending.push(path.join(current, entry));
      }
      continue;
    }

    if (predicate(current)) {
      files.push(current);
    }
  }

  return files;
}

function relativePath(filePath) {
  return path.relative(repoRoot, filePath).replaceAll("\\", "/");
}

function findMatches(files, patterns) {
  const violations = [];
  for (const filePath of files) {
    const content = readFileSync(filePath, "utf8");
    for (const { label, pattern } of patterns) {
      if (pattern.test(content)) {
        violations.push(`${relativePath(filePath)}: ${label}`);
      }
    }
  }
  return violations;
}

const dependencyToken = ["sdkwork", "im"].join("-");
const dependencyTokenPattern = new RegExp(
  `(?:@sdkwork[\\/]im(?:[\\/-][a-z0-9-]+)?|${dependencyToken}(?:-[a-z0-9-]+)?|\\.\\.[\\\\/]${dependencyToken}(?:[\\\\/]|$))`,
  "iu",
);
const sourceDependencyPattern = new RegExp(
  `(?:\\bfrom\\s*|\\bimport\\s*\\(|\\brequire\\s*\\(|\\buse\\s+|\\bextern\\s+crate\\s+|\\bpath\\s*=\\s*)[\"']?(?:@sdkwork[\\/]im[a-z0-9_./\\-]*|sdkwork_im[a-z0-9_]*|\\.\\.[\\\\/]${dependencyToken}(?:[\\\\/][a-z0-9_./\\\\-]*)?)`,
  "iu",
);

test("Agents manifests and authored runtime sources do not depend on sdkwork-im", () => {
  const manifestFiles = collectFiles(
    ["Cargo.toml", "package.json", "pnpm-workspace.yaml", "apps", "crates", "sdks"],
    (filePath) =>
      path.basename(filePath) === "Cargo.toml" ||
      path.basename(filePath) === "package.json" ||
      path.basename(filePath) === "tsconfig.json" ||
      /(?:vite|webpack|rollup)\.config\.(?:js|mjs|cjs|ts)$/u.test(path.basename(filePath)),
  );
  const sourceFiles = collectFiles(
    ["apis", "apps", "crates", "scripts", "sdks", "src", "tools"],
    (filePath) => /\.(?:cjs|js|jsx|mjs|rs|ts|tsx)$/u.test(filePath),
  ).filter((filePath) => filePath !== fileURLToPath(import.meta.url));
  const manifestViolations = findMatches(
    [...new Set(manifestFiles)],
    [{ label: "forbidden sdkwork-im dependency/import/alias", pattern: dependencyTokenPattern }],
  );
  const sourceViolations = findMatches(sourceFiles, [
    { label: "forbidden sdkwork-im import/path", pattern: sourceDependencyPattern },
  ]);

  assert.deepEqual(
    [...manifestViolations, ...sourceViolations],
    [],
    [
      "The dependency direction is sdkwork-im -> sdkwork-agents -> sdkwork-kernel.",
      "Agents manifests and runtime sources must not import, alias, mount, or depend on sdkwork-im.",
    ].join(" "),
  );
});

test("Agents database contracts do not own IM tables or correlation identifiers", () => {
  const databaseFiles = collectFiles(
    ["database"],
    (filePath) => /\.(?:json|sql|yaml|yml)$/u.test(filePath),
  );
  const violations = findMatches(databaseFiles, [
    {
      label: "forbidden IM-owned table",
      pattern: /\bcreate\s+table\s+(?:if\s+not\s+exists\s+)?(?:[a-z0-9_]+\.)?["`]?im_[a-z0-9_]+["`]?/iu,
    },
    {
      label: "forbidden IM correlation identifier",
      pattern: /\bim_(?:channel|conversation|group|message|thread)_id\b/iu,
    },
    {
      label: "forbidden foreign key to an IM-owned table",
      pattern: /\breferences\s+(?:[a-z0-9_]+\.)?["`]?im_[a-z0-9_]+["`]?/iu,
    },
  ]);

  assert.deepEqual(
    violations,
    [],
    [
      "Agents owns execution sessions, turns, messages, projects, and audit state only.",
      "IM conversation/message correlation remains in sdkwork-im and must cross the boundary through public APIs or SDKs.",
    ].join(" "),
  );
});

test("the local dependency-boundary specification remains active and discoverable", () => {
  const boundarySpec = readFileSync(
    path.join(repoRoot, "specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md"),
    "utf8",
  );
  const componentSpec = JSON.parse(
    readFileSync(path.join(repoRoot, "specs/component.spec.json"), "utf8"),
  );

  assert.match(boundarySpec, /Status:\s*active architecture constraint/iu);
  assert.match(boundarySpec, /sdkwork-im\s*->\s*sdkwork-agents\s*->\s*sdkwork-kernel/iu);
  assert.ok(
    componentSpec.canonicalSpecs?.some(
      (entry) => entry.path === "specs/AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md",
    ),
    "component.spec.json must index AGENTS_IM_DEPENDENCY_BOUNDARY_SPEC.md",
  );
});
