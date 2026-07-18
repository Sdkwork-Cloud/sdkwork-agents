import {
  createClient,
  type SdkworkAppClient as GeneratedSdkworkAgentsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/agents-app-sdk";

import {
  readAppSdkSessionTokens,
  resolveAppSdkAccessToken,
  resolveAppSdkAuthToken,
  type SdkworkAgentsMpSession,
} from "../session/session";

export type SdkworkAgentsAppClient = GeneratedSdkworkAgentsAppClient;
export type SdkworkAgentsAppClientConfig = SdkworkAppConfig;

let agentsAppSdkClient: SdkworkAgentsAppClient | null = null;
let configuredBaseUrl: string | null = null;
let bootstrapAccessToken: string | null = null;

const DEFAULT_MP_APP_API_BASE_URL = "http://127.0.0.1:8095/app/v3/api";

export function configureAgentsAppSdkBaseUrl(baseUrl: string): void {
  const normalized = baseUrl.trim().replace(/\/+$/u, "");
  configuredBaseUrl = normalized.length > 0 ? normalized : DEFAULT_MP_APP_API_BASE_URL;
}

export function configureAgentsAppSdkBootstrapAccessToken(accessToken?: string): void {
  const normalized = accessToken?.trim();
  bootstrapAccessToken = normalized && normalized.length > 0 ? normalized : null;
}

export function resolveAgentsAppSdkBaseUrl(): string {
  if (configuredBaseUrl) {
    return configuredBaseUrl;
  }
  return DEFAULT_MP_APP_API_BASE_URL;
}

export function createAgentsAppSdkClientConfig(
  session?: SdkworkAgentsMpSession | null,
): SdkworkAgentsAppClientConfig {
  const currentSession = session ?? readAppSdkSessionTokens();
  return {
    baseUrl: resolveAgentsAppSdkBaseUrl(),
    accessToken: resolveAppSdkAccessToken(currentSession) ?? bootstrapAccessToken ?? undefined,
    authToken: resolveAppSdkAuthToken(currentSession),
    platform: "mini-program",
  };
}

export function initAgentsAppSdkClient(
  config: SdkworkAgentsAppClientConfig = createAgentsAppSdkClientConfig(),
): SdkworkAgentsAppClient {
  agentsAppSdkClient = createClient(config);
  return agentsAppSdkClient;
}

export function getAgentsAppSdkClient(): SdkworkAgentsAppClient {
  return agentsAppSdkClient ?? initAgentsAppSdkClient();
}

export function resetAgentsAppSdkClient(): void {
  agentsAppSdkClient = null;
  bootstrapAccessToken = null;
}
