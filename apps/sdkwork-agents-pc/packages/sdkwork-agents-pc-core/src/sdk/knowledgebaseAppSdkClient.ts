import {
  createClient,
  type SdkworkKnowledgebaseAppClient as GeneratedKnowledgebaseAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/knowledgebase-app-sdk";
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

export type SdkworkKnowledgebaseAppClient = GeneratedKnowledgebaseAppClient;
export type SdkworkKnowledgebaseAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";

let knowledgebaseAppSdkClient: SdkworkKnowledgebaseAppClient | null = null;

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function resolveKnowledgebaseAppSdkBaseUrl(): string | null {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_KNOWLEDGEBASE_APP_API_BASE_URL");
  if (fromEnv) return fromEnv;
  // Gateway-routed deployments (cloud profiles and local dev ingress) serve
  // every app API under the same origin as the Agents API. Reuse the Agents
  // base URL fallback chain (public HTTP URL -> window origin) so the
  // knowledgebase SDK works without its own explicit VITE_ override.
  return resolveAgentsAppSdkBaseUrl();
}

export function isKnowledgebaseAppSdkConfigured(): boolean {
  return resolveKnowledgebaseAppSdkBaseUrl() !== null;
}

export function createKnowledgebaseAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkKnowledgebaseAppClientConfig {
  const baseUrl = resolveKnowledgebaseAppSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("knowledgebase app SDK base URL is not configured");
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

export function initKnowledgebaseAppSdkClient(
  config: SdkworkKnowledgebaseAppClientConfig = createKnowledgebaseAppSdkClientConfig(),
): SdkworkKnowledgebaseAppClient {
  knowledgebaseAppSdkClient = createClient(config);
  return knowledgebaseAppSdkClient;
}

export function getKnowledgebaseAppSdkClient(): SdkworkKnowledgebaseAppClient {
  return knowledgebaseAppSdkClient ?? initKnowledgebaseAppSdkClient();
}

export function resetKnowledgebaseAppSdkClient(): void {
  knowledgebaseAppSdkClient = null;
}

export type { KnowledgeMarketCatalogItem } from "@sdkwork/knowledgebase-app-sdk";
