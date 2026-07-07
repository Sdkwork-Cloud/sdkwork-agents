import assert from "node:assert/strict";
import { readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const authoredClientSourceRoots = [
  "apps/sdkwork-agents-pc/packages/sdkwork-agents-pc-agents/src",
  "apps/sdkwork-agents-h5/packages/sdkwork-agents-h5-agents/src",
];

const ignoredDirectoryNames = new Set([
  "dist",
  "generated",
  "node_modules",
  "target",
]);

function collectSourceFiles(relativeRoot) {
  const absoluteRoot = path.join(repoRoot, relativeRoot);
  const pending = [absoluteRoot];
  const files = [];

  while (pending.length > 0) {
    const current = pending.pop();
    const currentStat = statSync(current);

    if (currentStat.isDirectory()) {
      if (ignoredDirectoryNames.has(path.basename(current))) {
        continue;
      }
      for (const entry of readdirSync(current)) {
        pending.push(path.join(current, entry));
      }
      continue;
    }

    if (/\.(?:ts|tsx|js|jsx|mjs|cjs)$/u.test(current)) {
      files.push(current);
    }
  }

  return files;
}

test("frontend authored services do not call crypto.randomUUID directly", () => {
  const violations = [];

  for (const relativeRoot of authoredClientSourceRoots) {
    for (const filePath of collectSourceFiles(relativeRoot)) {
      const content = readFileSync(filePath, "utf8");
      if (/\bcrypto\.randomUUID\s*\(/u.test(content)) {
        violations.push(path.relative(repoRoot, filePath));
      }
    }
  }

  assert.deepEqual(
    violations,
    [],
    [
      "Frontend services must not call crypto.randomUUID directly.",
      "Use server-returned identifiers, generated SDK outputs, or an approved business-id helper instead.",
    ].join(" "),
  );
});
