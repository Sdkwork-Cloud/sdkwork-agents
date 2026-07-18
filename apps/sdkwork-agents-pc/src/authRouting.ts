import {
  isAppSdkSessionAuthenticated,
  type SdkworkChatSession,
} from '@sdkwork/agents-pc-core/session';

export const AUTH_BASE_PATH = '/auth';
export const AUTH_LOGIN_PATH = '/auth/login';

export function isAuthRoute(pathname: string): boolean {
  const normalized = normalizePath(pathname);
  return normalized === AUTH_BASE_PATH || normalized.startsWith(`${AUTH_BASE_PATH}/`);
}

export function isSessionReady(session: SdkworkChatSession | null): boolean {
  const context = session?.context;
  return Boolean(
    isAppSdkSessionAuthenticated(session)
    && context?.appId
    && context.authLevel
    && context.deploymentMode
    && context.environment
    && (context.sessionId || session.sessionId)
    && context.tenantId
    && context.userId,
  );
}

export function buildLoginRedirect(pathname: string, search = '', hash = ''): string {
  const returnPath = `${normalizePath(pathname)}${search}${hash}`;
  return `${AUTH_LOGIN_PATH}?redirect=${encodeURIComponent(returnPath)}`;
}

export function readSafeRedirect(search: string): string {
  const value = new URLSearchParams(search.replace(/^\?/u, '')).get('redirect');
  if (!value) {
    return '/';
  }
  let decoded: string;
  try {
    decoded = decodeURIComponent(value);
  } catch {
    return '/';
  }
  if (!decoded.startsWith('/') || decoded.startsWith('//')) {
    return '/';
  }
  const target = new URL(decoded, 'http://sdkwork-agents.local');
  if (isAuthRoute(target.pathname)) {
    return '/';
  }
  return `${target.pathname}${target.search}${target.hash}`;
}

function normalizePath(pathname: string): string {
  if (!pathname.trim()) {
    return '/';
  }
  return pathname.startsWith('/') ? pathname : `/${pathname}`;
}
