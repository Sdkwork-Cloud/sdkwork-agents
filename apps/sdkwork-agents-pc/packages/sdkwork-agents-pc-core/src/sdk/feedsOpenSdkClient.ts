import {
  createClient,
  type SdkworkFeedsOpenClient,
  type SdkworkCustomConfig,
} from "@sdkwork/feeds-sdk";

import { readRuntimeEnv } from "./runtimeEnv";

/**
 * Standard feeds stream open-surface client.
 *
 * Feed streams are categorized by `feed_type` and isolated by `stream_id`
 * (news, community circles, moments/朋友圈, inspiration assets never
 * interfere with each other). Product surfaces read curated streams through
 * this client: operations are `skipAuth`, so content stays browsable without
 * login and without session tokens.
 */
export type SdkworkFeedsClient = SdkworkFeedsOpenClient;

const APP_API_SUFFIX = "/app/v3/api";

let feedsOpenSdkClient: SdkworkFeedsClient | null = null;
let feedsOpenSdkClientProvider: (() => SdkworkFeedsClient) | null = null;

function normalizeGeneratedSdkBaseUrl(baseUrl: string): string {
  const normalized = baseUrl.replace(/\/+$/u, "");
  if (normalized.endsWith(APP_API_SUFFIX)) {
    return normalized.slice(0, -APP_API_SUFFIX.length) || normalized;
  }
  return normalized;
}

export function configureFeedsOpenSdkClientProvider(
  provider: () => SdkworkFeedsClient,
): void {
  feedsOpenSdkClientProvider = provider;
  feedsOpenSdkClient = null;
}

export function resolveFeedsOpenSdkBaseUrl(): string | null {
  const fromEnv = readRuntimeEnv("VITE_SDKWORK_AGENTS_PC_FEEDS_OPEN_API_BASE_URL");
  if (fromEnv) return fromEnv;
  // Deployment topology: the feeds capability runs as its own gateway
  // (e.g. http://127.0.0.1:18095 in standalone dev); every profile configures
  // the explicit feeds open URL above. Cloud profiles may serve the feeds
  // open surface on the same origin as other app surfaces, so fall back to
  // the platform gateway URL when the explicit override is absent.
  return (
    readRuntimeEnv("VITE_SDKWORK_AGENTS_PLATFORM_API_GATEWAY_HTTP_URL")
    ?? null
  );
}

export function createFeedsOpenSdkClientConfig(): SdkworkCustomConfig {
  const baseUrl = resolveFeedsOpenSdkBaseUrl();
  if (!baseUrl) {
    throw new Error("feeds open SDK base URL is not configured");
  }
  return {
    // Public operations skip auth; the open surface needs no session tokens.
    baseUrl: normalizeGeneratedSdkBaseUrl(baseUrl),
    platform: "pc",
  };
}

export function initFeedsOpenSdkClient(
  config: SdkworkCustomConfig = createFeedsOpenSdkClientConfig(),
): SdkworkFeedsClient {
  feedsOpenSdkClient = createClient(config);
  return feedsOpenSdkClient;
}

export function getFeedsOpenSdkClient(): SdkworkFeedsClient {
  if (feedsOpenSdkClientProvider) {
    return feedsOpenSdkClientProvider();
  }
  return feedsOpenSdkClient ?? initFeedsOpenSdkClient();
}

export function resetFeedsOpenSdkClient(): void {
  feedsOpenSdkClient = null;
  feedsOpenSdkClientProvider = null;
}

export type { FeedItem, FeedStream, SdkWorkPageData } from "@sdkwork/feeds-sdk";
