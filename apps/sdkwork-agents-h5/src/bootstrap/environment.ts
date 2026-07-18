export type AgentsLifecycleEnvironment = "development" | "test" | "staging" | "production";
export type AgentsRuntimeEnvironment = Readonly<Record<string, unknown>>;

export interface AgentsEnvironment {
  apiBaseUrl: string;
  appbaseAppApiBaseUrl: string;
  appbaseLoginUrl: string;
  backendApiBaseUrl: string;
  lifecycleEnvironment: AgentsLifecycleEnvironment;
}

const APP_API_SUFFIX = "/app/v3/api";
const DEVELOPMENT_APPLICATION_PUBLIC_HTTP_URL = "http://127.0.0.1:8095";
const DEVELOPMENT_APPBASE_GATEWAY_HTTP_URL = DEVELOPMENT_APPLICATION_PUBLIC_HTTP_URL;
const APPBASE_APP_API_BASE_URL_ENV = "VITE_SDKWORK_AGENTS_H5_APPBASE_APP_API_BASE_URL";
const PLATFORM_API_GATEWAY_HTTP_URL_ENV = "VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL";
const LIFECYCLE_ENVIRONMENTS = new Set<AgentsLifecycleEnvironment>([
  "development",
  "test",
  "staging",
  "production",
]);

function readEnv(environment: AgentsRuntimeEnvironment, key: string): string | undefined {
  const value = environment[key];
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function deriveAppApiBaseUrl(applicationPublicHttpUrl: string): string {
  return `${applicationPublicHttpUrl.replace(/\/+$/u, "")}${APP_API_SUFFIX}`;
}

function deriveBackendApiBaseUrl(applicationPublicHttpUrl: string): string {
  return `${applicationPublicHttpUrl.replace(/\/+$/u, "")}/backend/v3/api`;
}

function resolveLifecycleEnvironment(
  environment: AgentsRuntimeEnvironment,
): AgentsLifecycleEnvironment {
  const configuredEnvironment = readEnv(environment, "VITE_SDKWORK_AGENTS_H5_ENVIRONMENT")
    ?? readEnv(environment, "MODE");
  if (configuredEnvironment) {
    const normalized = configuredEnvironment.toLowerCase();
    if (LIFECYCLE_ENVIRONMENTS.has(normalized as AgentsLifecycleEnvironment)) {
      return normalized as AgentsLifecycleEnvironment;
    }
    throw new Error(
      "VITE_SDKWORK_AGENTS_H5_ENVIRONMENT must be development, test, staging, or production.",
    );
  }

  return environment.PROD === true || environment.PROD === "true"
    ? "production"
    : "development";
}

export function normalizeAppbaseGatewayBaseUrl(value: string): string {
  const normalized = value.trim().replace(/\/+$/u, "");
  if (!normalized) {
    throw new Error("Appbase IAM gateway URL must not be empty.");
  }

  let parsed: URL;
  try {
    parsed = new URL(normalized);
  } catch {
    throw new Error("Appbase IAM gateway URL must be an absolute HTTP(S) URL.");
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("Appbase IAM gateway URL must use HTTP or HTTPS.");
  }
  if (parsed.search || parsed.hash) {
    throw new Error("Appbase IAM gateway URL must not include a query string or fragment.");
  }

  const duplicatedAppApiSuffix = `${APP_API_SUFFIX}${APP_API_SUFFIX}`;
  if (normalized.endsWith(duplicatedAppApiSuffix)) {
    throw new Error(`Appbase IAM gateway URL must not include ${APP_API_SUFFIX} more than once.`);
  }

  return normalized.endsWith(APP_API_SUFFIX)
    ? normalized.slice(0, -APP_API_SUFFIX.length)
    : normalized;
}

function resolveAppbaseGatewayBaseUrl(
  environment: AgentsRuntimeEnvironment,
  lifecycleEnvironment: AgentsLifecycleEnvironment,
): string {
  const configuredBaseUrl = readEnv(
    environment,
    APPBASE_APP_API_BASE_URL_ENV,
  ) ?? readEnv(environment, PLATFORM_API_GATEWAY_HTTP_URL_ENV);
  if (configuredBaseUrl) {
    return normalizeAppbaseGatewayBaseUrl(configuredBaseUrl);
  }
  if (lifecycleEnvironment === "development") {
    return DEVELOPMENT_APPBASE_GATEWAY_HTTP_URL;
  }

  throw new Error(
    `${APPBASE_APP_API_BASE_URL_ENV} or ${PLATFORM_API_GATEWAY_HTTP_URL_ENV} is required for ${lifecycleEnvironment}.`,
  );
}

function resolveDefaultApplicationPublicHttpUrl(): string {
  return typeof window === "undefined"
    ? DEVELOPMENT_APPLICATION_PUBLIC_HTTP_URL
    : window.location.origin;
}

export function createAgentsEnvironment(
  environment: AgentsRuntimeEnvironment,
): AgentsEnvironment {
  const lifecycleEnvironment = resolveLifecycleEnvironment(environment);
  const applicationPublicHttpUrl = readEnv(
    environment,
    "VITE_SDKWORK_AGENTS_H5_APPLICATION_PUBLIC_HTTP_URL",
  ) ?? resolveDefaultApplicationPublicHttpUrl();
  const appbaseAppApiBaseUrl = resolveAppbaseGatewayBaseUrl(environment, lifecycleEnvironment);

  return {
    apiBaseUrl: readEnv(environment, "VITE_SDKWORK_AGENTS_H5_APP_API_BASE_URL")
      ?? deriveAppApiBaseUrl(applicationPublicHttpUrl),
    appbaseAppApiBaseUrl,
    appbaseLoginUrl: readEnv(environment, "VITE_SDKWORK_AGENTS_H5_APPBASE_LOGIN_URL")
      ?? appbaseAppApiBaseUrl,
    backendApiBaseUrl: readEnv(environment, "VITE_SDKWORK_AGENTS_H5_BACKEND_API_BASE_URL")
      ?? deriveBackendApiBaseUrl(applicationPublicHttpUrl),
    lifecycleEnvironment,
  };
}

export function resolveEnvironment(): AgentsEnvironment {
  return createAgentsEnvironment(import.meta.env as AgentsRuntimeEnvironment);
}
