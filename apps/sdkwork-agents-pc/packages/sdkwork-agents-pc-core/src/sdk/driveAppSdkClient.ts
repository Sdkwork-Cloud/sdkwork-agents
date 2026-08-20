import {
  createClient,
  type SdkworkDriveAppClient,
} from "@sdkwork/drive-app-sdk";
import type { SdkworkAppConfig } from "@sdkwork/drive-app-sdk";
import type { Interceptors } from "@sdkwork/sdk-common";

export type { MediaResource } from "@sdkwork/assets-app-sdk";

import {
  createSdkworkChatRequestContextInterceptors,
  getSdkworkChatGlobalTokenManager,
  readAppSdkSessionTokens,
  resolveAppSdkAccessToken,
  resolveAppSdkAuthToken,
  type SdkworkChatSession,
} from "../session/session";
import { readRuntimeEnv } from "./runtimeEnv";

export type SdkworkAgentsDriveAppClient = SdkworkDriveAppClient;
export type SdkworkAgentsDriveAppClientConfig = SdkworkAppConfig & {
  interceptors?: Interceptors;
};

const DEFAULT_DRIVE_APP_API_BASE_URL = "http://127.0.0.1:3900/app/v3/api";

let driveAppSdkClient: SdkworkAgentsDriveAppClient | null = null;
let driveAppSdkClientProvider: (() => SdkworkAgentsDriveAppClient) | null = null;

export function configureDriveAppSdkClientProvider(
  provider: () => SdkworkAgentsDriveAppClient,
): void {
  driveAppSdkClientProvider = provider;
  driveAppSdkClient = null;
}

export function resolveDriveAppSdkBaseUrl(): string {
  return readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_DRIVE_APP_API_BASE_URL")
    ?? DEFAULT_DRIVE_APP_API_BASE_URL;
}

export function createDriveAppSdkClientConfig(
  session?: SdkworkChatSession | null,
): SdkworkAgentsDriveAppClientConfig {
  const currentSession = session ?? readAppSdkSessionTokens();
  const envAccessToken = readRuntimeEnv("SDKWORK_ACCESS_TOKEN");
  return {
    baseUrl: resolveDriveAppSdkBaseUrl(),
    accessToken: resolveAppSdkAccessToken(currentSession) ?? envAccessToken,
    authToken: resolveAppSdkAuthToken(currentSession),
    interceptors: createSdkworkChatRequestContextInterceptors(
      () => readAppSdkSessionTokens() ?? currentSession,
    ),
    platform: "pc",
    tokenManager: getSdkworkChatGlobalTokenManager(),
  };
}

export function initDriveAppSdkClient(
  config: SdkworkAgentsDriveAppClientConfig = createDriveAppSdkClientConfig(),
): SdkworkAgentsDriveAppClient {
  driveAppSdkClient = createClient(config);
  return driveAppSdkClient;
}

export function getDriveAppSdkClient(): SdkworkAgentsDriveAppClient {
  if (driveAppSdkClientProvider) {
    return driveAppSdkClientProvider();
  }
  return driveAppSdkClient ?? initDriveAppSdkClient();
}

export function getDriveAppSdkClientWithSession(
  session = readAppSdkSessionTokens(),
): SdkworkAgentsDriveAppClient {
  if (driveAppSdkClientProvider) {
    return driveAppSdkClientProvider();
  }
  return initDriveAppSdkClient(createDriveAppSdkClientConfig(session));
}

export function resetDriveAppSdkClient(): void {
  driveAppSdkClient = null;
  driveAppSdkClientProvider = null;
}
