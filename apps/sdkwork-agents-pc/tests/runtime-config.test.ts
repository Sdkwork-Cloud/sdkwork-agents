import assert from 'node:assert/strict';
import test from 'node:test';

import {
  createAgentsPcRuntimeConfig,
  normalizeAppbaseGatewayBaseUrl,
} from '../src/bootstrap/runtimeConfig.ts';

test('PC keeps the Agents application API and Appbase IAM gateway separate in development', () => {
  const config = createAgentsPcRuntimeConfig({
    VITE_SDKWORK_AGENTS_PC_ENVIRONMENT: 'development',
  });

  assert.equal(config.agentsAppApiBaseUrl, 'http://127.0.0.1:8095/app/v3/api');
  assert.equal(config.appbaseAppApiBaseUrl, 'http://127.0.0.1:3900');
  assert.equal(config.environment, 'dev');
  assert.equal(config.lifecycleEnvironment, 'development');
});

test('PC requires an explicit IAM gateway outside development', () => {
  for (const lifecycleEnvironment of ['test', 'staging', 'production'] as const) {
    assert.throws(
      () => createAgentsPcRuntimeConfig({
        VITE_SDKWORK_AGENTS_PC_ENVIRONMENT: lifecycleEnvironment,
      }),
      /APPBASE_APP_API_BASE_URL or VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL is required/,
    );
  }
});

test('PC resolves the platform IAM gateway and lets an Appbase override take precedence', () => {
  const platformGatewayConfig = createAgentsPcRuntimeConfig({
    VITE_SDKWORK_AGENTS_PC_ENVIRONMENT: 'test',
    VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL: 'https://iam.example.test/app/v3/api',
  });
  const overriddenConfig = createAgentsPcRuntimeConfig({
    VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL: 'https://iam-override.example.test',
    VITE_SDKWORK_AGENTS_PC_ENVIRONMENT: 'test',
    VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL: 'https://iam.example.test',
  });

  assert.equal(platformGatewayConfig.appbaseAppApiBaseUrl, 'https://iam.example.test');
  assert.equal(overriddenConfig.appbaseAppApiBaseUrl, 'https://iam-override.example.test');
});

test('PC normalizes an IAM app-api URL to a gateway root before OAuth calls', () => {
  const config = createAgentsPcRuntimeConfig({
    VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL: 'https://iam.example.test/app/v3/api/',
    VITE_SDKWORK_AGENTS_PC_ENVIRONMENT: 'staging',
  });
  const oauthUrl = `${config.appbaseAppApiBaseUrl}/app/v3/api/oauth/provider-id`;

  assert.equal(config.appbaseAppApiBaseUrl, 'https://iam.example.test');
  assert.equal(oauthUrl, 'https://iam.example.test/app/v3/api/oauth/provider-id');
  assert.equal((oauthUrl.match(/\/app\/v3\/api/g) ?? []).length, 1);
  assert.throws(
    () => normalizeAppbaseGatewayBaseUrl(
      'https://iam.example.test/app/v3/api/app/v3/api',
    ),
    /must not include \/app\/v3\/api more than once/,
  );
});
