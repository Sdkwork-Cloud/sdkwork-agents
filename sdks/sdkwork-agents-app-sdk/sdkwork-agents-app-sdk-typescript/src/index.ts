import {
  SdkworkAppClient as GeneratedSdkworkAppClient,
} from '../generated/server-openapi/src/index';
import { APP_API_PREFIX } from '../generated/server-openapi/src/api/paths';
import type { SdkworkAppConfig } from '../generated/server-openapi/src/types/common';

export type { SdkworkAppConfig };
export * from '../generated/server-openapi/src/types';
export * from '../generated/server-openapi/src/api';
export * from '../generated/server-openapi/src/http';
export * from '../generated/server-openapi/src/auth';

function resolveTransportBaseUrl(appApiBaseUrl: string): string {
  const normalized = appApiBaseUrl.trim().replace(/\/+$/u, '');
  if (!normalized) {
    throw new Error('Agents App SDK base URL must not be empty.');
  }

  const prefixIndex = normalized.indexOf(APP_API_PREFIX);
  const hasCanonicalSurfaceSuffix = normalized.endsWith(APP_API_PREFIX);
  if (prefixIndex >= 0 && !hasCanonicalSurfaceSuffix) {
    throw new Error(
      `Agents App SDK base URL must identify a gateway root or end with ${APP_API_PREFIX}.`,
    );
  }

  const transportBaseUrl = hasCanonicalSurfaceSuffix
    ? normalized.slice(0, -APP_API_PREFIX.length)
    : normalized;
  if (transportBaseUrl.includes(APP_API_PREFIX)) {
    throw new Error(`Agents App API base URL must include ${APP_API_PREFIX} exactly once.`);
  }
  return transportBaseUrl;
}

export class SdkworkAppClient extends GeneratedSdkworkAppClient {
  constructor(config: SdkworkAppConfig) {
    super({
      ...config,
      baseUrl: resolveTransportBaseUrl(config.baseUrl),
    });
  }
}

export function createClient(config: SdkworkAppConfig): SdkworkAppClient {
  return new SdkworkAppClient(config);
}

export { completeAgentTurn } from './turns';
export type { CompleteAgentTurnResult } from './turns';
