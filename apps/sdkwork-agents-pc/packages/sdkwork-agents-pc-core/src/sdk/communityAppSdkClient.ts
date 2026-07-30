import {
  createClient,
  type SdkworkAppClient as GeneratedCommunityAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/community-app-sdk";
import type { Interceptors } from "@sdkwork/sdk-common";

import {
  createSdkworkChatRequestContextInterceptors,
  getSdkworkChatGlobalTokenManager,
  readAppSdkSessionTokens,
  resolveAppSdkAccessToken,
  resolveAppSdkAuthToken,
  type SdkworkChatSession,
} from "../session/session";
import { readRuntimeEnv } from "./runtimeEnv";

export type SdkworkCommunityAppClient = GeneratedCommunityAppClient;
export type SdkworkCommunityAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";

let communityAppSdkClient: SdkworkCommunityAppClient | null = null;
let communityAppSdkClientProvider: (() => SdkworkCommunityAppClient) | null = null;

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function configureCommunityAppSdkClientProvider(
  provider: () => SdkworkCommunityAppClient,
): void {
  communityAppSdkClientProvider = provider;
  communityAppSdkClient = null;
}

export function resolveCommunityAppSdkBaseUrl(): string | null {
  return readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_COMMUNITY_APP_API_BASE_URL")
    ?? readRuntimeEnv("VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL")
    ?? null;
}

export function isCommunityAppSdkConfigured(): boolean {
  return resolveCommunityAppSdkBaseUrl() !== null;
}

export function createCommunityAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkCommunityAppClientConfig {
  const baseUrl = resolveCommunityAppSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("community app SDK base URL is not configured");
  }

  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");
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

export function initCommunityAppSdkClient(
  config: SdkworkCommunityAppClientConfig = createCommunityAppSdkClientConfig(),
): SdkworkCommunityAppClient {
  communityAppSdkClient = createClient(config);
  return communityAppSdkClient;
}

export function getCommunityAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkCommunityAppClient {
  if (communityAppSdkClientProvider) {
    return communityAppSdkClientProvider();
  }
  return initCommunityAppSdkClient(createCommunityAppSdkClientConfig(session));
}

export function getCommunityAppSdkClient(): SdkworkCommunityAppClient {
  if (communityAppSdkClientProvider) {
    return communityAppSdkClientProvider();
  }
  return communityAppSdkClient ?? initCommunityAppSdkClient();
}

export function resetCommunityAppSdkClient(): void {
  communityAppSdkClient = null;
  communityAppSdkClientProvider = null;
}

export type { CommunityEntry, SdkWorkPageData } from "@sdkwork/community-app-sdk";
