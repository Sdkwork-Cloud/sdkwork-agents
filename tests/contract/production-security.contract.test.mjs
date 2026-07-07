import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readText(relativePath) {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

test("runtime environment helpers fail closed for production-like profiles", () => {
  const source = readText("crates/sdkwork-agents-contract/src/lib.rs");
  const expectedProfiles = ["production", "prod", "staging", "stage", "live", "test"];
  const identifiersMatch = source.match(
    /const PRODUCTION_LIKE_ENV_IDENTIFIERS:[\s\S]*?&\[(?<items>[\s\S]*?)\];/,
  );

  assert.ok(
    identifiersMatch?.groups?.items,
    "sdkwork-agents-contract must declare PRODUCTION_LIKE_ENV_IDENTIFIERS",
  );

  const productionLikeProfiles = Array.from(
    identifiersMatch.groups.items.matchAll(/"([^"]+)"/g),
    (match) => match[1],
  );

  for (const profile of expectedProfiles) {
    assert.ok(
      productionLikeProfiles.includes(profile),
      `production-like profile '${profile}' must be gated`,
    );
  }

  assert.match(
    source,
    /pub fn agents_allow_contract_runtime_fallback\(\) -> bool\s*\{\s*!agents_is_production_like_environment\(\)\s*\}/,
    "contract runtime fallback must be disabled in production-like environments",
  );
  assert.match(
    source,
    /pub fn agents_use_dev_inline_auth_resolver\(\) -> bool\s*\{\s*!agents_is_production_like_environment\(\) && agents_dev_auth_bypass_enabled\(\)\s*\}/,
    "dev inline auth resolver must require both non-production and explicit bypass",
  );
  assert.match(
    source,
    /pub fn ensure_dev_auth_bypass_allowed\(\) -> Result<\(\), String>\s*\{[\s\S]*agents_dev_auth_bypass_enabled\(\) && agents_is_production_like_environment\(\)[\s\S]*return Err/,
    "dev auth bypass must fail closed when enabled in a production-like environment",
  );

  const deploymentEnvIndex = source.indexOf('std::env::var("SDKWORK_DEPLOYMENT_ENV")');
  const genericEnvIndex = source.indexOf('std::env::var("ENVIRONMENT")');
  assert.ok(
    deploymentEnvIndex >= 0 && genericEnvIndex > deploymentEnvIndex,
    "SDKWORK_DEPLOYMENT_ENV must be the first security gate environment authority",
  );
});

test("production HTTP bootstrap uses IAM, Postgres, and runtime facade completion", () => {
  const source = readText(
    "crates/sdkwork-agents-kernel-bridge/src/agent_http_state.rs",
  );

  assert.match(
    source,
    /ensure_dev_auth_bypass_allowed\(\)[\s\S]*if agents_use_dev_inline_auth_resolver\(\)/,
    "HTTP state bootstrap must reject unsafe dev auth before considering the dev branch",
  );
  assert.match(
    source,
    /if agents_use_dev_inline_auth_resolver\(\)\s*\{[\s\S]*return dev_agent_http_state\(\);[\s\S]*\}\s*production_postgres_agent_http_state\(\)/,
    "production bootstrap must fall through to Postgres state when dev inline auth is unavailable",
  );
  assert.match(
    source,
    /AllowAllPolicyProvider::try_allow\("policy\.agents\.dev"\)[\s\S]*context\("build agents dev-only policy provider"\)\?/,
    "dev HTTP bootstrap must use the fallible dev-only policy constructor",
  );
  assert.match(
    source,
    /fn production_postgres_agent_http_state\(\) -> Result<AgentHttpState>\s*\{[\s\S]*PostgresAgentRepository::new[\s\S]*PostgresAgentAuditSink::new_global[\s\S]*IamGatedPolicyProvider::default\(\)[\s\S]*RuntimeFacadeChatCompleter/,
    "production state must use Postgres repository, Postgres audit, IAM policy, and RuntimeFacadeChatCompleter",
  );
  assert.match(
    source,
    /AllowAllPolicyProvider` is only used for development scenarios/,
    "AllowAllPolicyProvider must remain documented as development-only",
  );
});

test("standalone gateway shutdown path does not panic on signal installation failure", () => {
  const source = readText(
    "crates/sdkwork-agents-standalone-gateway/src/shutdown.rs",
  );

  assert.doesNotMatch(
    source,
    /\.expect\(|panic!\(/,
    "shutdown signal handling must log installation failures instead of panicking",
  );
  assert.match(
    source,
    /tracing::warn!|tracing::error!/,
    "shutdown signal handling must emit an operator-visible log when signal setup fails",
  );
});

test("in-memory repository and audit locks recover from poisoned guards", () => {
  const source = readText(
    "crates/sdkwork-intelligence-agents-service/src/infrastructure.rs",
  );

  assert.doesNotMatch(
    source,
    /expect\("in-memory repository rwlock poisoned"\)|expect\("in-memory audit sink mutex poisoned"\)|expect\("agents metrics scrape state poisoned"\)/,
    "in-memory repository, audit, and metrics locks must recover and log instead of panicking",
  );
  assert.match(
    source,
    /trait RecoveringRwLock[\s\S]*fn recovering_read[\s\S]*fn recovering_write/,
    "in-memory repository lock recovery must stay centralized",
  );
  assert.match(
    source,
    /trait RecoveringMutex[\s\S]*fn recovering_lock/,
    "in-memory audit and metrics lock recovery must stay centralized",
  );
});

test("managed-store constructors propagate snowflake initialization errors", () => {
  const infrastructure = readText(
    "crates/sdkwork-intelligence-agents-service/src/infrastructure.rs",
  );
  const persistence = readText(
    "crates/sdkwork-intelligence-agents-service/src/persistence.rs",
  );
  const bridge = readText(
    "crates/sdkwork-agents-kernel-bridge/src/agent_http_state.rs",
  );

  assert.doesNotMatch(
    `${infrastructure}\n${persistence}`,
    /AgentBusinessIdGenerator::new_default\(\)\s*\.expect\(/,
    "managed-store constructors must propagate ID generator initialization errors instead of panicking",
  );
  assert.doesNotMatch(
    infrastructure,
    new RegExp(
      [
        "SECURITY " + "VIOLATION",
        "panic to " + "prevent " + "security",
        "will " + "panic to " + "prevent",
      ].join("|"),
    ),
    "production security validation must return explicit errors or fail closed instead of panicking",
  );
  assert.match(
    infrastructure,
    /pub fn validate_production_security_config\(\) -> Result<\(\), String>/,
    "production security validation must be fallible",
  );
  assert.match(
    infrastructure,
    /pub fn try_allow\(provider_id: impl Into<String>\) -> Result<Self, String>/,
    "AllowAllPolicyProvider must expose a fallible dev-only constructor",
  );
  assert.match(
    infrastructure,
    /pub fn try_new\(\) -> KernelResult<Self>/,
    "in-memory repository must expose a fallible constructor for runtime bootstrap",
  );
  assert.match(
    bridge,
    /InMemoryAgentRepository::try_new\(\)[\s\S]*context\("build agents dev in-memory repository"\)\?/,
    "dev HTTP bootstrap must use the fallible in-memory repository constructor",
  );
  assert.match(
    persistence,
    /pub fn from_pool\(pool: BlockingPostgresPool\) -> KernelResult<Self>/,
    "postgres adapter from_pool must remain fallible when it builds the default ID generator",
  );
});

test("agent repository port does not compile incomplete persistence adapters", () => {
  const source = readText("crates/sdkwork-intelligence-agents-service/src/ports.rs");

  assert.doesNotMatch(
    source,
    /default stubs|backward compatibility with adapters|CapabilityMissing \{/,
    "AgentRepository must require production persistence methods instead of default stubs",
  );

  for (const methodName of [
    "insert_provider_binding",
    "update_provider_binding",
    "get_provider_binding",
    "insert_composition_slot",
    "update_composition_slot",
    "get_composition_slot",
    "insert_session",
    "update_session",
    "get_session",
    "list_sessions",
    "count_sessions",
    "insert_message",
    "update_message",
    "get_message",
    "list_messages",
    "count_messages",
    "next_message_sequence",
    "insert_interaction",
    "update_interaction",
    "get_interaction",
    "list_interactions",
    "count_interactions",
    "insert_task",
    "update_task",
    "get_task",
    "list_tasks",
    "count_tasks",
  ]) {
    assert.match(
      source,
      new RegExp(`fn ${methodName}\\([\\s\\S]*?;`),
      `AgentRepository.${methodName} must be a required trait method`,
    );
  }
});

test("operator docs expose production security smoke criteria", () => {
  const smokeRunbook = readText("docs/runbooks/smoke-test.md");
  const prd = readText("docs/product/prd/PRD.md");
  const techArchitecture = readText("docs/architecture/tech/TECH_ARCHITECTURE.md");

  assert.match(
    smokeRunbook,
    /SDKWORK_AGENTS_DEV_AUTH_BYPASS=true` must be `false` in staging\/production/,
    "smoke runbook must require dev auth bypass to be false in staging and production",
  );
  assert.match(
    smokeRunbook,
    /runtimeMode`[\s\S]{0,80}contract stub/,
    "smoke runbook must reject contract stub runtime mode for chat",
  );
  assert.match(
    prd,
    /Production gates[\s\S]*check:production-security/,
    "PRD production gates must include the production-security check",
  );
  assert.match(
    techArchitecture,
    /check:production-security/,
    "technical architecture verification must include the production-security check",
  );
});
