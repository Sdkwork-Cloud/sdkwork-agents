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

export function configureAgentsAppSdkBaseUrl(baseUrl: string): void {
  const normalized = baseUrl.trim().replace(/\/+$/u, "");
  if (normalized.length === 0) {
    throw new Error("SDKWORK_AGENTS_APP_API_BASE_URL is required");
  }
  configuredBaseUrl = normalized;
}

export function configureAgentsAppSdkBootstrapAccessToken(accessToken?: string): void {
  const normalized = accessToken?.trim();
  bootstrapAccessToken = normalized && normalized.length > 0 ? normalized : null;
}

export function resolveAgentsAppSdkBaseUrl(): string {
  if (configuredBaseUrl) {
    return configuredBaseUrl;
  }
  throw new Error("SDKWORK_AGENTS_APP_API_BASE_URL must be configured before SDK bootstrap");
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
