import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const serviceSrc = path.join(repoRoot, "crates/sdkwork-intelligence-agents-service/src");

function read(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function serviceFile(relativePath) {
  return path.join(serviceSrc, relativePath);
}

function readJson(relativePath) {
  return JSON.parse(read(relativePath));
}

const routeComponentCrates = [
  "sdkwork-routes-agents-open-api",
  "sdkwork-routes-agents-app-api",
  "sdkwork-routes-agents-backend-api",
  "sdkwork-routes-agents-http-shared",
];

test("agents Rust HTTP adapter separates request context, middleware, and test helpers", () => {
  const rootHttp = read("crates/sdkwork-intelligence-agents-service/src/http.rs");
  const contextPath = serviceFile("http/context.rs");
  const middlewarePath = serviceFile("http/middleware.rs");
  const testingPath = serviceFile("http/testing.rs");

  assert.ok(existsSync(contextPath), "HTTP request context must live in src/http/context.rs");
  assert.ok(existsSync(middlewarePath), "HTTP middleware must live in src/http/middleware.rs");
  assert.ok(existsSync(testingPath), "HTTP test helpers must live in src/http/testing.rs");

  assert.match(rootHttp, /mod context;/, "http.rs must assemble context as a focused module");
  assert.match(rootHttp, /mod middleware;/, "http.rs must assemble middleware as a focused module");
  assert.match(rootHttp, /pub mod testing;/, "http.rs must expose test helpers as a focused module");

  assert.doesNotMatch(rootHttp, /pub struct AgentRequestContext\b/);
  assert.doesNotMatch(rootHttp, /pub\(crate\) struct RequestScope\b/);
  assert.doesNotMatch(rootHttp, /async fn reject_client_scope_selectors\b/);
  assert.doesNotMatch(rootHttp, /pub mod testing\s*\{/);

  const context = read("crates/sdkwork-intelligence-agents-service/src/http/context.rs");
  const middleware = read("crates/sdkwork-intelligence-agents-service/src/http/middleware.rs");
  const testing = read("crates/sdkwork-intelligence-agents-service/src/http/testing.rs");

  assert.match(context, /pub struct AgentRequestContext\b/);
  assert.match(context, /pub\(crate\) struct RequestScope\b/);
  assert.match(context, /pub\(crate\) fn owner_scope\b/);
  assert.match(context, /pub\(crate\) fn tenant_id_u64\b/);
  assert.match(middleware, /async fn reject_client_scope_selectors\b/);
  assert.match(middleware, /const CLIENT_SCOPE_QUERY_KEYS\b/);
  assert.match(testing, /pub fn test_web_context\b/);
  assert.match(testing, /WebRequestContext\b/);
});

test("agents Postgres persistence keeps SQL constants in a focused module", () => {
  const rootPersistence = read("crates/sdkwork-intelligence-agents-service/src/persistence.rs");
  const sqlPath = serviceFile("persistence/sql.rs");

  assert.ok(existsSync(sqlPath), "Postgres SQL constants must live in src/persistence/sql.rs");
  assert.match(rootPersistence, /mod sql;/, "persistence.rs must assemble SQL constants as a focused module");
  assert.match(rootPersistence, /pub use sql::\{/, "persistence.rs must re-export existing SQL constants");
  assert.doesNotMatch(rootPersistence, /^pub const SQL_/m);

  const sql = read("crates/sdkwork-intelligence-agents-service/src/persistence/sql.rs");
  assert.match(sql, /^pub const SQL_SELECT_AGENT_BY_TENANT_AND_AGENT_ID:/m);
  assert.match(sql, /^pub const SQL_INSERT_AGENT_SESSION:/m);
  assert.match(sql, /^pub const SQL_INSERT_AGENT_TASK:/m);
});

test("agents route crate component specs own repository-local crate roots", () => {
  for (const crateName of routeComponentCrates) {
    const expectedRoot = `crates/${crateName}`;
    const spec = readJson(`${expectedRoot}/specs/component.spec.json`);
    const actualRoot = spec.component?.root;

    assert.equal(
      actualRoot,
      expectedRoot,
      `${crateName} component.root must be ${expectedRoot}, not a sibling workspace path`,
    );
    assert.ok(
      existsSync(path.join(repoRoot, actualRoot)),
      `${crateName} component.root must resolve inside sdkwork-agents`,
    );
    assert.ok(
      !actualRoot.startsWith("sdkwork-kernel/"),
      `${crateName} component.root must not point to sdkwork-kernel`,
    );
  }
});

test("root contract checks include Rust service module boundary gate", () => {
  const packageJson = JSON.parse(read("package.json"));
  const command = packageJson.scripts?.["check:contracts"] ?? "";

  assert.match(
    command,
    /tests\/contract\/rust-service-module-boundaries\.contract\.test\.mjs/,
    "pnpm check:contracts must run rust-service-module-boundaries.contract.test.mjs",
  );
});
