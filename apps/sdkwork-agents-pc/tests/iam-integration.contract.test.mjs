import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');

test('pc composes the canonical Appbase PC auth runtime', () => {
  const runtime = read('src/bootstrap/iamRuntime.ts');
  assert.match(runtime, /createSdkworkAppbasePcAuthRuntime/);
  assert.match(runtime, /prepareCredentialEntryTokens/);
  assert.match(runtime, /credentialEntry:\s*\{/);
  assert.match(runtime, /__SDKWORK_CREDENTIAL_ENTRY_BOOTSTRAP_ACCESS_TOKEN__/);
  assert.match(runtime, /getSdkworkChatGlobalTokenManager\(\)/);
  assert.match(runtime, /sessionBridge:\s*\{/);
  assert.match(runtime, /persistAppSdkSessionTokens/);
  assert.match(runtime, /readAppSdkSessionTokens/);
  assert.match(runtime, /clearAppSdkSessionTokens/);
  assert.doesNotMatch(runtime, /createTokenManager\(/);
  assert.doesNotMatch(runtime, /Authorization|Access-Token/);
});

test('pc protects product routes with canonical IAM routes', () => {
  const gate = read('src/AuthGate.tsx');
  assert.match(gate, /SdkworkIamAuthRoutes/);
  assert.match(gate, /hydrateAgentsPcIamSession/);
  assert.match(gate, /isSessionReady\(readAppSdkSessionTokens\(\)\)/);
  assert.match(gate, /SDKWORK_AGENTS_PC_SESSION_CHANGED_EVENT/);
  assert.match(gate, /appearance=\{AGENTS_AUTH_APPEARANCE\}/);
  assert.match(gate, /loginMethods: \['password'\]/);
  assert.match(gate, /emailCodeLoginEnabled: false/);
  assert.match(gate, /phoneCodeLoginEnabled: false/);
  assert.match(gate, /matchMedia\('\(max-width: 767px\)'\)/);
  assert.match(gate, /qrLoginEnabled: !compactAuthViewport/);
  assert.match(gate, /aria-live="polite"/);
  assert.doesNotMatch(gate, /姝|鎭|浼/);
  const routing = read('src/authRouting.ts');
  assert.match(routing, /AUTH_LOGIN_PATH = '\/auth\/login'/);
  assert.match(routing, /AUTH_LOGIN_PATH\}\?redirect=/);
  assert.match(routing, /context\.authLevel/);
  assert.match(routing, /context\.tenantId/);
  assert.match(routing, /context\.userId/);
  assert.doesNotMatch(gate, /SDKWORK_ACCESS_TOKEN/);
  assert.doesNotMatch(gate, /localStorage|sessionStorage/);
});

test('pc validates persisted sessions through IAM before revealing product routes', () => {
  const runtime = read('src/bootstrap/iamRuntime.ts');
  assert.match(runtime, /service\.auth\.sessions\.current\.retrieve\(\)/);
  assert.match(runtime, /catch \{\s*clearAppSdkSessionTokens\(\)/s);
});

test('pc auth appearance uses the public IAM theme surface', () => {
  const appearance = read('src/authAppearance.ts');
  const styles = read('src/index.css');
  assert.match(appearance, /createSdkworkAuthAppearancePreset\('midnight'\)/);
  assert.match(appearance, /mergeSdkworkAuthAppearanceConfigs/);
  assert.match(appearance, /theme:/);
  assert.match(styles, /@source "\.\.\/\.\.\/\.\.\/\.\.\/sdkwork-iam\/.*sdkwork-auth-pc-react\/src"/);
  assert.match(styles, /@source "\.\.\/\.\.\/\.\.\/\.\.\/sdkwork-ui\/sdkwork-ui-pc-react\/src"/);
  assert.match(styles, /prefers-reduced-motion/);
  assert.doesNotMatch(appearance, /querySelector|document\./);
});

test('pc shares IAM credentials across authenticated SDK clients', () => {
  const bootstrap = read('src/bootstrap/index.ts');
  assert.match(bootstrap, /sdkClients.*initAgentsAppSdkClient/s);
  assert.match(bootstrap, /sdkClients\.push\(knowledgebaseClient\)/);
  assert.match(bootstrap, /sdkClients\.push\(initSkillsAppSdkClient\(\)\)/);
  assert.match(bootstrap, /sdkClients\.push\(initVoiceAppSdkClient\(\)\)/);
  assert.match(bootstrap, /initializeAgentsPcIamRuntime\(sdkClients\)/);
});

test('pc declares independent IAM and Agent API base URLs', () => {
  const config = read('src/bootstrap/runtimeConfig.ts');
  const env = read('.env.example');
  const vite = read('vite.config.ts');

  assert.match(config, /VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL/);
  assert.match(config, /VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL/);
  assert.match(config, /VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL/);
  assert.match(config, /DEVELOPMENT_APPBASE_GATEWAY_HTTP_URL = 'http:\/\/127\.0\.0\.1:3900'/);
  assert.doesNotMatch(config, /appbaseAppApiBaseUrl:\s*agentsAppApiBaseUrl/);
  assert.match(env, /VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL/);
  assert.match(env, /VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL/);
  assert.match(env, /VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL="http:\/\/127\.0\.0\.1:3900"/);
  assert.match(vite, /'\/app\/v3\/api': 'http:\/\/127\.0\.0\.1:8095'/);
  assert.match(vite, /__SDKWORK_CREDENTIAL_ENTRY_BOOTSTRAP_ACCESS_TOKEN__/);
});
