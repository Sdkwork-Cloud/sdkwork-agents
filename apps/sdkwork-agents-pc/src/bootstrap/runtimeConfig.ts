export type AgentsPcEnvironment = 'dev' | 'prod' | 'test';
export type AgentsPcDeploymentMode = 'local' | 'saas';

export interface AgentsPcRuntimeConfig {
  agentsAppApiBaseUrl: string;
  appbaseAppApiBaseUrl: string;
  appId: string;
  deploymentMode: AgentsPcDeploymentMode;
  environment: AgentsPcEnvironment;
  locale: string;
}

const APP_API_SUFFIX = '/app/v3/api';
const DEVELOPMENT_PUBLIC_HTTP_URL = 'http://127.0.0.1:8095';

function readEnv(key: string): string | undefined {
  const value = import.meta.env[key];
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function deriveAppApiBaseUrl(publicHttpUrl: string): string {
  return `${publicHttpUrl.replace(/\/+$/u, '')}${APP_API_SUFFIX}`;
}

function resolveDefaultPublicHttpUrl(): string {
  return typeof window === 'undefined' ? DEVELOPMENT_PUBLIC_HTTP_URL : window.location.origin;
}

export function resolveAgentsPcRuntimeConfig(): AgentsPcRuntimeConfig {
  const publicHttpUrl = readEnv('VITE_SDKWORK_AGENTS_PC_APPLICATION_PUBLIC_HTTP_URL')
    ?? resolveDefaultPublicHttpUrl();
  const agentsAppApiBaseUrl = readEnv('VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL')
    ?? deriveAppApiBaseUrl(publicHttpUrl);

  return {
    agentsAppApiBaseUrl,
    appbaseAppApiBaseUrl: readEnv('VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL')
      ?? agentsAppApiBaseUrl,
    appId: manifest.backend.appId,
    deploymentMode: import.meta.env.PROD ? 'saas' : 'local',
    environment: import.meta.env.PROD ? 'prod' : 'dev',
    locale: readEnv('VITE_SDKWORK_AGENTS_PC_DEFAULT_LOCALE') ?? 'zh-CN',
  };
}
import manifest from '../../sdkwork.app.config.json';
