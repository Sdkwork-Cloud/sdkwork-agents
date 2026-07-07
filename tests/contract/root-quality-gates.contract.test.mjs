import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const requiredCheckScripts = [
  "check:api-operation-patterns",
  "check:pagination",
  "check:app-sdk-consumer-imports",
  "check:route-path-collisions",
  "check:apps-directory-index",
  "check:component-port-bindings",
  "check:frontend-composition",
  "check:permission-composition",
  "check:composition-resolver",
  "check:rust-backend-composition",
  "check:production-security",
];

test("root check command includes production readiness quality gates", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "package.json"), "utf8"),
  );
  const scripts = packageJson.scripts ?? {};
  const checkCommand = scripts.check ?? "";

  for (const script of requiredCheckScripts) {
    assert.ok(scripts[script], `package.json must expose ${script}`);
    assert.match(
      checkCommand,
      new RegExp(`pnpm (run )?${script.replaceAll(":", "\\:")}\\b`),
      `pnpm check must run ${script}`,
    );
  }
});

const requiredContractTestFiles = [
  "tests/contract/production-security.contract.test.mjs",
];

test("contract check command includes production security contracts", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "package.json"), "utf8"),
  );
  const scripts = packageJson.scripts ?? {};
  const checkContractsCommand = scripts["check:contracts"] ?? "";

  for (const relativePath of requiredContractTestFiles) {
    assert.match(
      checkContractsCommand,
      new RegExp(relativePath.replaceAll("/", "[\\\\/]")),
      `pnpm check:contracts must run ${relativePath}`,
    );
  }
});

test("root workflow scripts do not expose migration-only IM synchronization tools", () => {
  const packageJson = JSON.parse(
    readFileSync(path.join(repoRoot, "package.json"), "utf8"),
  );
  const scripts = packageJson.scripts ?? {};

  for (const [scriptName, command] of Object.entries(scripts)) {
    assert.doesNotMatch(
      `${scriptName} ${command}`,
      /\b(?:sdkwork-im|im-agent|im-pc-agent|migrate-im|retarget-im|sync-im|run-im)\b/,
      `${scriptName} must not expose migration-only sdkwork-im synchronization in the agents release surface`,
    );
  }

  const migrationOnlyScripts = readdirSync(path.join(repoRoot, "scripts")).filter((fileName) =>
    /(?:^|-)im(?:-|_).*agent|agent.*(?:^|-)im(?:-|_)/.test(fileName),
  );
  assert.deepEqual(
    migrationOnlyScripts,
    [],
    "migration-only sdkwork-im agent synchronization scripts must be removed from the agents release root",
  );
});

const launchCanonFiles = [
  "apps/README.md",
  "docs/product/prd/PRD.md",
  "docs/architecture/tech/TECH_ARCHITECTURE.md",
  "docs/architecture/tech/TECH-api-specification.md",
  "docs/guides/operator/README.md",
  "docs/runbooks/pre-launch-verification.md",
  "crates/sdkwork-intelligence-agents-service/specs/AGENTS_AI_COMPOSITION_DATABASE_SPEC.md",
  "crates/sdkwork-intelligence-agents-service/src/api.rs",
  "specs/README.md",
  "specs/AGENTS_KERNEL_BOUNDARY_SPEC.md",
  "specs/AGENTS_KERNEL_SPI_GAP_ANALYSIS.md",
  "specs/agents-birdcoder-alignment.spec.json",
  "scripts/client-surface-readiness-contract.test.mjs",
];

const forbiddenLaunchScopePatterns = [
  {
    pattern: /\bpost-GA\b/i,
    guidance: "use current GA scope or non-GA scope ownership instead of post-GA wording",
  },
  {
    pattern: /\bpost_GA\b/i,
    guidance: "use current GA scope or non-GA scope ownership instead of post_GA naming",
  },
  {
    pattern: /\bpost-launch\b/i,
    guidance: "use GA scope or non-GA scope ownership instead of post-launch wording",
  },
  {
    pattern: /\bpost_launch\b/i,
    guidance: "use GA scope or non-GA scope ownership instead of post_launch naming",
  },
  {
    pattern: /\bCommercial GA blocker\b/i,
    guidance: "express release exclusions as scoped ownership and entry criteria",
  },
  {
    pattern: /\bnot blockers?\b/i,
    guidance: "do not label active launch gaps as non-blockers",
  },
  {
    pattern: /\bpending-dart-sdk\b/i,
    guidance: "describe Flutter as out of GA scope until an owned Dart SDK is available",
  },
  {
    pattern: /\bH5 fallback\b/i,
    guidance: "describe mini-program WebView coverage as an explicit editor bridge",
  },
  {
    pattern: /\bupload deferred\b/i,
    guidance: "describe upload as out of current product scope unless Drive Uploader is wired",
  },
];

test("launch canon files use explicit GA scope instead of deferred debt wording", () => {
  for (const relativePath of launchCanonFiles) {
    const content = readFileSync(path.join(repoRoot, relativePath), "utf8");

    for (const { pattern, guidance } of forbiddenLaunchScopePatterns) {
      assert.doesNotMatch(
        content,
        pattern,
        `${relativePath} must ${guidance}`,
      );
    }
  }
});
