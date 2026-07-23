import {
  createClient,
  type SdkworkAppClient as SdkworkMemoryAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/memory-app-sdk";
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

export type SdkworkAgentsMemoryAppClient = SdkworkMemoryAppClient;
export type SdkworkAgentsMemoryAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";
let memoryAppSdkClient: SdkworkAgentsMemoryAppClient | null = null;
let memoryAppSdkClientProvider: (() => SdkworkAgentsMemoryAppClient) | null = null;

export function configureMemoryAppSdkClientProvider(
  provider: () => SdkworkAgentsMemoryAppClient,
): void {
  memoryAppSdkClientProvider = provider;
}

function transportBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  return normalized.endsWith(APP_API_SUFFIX)
    ? normalized.slice(0, -APP_API_SUFFIX.length)
    : normalized;
}

export function resolveMemoryAppSdkBaseUrl(): string {
  return readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_MEMORY_APP_API_BASE_URL")
    ?? resolveAgentsAppSdkBaseUrl();
}

export function createMemoryAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkAgentsMemoryAppClientConfig {
  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");
  return {
    baseUrl: transportBaseUrl(resolveMemoryAppSdkBaseUrl()),
    accessToken: resolveAppSdkAccessToken(currentSession) ?? envAccessToken,
    authToken: resolveAppSdkAuthToken(currentSession),
    interceptors: createSdkworkChatRequestContextInterceptors(
      () => readAppSdkSessionTokens() ?? currentSession,
    ),
    platform: "pc",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initMemoryAppSdkClient(
  config: SdkworkAgentsMemoryAppClientConfig = createMemoryAppSdkClientConfig(),
): SdkworkAgentsMemoryAppClient {
  memoryAppSdkClient = createClient(config);
  return memoryAppSdkClient;
}

export function getMemoryAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkAgentsMemoryAppClient {
  if (memoryAppSdkClientProvider) {
    return memoryAppSdkClientProvider();
  }
  return initMemoryAppSdkClient(createMemoryAppSdkClientConfig(session));
}

export function resetMemoryAppSdkClient(): void {
  memoryAppSdkClient = null;
  memoryAppSdkClientProvider = null;
}

export type { MemorySpace, MemorySpaceList } from "@sdkwork/memory-app-sdk";
