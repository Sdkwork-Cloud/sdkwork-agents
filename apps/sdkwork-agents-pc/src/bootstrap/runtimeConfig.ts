import manifest from '../../sdkwork.app.config.json';

export type AgentsPcEnvironment = 'dev' | 'prod' | 'test';
export type AgentsPcLifecycleEnvironment = 'development' | 'test' | 'staging' | 'production';
export type AgentsPcDeploymentMode = 'local' | 'saas';
export type AgentsPcRuntimeEnvironment = Readonly<Record<string, unknown>>;

export interface AgentsPcRuntimeConfig {
  agentsAppApiBaseUrl: string;
  appbaseAppApiBaseUrl: string;
  appId: string;
  deploymentMode: AgentsPcDeploymentMode;
  environment: AgentsPcEnvironment;
  lifecycleEnvironment: AgentsPcLifecycleEnvironment;
  locale: string;
}

const APP_API_SUFFIX = '/app/v3/api';
const DEVELOPMENT_PUBLIC_HTTP_URL = 'http://127.0.0.1:8095';
const DEVELOPMENT_APPBASE_GATEWAY_HTTP_URL = 'http://127.0.0.1:3900';
const APPBASE_APP_API_BASE_URL_ENV = 'VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL';
const PLATFORM_API_GATEWAY_HTTP_URL_ENV = 'VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL';
const LIFECYCLE_ENVIRONMENTS = new Set<AgentsPcLifecycleEnvironment>([
  'development',
  'test',
  'staging',
  'production',
]);

function readEnv(environment: AgentsPcRuntimeEnvironment, key: string): string | undefined {
  const value = environment[key];
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function deriveAppApiBaseUrl(publicHttpUrl: string): string {
  return `${publicHttpUrl.replace(/\/+$/u, '')}${APP_API_SUFFIX}`;
}

function resolveLifecycleEnvironment(
  environment: AgentsPcRuntimeEnvironment,
): AgentsPcLifecycleEnvironment {
  const configuredEnvironment = readEnv(environment, 'VITE_SDKWORK_AGENTS_PC_ENVIRONMENT')
    ?? readEnv(environment, 'MODE');
  if (configuredEnvironment) {
    const normalized = configuredEnvironment.toLowerCase();
    if (LIFECYCLE_ENVIRONMENTS.has(normalized as AgentsPcLifecycleEnvironment)) {
      return normalized as AgentsPcLifecycleEnvironment;
    }
    throw new Error(
      'VITE_SDKWORK_AGENTS_PC_ENVIRONMENT must be development, test, staging, or production.',
    );
  }

  return environment.PROD === true || environment.PROD === 'true'
    ? 'production'
    : 'development';
}

export function normalizeAppbaseGatewayBaseUrl(value: string): string {
  const normalized = value.trim().replace(/\/+$/u, '');
  if (!normalized) {
    throw new Error('Appbase IAM gateway URL must not be empty.');
  }

  let parsed: URL;
  try {
    parsed = new URL(normalized);
  } catch {
    throw new Error('Appbase IAM gateway URL must be an absolute HTTP(S) URL.');
  }
  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    throw new Error('Appbase IAM gateway URL must use HTTP or HTTPS.');
  }
  if (parsed.search || parsed.hash) {
    throw new Error('Appbase IAM gateway URL must not include a query string or fragment.');
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
  environment: AgentsPcRuntimeEnvironment,
  lifecycleEnvironment: AgentsPcLifecycleEnvironment,
): string {
  const configuredBaseUrl = readEnv(
    environment,
    APPBASE_APP_API_BASE_URL_ENV,
  ) ?? readEnv(environment, PLATFORM_API_GATEWAY_HTTP_URL_ENV);
  if (configuredBaseUrl) {
    return normalizeAppbaseGatewayBaseUrl(configuredBaseUrl);
  }
  if (lifecycleEnvironment === 'development') {
    return DEVELOPMENT_APPBASE_GATEWAY_HTTP_URL;
  }

  throw new Error(
    `${APPBASE_APP_API_BASE_URL_ENV} or ${PLATFORM_API_GATEWAY_HTTP_URL_ENV} is required for ${lifecycleEnvironment}.`,
  );
}

function resolveIamEnvironment(
  lifecycleEnvironment: AgentsPcLifecycleEnvironment,
): AgentsPcEnvironment {
  if (lifecycleEnvironment === 'development') {
    return 'dev';
  }
  if (lifecycleEnvironment === 'test') {
    return 'test';
  }
  return 'prod';
}

function resolveDefaultPublicHttpUrl(): string {
  return typeof window === 'undefined' ? DEVELOPMENT_PUBLIC_HTTP_URL : window.location.origin;
}

export function createAgentsPcRuntimeConfig(
  environment: AgentsPcRuntimeEnvironment,
): AgentsPcRuntimeConfig {
  const lifecycleEnvironment = resolveLifecycleEnvironment(environment);
  const publicHttpUrl = readEnv(environment, 'VITE_SDKWORK_AGENTS_PC_APPLICATION_PUBLIC_HTTP_URL')
    ?? resolveDefaultPublicHttpUrl();
  const agentsAppApiBaseUrl = readEnv(environment, 'VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL')
    ?? deriveAppApiBaseUrl(publicHttpUrl);

  return {
    agentsAppApiBaseUrl,
    appbaseAppApiBaseUrl: resolveAppbaseGatewayBaseUrl(environment, lifecycleEnvironment),
    appId: manifest.backend.appId,
    deploymentMode: lifecycleEnvironment === 'staging' || lifecycleEnvironment === 'production'
      ? 'saas'
      : 'local',
    environment: resolveIamEnvironment(lifecycleEnvironment),
    lifecycleEnvironment,
    locale: readEnv(environment, 'VITE_SDKWORK_AGENTS_PC_DEFAULT_LOCALE') ?? 'zh-CN',
  };
}

export function resolveAgentsPcRuntimeConfig(): AgentsPcRuntimeConfig {
  return createAgentsPcRuntimeConfig(import.meta.env as AgentsPcRuntimeEnvironment);
}
