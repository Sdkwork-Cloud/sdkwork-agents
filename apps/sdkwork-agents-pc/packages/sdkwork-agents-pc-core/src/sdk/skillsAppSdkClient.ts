import {
  createClient,
  type SdkworkAppClient as GeneratedSkillsAppClient,
  type SdkworkAppConfig,
} from "@sdkwork/skills-app-sdk";
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

export type SdkworkSkillsAppClient = GeneratedSkillsAppClient;
export type SdkworkSkillsAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const APP_API_SUFFIX = "/app/v3/api";

let skillsAppSdkClient: SdkworkSkillsAppClient | null = null;
let skillsAppSdkClientProvider: (() => SdkworkSkillsAppClient) | null = null;

export function configureSkillsAppSdkClientProvider(
  provider: () => SdkworkSkillsAppClient,
): void {
  skillsAppSdkClientProvider = provider;
  skillsAppSdkClient = null;
}

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function resolveSkillsAppSdkBaseUrl(): string | null {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_SKILLS_APP_API_BASE_URL")
    ?? readRuntimeEnv("VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL");
  if (fromEnv) return fromEnv;
  // Gateway-routed deployments (cloud profiles and local dev ingress) serve
  // every app API under the same origin as the Agents API. Reuse the Agents
  // base URL fallback chain (public HTTP URL -> window origin) so the
  // skills SDK works without its own explicit VITE_ override.
  return resolveAgentsAppSdkBaseUrl();
}

export function isSkillsAppSdkConfigured(): boolean {
  return resolveSkillsAppSdkBaseUrl() !== null;
}

export function createSkillsAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkSkillsAppClientConfig {
  const baseUrl = resolveSkillsAppSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("skills app SDK base URL is not configured");
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

export function initSkillsAppSdkClient(
  config: SdkworkSkillsAppClientConfig = createSkillsAppSdkClientConfig(),
): SdkworkSkillsAppClient {
  skillsAppSdkClient = createClient(config);
  return skillsAppSdkClient;
}

export function getSkillsAppSdkClient(): SdkworkSkillsAppClient {
  if (skillsAppSdkClientProvider) {
    return skillsAppSdkClientProvider();
  }
  return skillsAppSdkClient ?? initSkillsAppSdkClient();
}

export function resetSkillsAppSdkClient(): void {
  skillsAppSdkClient = null;
  skillsAppSdkClientProvider = null;
}

export type { SkillPackageRecord, SkillRecord } from "@sdkwork/skills-app-sdk";
