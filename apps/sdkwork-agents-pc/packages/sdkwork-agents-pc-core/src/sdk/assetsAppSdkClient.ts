import {
  createClient,
  type SdkworkAppClient as GeneratedAssetsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/assets-app-sdk";
import type { Interceptors } from "@sdkwork/sdk-common";

import {
  createSdkworkChatRequestContextInterceptors,
  getSdkworkChatGlobalTokenManager,
  readAppSdkSessionTokens,
  resolveAppSdkAccessToken,
  resolveAppSdkAuthToken,
  type SdkworkChatSession,
} from "../session/session";
import { resolveAgentsAppSdkBaseUrl } from "./agentsAppSdkClient";
import { readRuntimeEnv } from "./runtimeEnv";

export type { AssetItem, MediaResource } from "@sdkwork/assets-app-sdk";

export type SdkworkAgentsAssetsAppClient = GeneratedAssetsAppClient;
export type SdkworkAgentsAssetsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";

let assetsAppSdkClient: SdkworkAgentsAssetsAppClient | null = null;
let assetsAppSdkClientProvider: (() => SdkworkAgentsAssetsAppClient) | null = null;

export function configureAssetsAppSdkClientProvider(
  provider: () => SdkworkAgentsAssetsAppClient,
): void {
  assetsAppSdkClientProvider = provider;
  assetsAppSdkClient = null;
}

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function resolveAssetsAppSdkBaseUrl(): string | null {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_ASSETS_APP_API_BASE_URL");
  if (fromEnv) return fromEnv;
  return resolveAgentsAppSdkBaseUrl();
}

export function createAssetsAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkAgentsAssetsAppClientConfig {
  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");
  const baseUrl = resolveAssetsAppSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("Assets app SDK base URL is not configured");
  }

  return {
    baseUrl: normalizeGeneratedSdkBaseUrl(baseUrl),
    accessToken: resolveAppSdkAccessToken(currentSession) ?? envAccessToken,
    authToken: resolveAppSdkAuthToken(currentSession),
    interceptors: createSdkworkChatRequestContextInterceptors(
      () => readAppSdkSessionTokens() ?? currentSession,
    ),
    platform: "pc",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initAssetsAppSdkClient(
  config: SdkworkAgentsAssetsAppClientConfig = createAssetsAppSdkClientConfig(),
): SdkworkAgentsAssetsAppClient {
  assetsAppSdkClient = createClient(config);
  return assetsAppSdkClient;
}

export function getAssetsAppSdkClient(): SdkworkAgentsAssetsAppClient {
  if (assetsAppSdkClientProvider) {
    return assetsAppSdkClientProvider();
  }
  return assetsAppSdkClient ?? initAssetsAppSdkClient();
}

export function getAssetsAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkAgentsAssetsAppClient {
  if (assetsAppSdkClientProvider) {
    return assetsAppSdkClientProvider();
  }
  return initAssetsAppSdkClient(createAssetsAppSdkClientConfig(session));
}

export function resetAssetsAppSdkClient(): void {
  assetsAppSdkClient = null;
  assetsAppSdkClientProvider = null;
}
