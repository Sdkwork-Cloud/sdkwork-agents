import {
  createClient,
  type SdkworkAppClient as GeneratedGenerationsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/generations-app-sdk";
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

export type SdkworkGenerationsAppClient = GeneratedGenerationsAppClient;
export type SdkworkGenerationsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";

let generationsAppSdkClient: SdkworkGenerationsAppClient | null = null;
let generationsAppSdkClientProvider: (() => SdkworkGenerationsAppClient) | null = null;

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function configureGenerationsAppSdkClientProvider(
  provider: () => SdkworkGenerationsAppClient,
): void {
  generationsAppSdkClientProvider = provider;
  generationsAppSdkClient = null;
}

export function resolveGenerationsAppSdkBaseUrl(): string | null {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_GENERATIONS_APP_API_BASE_URL")
    ?? readRuntimeEnv("VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL");
  if (fromEnv) return fromEnv;
  // Gateway-routed deployments (cloud profiles and local dev ingress) serve
  // every app API under the same origin as the Agents API. Reuse the Agents
  // base URL fallback chain (public HTTP URL -> window origin) so the
  // generations SDK works without its own explicit VITE_ override.
  return resolveAgentsAppSdkBaseUrl();
}

export function isGenerationsAppSdkConfigured(): boolean {
  return resolveGenerationsAppSdkBaseUrl() !== null;
}

export function createGenerationsAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkGenerationsAppClientConfig {
  const baseUrl = resolveGenerationsAppSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("generations app SDK base URL is not configured");
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

export function initGenerationsAppSdkClient(
  config: SdkworkGenerationsAppClientConfig = createGenerationsAppSdkClientConfig(),
): SdkworkGenerationsAppClient {
  generationsAppSdkClient = createClient(config);
  return generationsAppSdkClient;
}

export function getGenerationsAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkGenerationsAppClient {
  if (generationsAppSdkClientProvider) {
    return generationsAppSdkClientProvider();
  }
  return initGenerationsAppSdkClient(createGenerationsAppSdkClientConfig(session));
}

export function getGenerationsAppSdkClient(): SdkworkGenerationsAppClient {
  if (generationsAppSdkClientProvider) {
    return generationsAppSdkClientProvider();
  }
  return generationsAppSdkClient ?? initGenerationsAppSdkClient();
}

export function resetGenerationsAppSdkClient(): void {
  generationsAppSdkClient = null;
  generationsAppSdkClientProvider = null;
}

export type {
  CreateGenerationCommandRequest,
  GenerationCommandResponse,
  GenerationModality,
  GenerationRecord,
  GenerationRecordPage,
  GenerationResult,
  GenerationResultPage,
  GenerationStatus,
} from "@sdkwork/generations-app-sdk";
