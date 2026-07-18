import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { createAgentsEnvironment } from "../src/bootstrap/environment.ts";

test("H5 keeps the Agents application API and Appbase IAM gateway separate in development", () => {
  const environment = createAgentsEnvironment({
    VITE_SDKWORK_AGENTS_H5_ENVIRONMENT: "development",
  });

  assert.equal(environment.apiBaseUrl, "http://127.0.0.1:8095/app/v3/api");
  assert.equal(environment.appbaseAppApiBaseUrl, "http://127.0.0.1:3900");
  assert.equal(environment.appbaseLoginUrl, "http://127.0.0.1:3900");
  assert.equal(environment.lifecycleEnvironment, "development");
});

test("H5 requires an explicit IAM gateway outside development", () => {
  for (const lifecycleEnvironment of ["test", "staging", "production"] as const) {
    assert.throws(
      () => createAgentsEnvironment({
        VITE_SDKWORK_AGENTS_H5_ENVIRONMENT: lifecycleEnvironment,
      }),
      /APPBASE_APP_API_BASE_URL or VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL is required/,
    );
  }
});

test("H5 resolves the platform IAM gateway and lets an Appbase override take precedence", () => {
  const platformGatewayEnvironment = createAgentsEnvironment({
    VITE_SDKWORK_AGENTS_H5_ENVIRONMENT: "test",
    VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL: "https://iam.example.test/app/v3/api",
  });
  const overriddenEnvironment = createAgentsEnvironment({
    VITE_SDKWORK_AGENTS_H5_APPBASE_APP_API_BASE_URL: "https://iam-override.example.test",
    VITE_SDKWORK_AGENTS_H5_ENVIRONMENT: "test",
    VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL: "https://iam.example.test",
  });

  assert.equal(platformGatewayEnvironment.appbaseAppApiBaseUrl, "https://iam.example.test");
  assert.equal(platformGatewayEnvironment.appbaseLoginUrl, "https://iam.example.test");
  assert.equal(overriddenEnvironment.appbaseAppApiBaseUrl, "https://iam-override.example.test");
});

test("H5 normalizes an IAM app-api URL to a gateway root before OAuth calls", () => {
  const environment = createAgentsEnvironment({
    VITE_SDKWORK_AGENTS_H5_APPBASE_APP_API_BASE_URL: "https://iam.example.test/app/v3/api/",
    VITE_SDKWORK_AGENTS_H5_ENVIRONMENT: "staging",
  });
  const oauthUrl = `${environment.appbaseAppApiBaseUrl}/app/v3/api/oauth/provider-id`;

  assert.equal(environment.appbaseAppApiBaseUrl, "https://iam.example.test");
  assert.equal(oauthUrl, "https://iam.example.test/app/v3/api/oauth/provider-id");
  assert.equal((oauthUrl.match(/\/app\/v3\/api/g) ?? []).length, 1);
  assert.throws(
    () => createAgentsEnvironment({
      VITE_SDKWORK_AGENTS_H5_APPBASE_APP_API_BASE_URL:
        "https://iam.example.test/app/v3/api/app/v3/api",
      VITE_SDKWORK_AGENTS_H5_ENVIRONMENT: "staging",
    }),
    /must not include \/app\/v3\/api more than once/,
  );
});

test("the client materializer keeps Appbase independent from the Agents app-api origin", () => {
  const materializer = readFileSync(
    new URL("../../../scripts/materialize-client-app-surfaces.mjs", import.meta.url),
    "utf8",
  );

  assert.match(
    materializer,
    /defaultAppbaseGatewayHttpUrl = ['"]http:\/\/127\.0\.0\.1:3900['"]/,
  );
  assert.match(
    materializer,
    /VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL=/,
  );
  assert.match(
    materializer,
    /envPrefix \+ '_APPBASE_APP_API_BASE_URL='/,
  );
  assert.match(
    materializer,
    /function resolveAppbaseGatewayBaseUrl/,
  );
  assert.doesNotMatch(
    materializer,
    /\$\{envPrefix\}_APPBASE_APP_API_BASE_URL=\$\{defaultPublicHttpUrl\}\/app\/v3\/api/,
  );
  assert.doesNotMatch(materializer, /environments:\s*\{/);
});
