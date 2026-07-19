import type { IamAppContext } from '@sdkwork/iam-contracts';
import type { AuthTokenManager, AuthTokens, Interceptors, RequestConfig } from '@sdkwork/sdk-common';
import {
  clearDesktopSecureSessionRawValue,
  isDesktopSecureSessionStorageEnabled,
  readDesktopSecureSessionRawValue,
  writeDesktopSecureSessionRawValue,
} from './secureSessionStorage';

export interface SdkworkChatSessionUser {
  avatar?: string;
  chatId?: string;
  displayName?: string;
  email?: string;
  id?: string | number;
  name?: string;
  nickname?: string;
  phone?: string;
  userId?: string;
  username?: string;
}

export interface SdkworkChatSessionTokens {
  accessToken?: string;
  authToken?: string;
  refreshToken?: string;
}

export interface SdkworkChatAppContext extends Partial<IamAppContext> {
  actorId?: string;
  actorKind?: string;
  appId: string;
  deviceId?: string;
  tenantId: string;
  userId: string;
}

export interface SdkworkChatSession extends SdkworkChatSessionTokens {
  context?: SdkworkChatAppContext;
  expiresAt?: number;
  sessionId?: string;
  user?: SdkworkChatSessionUser;
}

export interface SdkworkChatSessionChangedDetail {
  session: SdkworkChatSession | null;
}

export type SdkworkChatRequestContext = Partial<SdkworkChatAppContext>;

const SDKWORK_AGENTS_PC_SESSION_KEY = 'sdkwork-agents-pc:session:v1';
const SDKWORK_AGENTS_PC_LEGACY_SESSION_STORAGE_KEY = 'sdkwork-agents-pc:session:v1';
export const SDKWORK_AGENTS_PC_SESSION_CHANGED_EVENT = 'sdkwork-agents-pc:auth-session-changed';

let sdkworkChatGlobalTokenManager: AuthTokenManager | null = null;
let sdkworkChatGlobalTokenManagerSession: SdkworkChatSession | null = null;
let persistedSessionRawValueCache: string | null | undefined;

function readPersistedSessionRawValue(): string | null {
  if (persistedSessionRawValueCache !== undefined) {
    return persistedSessionRawValueCache;
  }

  if (isDesktopSecureSessionStorageEnabled()) {
    return null;
  }

  migrateLegacySessionStorage();
  const localStorage = getLocalStorage();
  const durableValue = localStorage?.getItem(SDKWORK_AGENTS_PC_SESSION_KEY) ?? null;
  if (durableValue) {
    persistedSessionRawValueCache = durableValue;
    return durableValue;
  }

  const legacyValue = getSessionStorage()?.getItem(SDKWORK_AGENTS_PC_SESSION_KEY) ?? null;
  if (legacyValue) {
    localStorage?.setItem(SDKWORK_AGENTS_PC_SESSION_KEY, legacyValue);
    getSessionStorage()?.removeItem(SDKWORK_AGENTS_PC_SESSION_KEY);
  }
  persistedSessionRawValueCache = legacyValue;
  return legacyValue;
}

function handleDesktopSessionPersistError(operation: string, error: unknown): void {
  persistedSessionRawValueCache = undefined;
  console.error(`desktop secure session ${operation} failed`, error);
}

function writePersistedSessionRawValue(value: string): void {
  persistedSessionRawValueCache = value;
  if (isDesktopSecureSessionStorageEnabled()) {
    void writeDesktopSecureSessionRawValue(value).catch((error) => {
      handleDesktopSessionPersistError('write', error);
    });
    return;
  }

  getLocalStorage()?.setItem(SDKWORK_AGENTS_PC_SESSION_KEY, value);
  getSessionStorage()?.removeItem(SDKWORK_AGENTS_PC_SESSION_KEY);
}

function removePersistedSessionRawValue(): void {
  persistedSessionRawValueCache = null;
  if (isDesktopSecureSessionStorageEnabled()) {
    void clearDesktopSecureSessionRawValue().catch((error) => {
      handleDesktopSessionPersistError('clear', error);
    });
    return;
  }

  getSessionStorage()?.removeItem(SDKWORK_AGENTS_PC_SESSION_KEY);
  getLocalStorage()?.removeItem(SDKWORK_AGENTS_PC_LEGACY_SESSION_STORAGE_KEY);
}

export async function hydrateAppSdkSessionFromSecureStorage(): Promise<void> {
  if (!isDesktopSecureSessionStorageEnabled()) {
    return;
  }

  migrateLegacySessionStorage();
  const secureValue = await readDesktopSecureSessionRawValue();
  if (secureValue) {
    persistedSessionRawValueCache = secureValue;
    removePersistedSessionRawValueFromWebStorageOnly();
    return;
  }

  const legacyValue = getSessionStorage()?.getItem(SDKWORK_AGENTS_PC_SESSION_KEY)
    ?? getLocalStorage()?.getItem(SDKWORK_AGENTS_PC_LEGACY_SESSION_STORAGE_KEY)
    ?? null;
  if (!legacyValue) {
    persistedSessionRawValueCache = null;
    return;
  }

  persistedSessionRawValueCache = legacyValue;
  await writeDesktopSecureSessionRawValue(legacyValue);
  removePersistedSessionRawValueFromWebStorageOnly();
}

function removePersistedSessionRawValueFromWebStorageOnly(): void {
  getSessionStorage()?.removeItem(SDKWORK_AGENTS_PC_SESSION_KEY);
  getLocalStorage()?.removeItem(SDKWORK_AGENTS_PC_LEGACY_SESSION_STORAGE_KEY);
}

function getLocalStorage(): Storage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }
  return window.localStorage;
}

function getSessionStorage(): Storage | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }
  return window.sessionStorage;
}

function migrateLegacySessionStorage(): void {
  const sessionStorage = getSessionStorage();
  const localStorage = getLocalStorage();
  if (!sessionStorage || !localStorage) {
    return;
  }

  const legacyValue = sessionStorage.getItem(SDKWORK_AGENTS_PC_SESSION_KEY);
  if (!legacyValue || localStorage.getItem(SDKWORK_AGENTS_PC_SESSION_KEY)) {
    return;
  }

  localStorage.setItem(SDKWORK_AGENTS_PC_SESSION_KEY, legacyValue);
  sessionStorage.removeItem(SDKWORK_AGENTS_PC_SESSION_KEY);
}

function dispatchAppSdkSessionChanged(session: SdkworkChatSession | null): void {
  if (typeof window === 'undefined') {
    return;
  }
  window.dispatchEvent(new CustomEvent(
    SDKWORK_AGENTS_PC_SESSION_CHANGED_EVENT,
    { detail: { session } },
  ));
}

function normalizeToken(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined;
}

function normalizeString(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined;
}

function normalizeStringArray(value: unknown): string[] {
  if (Array.isArray(value)) {
    return value
      .map((item) => normalizeString(item))
      .filter(Boolean) as string[];
  }

  const normalized = normalizeString(value);
  if (!normalized) {
    return [];
  }
  const separator = normalized.includes(',') ? /,/u : /\s+/u;
  return normalized
    .split(separator)
    .map((item) => item.trim())
    .filter(Boolean);
}

function normalizeExpiresAt(value: unknown): number | undefined {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return value;
  }
  if (typeof value !== 'string' || !value.trim()) {
    return undefined;
  }
  const timestamp = Date.parse(value);
  return Number.isFinite(timestamp) ? timestamp : undefined;
}

function decodeBase64UrlJson(value: string): Record<string, unknown> | undefined {
  try {
    const normalized = value.replace(/-/gu, '+').replace(/_/gu, '/');
    const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, '=');
    const binary = atob(padded);
    const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
    const text = new TextDecoder().decode(bytes);
    const parsed = JSON.parse(text);
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : undefined;
  } catch {
    return undefined;
  }
}

function decodeJwtClaims(token?: string): Record<string, unknown> | undefined {
  const normalized = normalizeToken(token);
  if (!normalized) {
    return undefined;
  }
  const [, payload] = normalized.split('.');
  if (!payload) {
    return undefined;
  }
  return decodeBase64UrlJson(payload);
}

function readSessionJwtClaims(session?: SdkworkChatSession | null): Record<string, unknown>[] {
  return [
    decodeJwtClaims(session?.accessToken),
    decodeJwtClaims(session?.authToken),
  ].filter(Boolean) as Record<string, unknown>[];
}

function pickClaimString(
  session: SdkworkChatSession | null | undefined,
  claimKeys: string[],
  ...fallbacks: unknown[]
): string | undefined {
  for (const claims of readSessionJwtClaims(session)) {
    for (const key of claimKeys) {
      const value = normalizeString(claims[key]);
      if (value) {
        return value;
      }
    }
  }
  return normalizeString(fallbacks.find((value) => normalizeString(value)));
}

function pickClaimStringArray(
  session: SdkworkChatSession | null | undefined,
  claimKeys: string[],
  fallback?: unknown,
): string[] {
  for (const claims of readSessionJwtClaims(session)) {
    for (const key of claimKeys) {
      const values = normalizeStringArray(claims[key]);
      if (values.length > 0) {
        return values;
      }
    }
  }
  return normalizeStringArray(fallback);
}

function normalizeContext(value: unknown): SdkworkChatAppContext | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return undefined;
  }
  const context = value as Partial<IamAppContext>;
  const record = value as Record<string, unknown>;
  const appId = normalizeString(context.appId) ?? normalizeString(record.app_id);
  const tenantId = normalizeString(context.tenantId) ?? normalizeString(record.tenant_id);
  const userId = normalizeString(context.userId) ?? normalizeString(record.user_id);
  const sessionId = normalizeString(context.sessionId) ?? normalizeString(record.session_id);
  const organizationId = normalizeString(context.organizationId) ?? normalizeString(record.organization_id);
  const environment = normalizeString(context.environment) ?? normalizeString(record.env);
  const deploymentMode = normalizeString(context.deploymentMode) ?? normalizeString(record.deployment_mode);
  const authLevel = normalizeString(context.authLevel) ?? normalizeString(record.auth_level);
  if (!appId || !tenantId || !userId) {
    return undefined;
  }
  return {
    ...context,
    appId,
    ...(organizationId ? { organizationId } : {}),
    tenantId,
    userId,
    ...(sessionId ? { sessionId } : {}),
    ...(environment ? { environment: environment as IamAppContext['environment'] } : {}),
    ...(deploymentMode ? { deploymentMode: deploymentMode as IamAppContext['deploymentMode'] } : {}),
    ...(authLevel ? { authLevel: authLevel as IamAppContext['authLevel'] } : {}),
    dataScope: normalizeStringArray(context.dataScope ?? record.data_scope),
    permissionScope: normalizeStringArray(context.permissionScope ?? record.permission_scope),
  };
}

export function normalizeSdkworkChatSessionUser(value: unknown): SdkworkChatSessionUser | undefined {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return undefined;
  }
  const user = value as Partial<SdkworkChatSessionUser>;
  const id = normalizeString(user.userId) ?? normalizeString(user.id);
  const avatar = normalizeString(user.avatar);
  const userRecord = value as Record<string, unknown>;
  const chatId = normalizeString(userRecord.chatId)
    ?? normalizeString(userRecord.chat_id)
    ?? normalizeString(userRecord.SDKWORK_AGENTS_PC_id);
  const normalized: SdkworkChatSessionUser = {
    ...(avatar ? { avatar } : {}),
    ...(chatId ? { chatId } : {}),
    ...(normalizeString(user.displayName) ? { displayName: normalizeString(user.displayName) } : {}),
    ...(normalizeString(user.email) ? { email: normalizeString(user.email) } : {}),
    ...(id ? { id } : {}),
    ...(normalizeString(user.name) ? { name: normalizeString(user.name) } : {}),
    ...(normalizeString(user.nickname) ? { nickname: normalizeString(user.nickname) } : {}),
    ...(normalizeString(user.phone) ? { phone: normalizeString(user.phone) } : {}),
    ...(normalizeString(user.userId) ? { userId: normalizeString(user.userId) } : {}),
    ...(normalizeString(user.username) ? { username: normalizeString(user.username) } : {}),
  };
  return Object.keys(normalized).length > 0 ? normalized : undefined;
}

function normalizeSession(value: unknown): SdkworkChatSession | null {
  if (!value || typeof value !== 'object') {
    return null;
  }

  const candidate = value as SdkworkChatSession & { expiresAt?: unknown };
  const context = normalizeContext(candidate.context);
  const sessionId = normalizeString(candidate.sessionId) ?? normalizeString(context?.sessionId);
  const session: SdkworkChatSession = {
    accessToken: normalizeToken(candidate.accessToken),
    authToken: normalizeToken(candidate.authToken),
    refreshToken: normalizeToken(candidate.refreshToken),
    ...(context ? { context } : {}),
    ...(typeof candidate.expiresAt !== 'undefined'
      ? { expiresAt: normalizeExpiresAt(candidate.expiresAt) }
      : {}),
    ...(sessionId ? { sessionId } : {}),
    ...(normalizeSdkworkChatSessionUser(candidate.user) ? { user: normalizeSdkworkChatSessionUser(candidate.user) } : {}),
  };

  return session.authToken && session.accessToken ? session : null;
}

export function readAppSdkSessionTokens(): SdkworkChatSession | null {
  const rawValue = readPersistedSessionRawValue();
  if (!rawValue) {
    return null;
  }

  try {
    return normalizeSession(JSON.parse(rawValue));
  } catch {
    removePersistedSessionRawValue();
    return null;
  }
}

export function persistAppSdkSessionTokens(session: SdkworkChatSession): SdkworkChatSession {
  const normalizedSession = normalizeSession(session);
  if (!normalizedSession) {
    clearAppSdkSessionTokens();
    throw new Error('Sdkwork IM session requires authToken and accessToken.');
  }

  writePersistedSessionRawValue(JSON.stringify(normalizedSession));
  syncSdkworkChatGlobalTokenManager(normalizedSession);
  dispatchAppSdkSessionChanged(normalizedSession);
  return normalizedSession;
}

export function applyAppSdkSessionTokens(session: SdkworkChatSession): SdkworkChatSession {
  return persistAppSdkSessionTokens(session);
}

export function clearAppSdkSessionTokens(): void {
  removePersistedSessionRawValue();
  syncSdkworkChatGlobalTokenManager(null);
  dispatchAppSdkSessionChanged(null);
}

export function resolveAppSdkAccessToken(session = readAppSdkSessionTokens()): string | undefined {
  return session?.accessToken;
}

export function resolveAppSdkAuthToken(session = readAppSdkSessionTokens()): string | undefined {
  return session?.authToken;
}

export function resolveAppSdkRefreshToken(session = readAppSdkSessionTokens()): string | undefined {
  return session?.refreshToken;
}

export function resolveAppSdkTenantId(session = readAppSdkSessionTokens()): string | undefined {
  return pickClaimString(session, ['tenantId', 'tenant_id', 'tid'], session?.context?.tenantId);
}

export function resolveAppSdkOrganizationId(session = readAppSdkSessionTokens()): string | undefined {
  return pickClaimString(
    session,
    ['organizationId', 'organization_id', 'orgId', 'org_id', 'oid'],
    session?.context?.organizationId,
  );
}

export function resolveAppSdkUserId(session = readAppSdkSessionTokens()): string | undefined {
  return pickClaimString(
    session,
    ['userId', 'user_id', 'uid', 'sub', 'principalId', 'principal_id', 'accountId', 'account_id'],
    session?.context?.userId,
    session?.user?.userId,
    session?.user?.id,
  );
}

export function resolveAppSdkSessionId(session = readAppSdkSessionTokens()): string | undefined {
  return pickClaimString(
    session,
    ['sessionId', 'session_id', 'sid'],
    session?.sessionId,
    session?.context?.sessionId,
  );
}

function resolveAppSdkAppId(session?: SdkworkChatSession | null): string | undefined {
  return pickClaimString(session, ['appId', 'app_id', 'azp', 'aud'], session?.context?.appId);
}

function resolveAppSdkEnvironment(session?: SdkworkChatSession | null): string | undefined {
  return pickClaimString(session, ['environment', 'env'], session?.context?.environment);
}

function resolveAppSdkDeploymentMode(session?: SdkworkChatSession | null): string | undefined {
  return pickClaimString(session, ['deploymentMode', 'deployment_mode'], session?.context?.deploymentMode);
}

function resolveAppSdkAuthLevel(session?: SdkworkChatSession | null): string | undefined {
  return pickClaimString(session, ['authLevel', 'auth_level', 'acr'], session?.context?.authLevel);
}

function resolveAppSdkActorId(session?: SdkworkChatSession | null): string | undefined {
  return pickClaimString(session, ['actorId', 'actor_id'], session?.context?.actorId);
}

function resolveAppSdkActorKind(session?: SdkworkChatSession | null): string | undefined {
  return pickClaimString(session, ['actorKind', 'actor_kind'], session?.context?.actorKind);
}

function resolveAppSdkDeviceId(session?: SdkworkChatSession | null): string | undefined {
  return pickClaimString(session, ['deviceId', 'device_id'], session?.context?.deviceId);
}

export function createSdkworkChatRequestContext(
  session = readAppSdkSessionTokens(),
): SdkworkChatRequestContext | undefined {
  const context = session?.context;
  const appId = resolveAppSdkAppId(session);
  const tenantId = resolveAppSdkTenantId(session);
  const organizationId = resolveAppSdkOrganizationId(session);
  const userId = resolveAppSdkUserId(session);
  const sessionId = resolveAppSdkSessionId(session);
  const environment = resolveAppSdkEnvironment(session) as IamAppContext['environment'] | undefined;
  const deploymentMode = resolveAppSdkDeploymentMode(session) as IamAppContext['deploymentMode'] | undefined;
  const authLevel = resolveAppSdkAuthLevel(session) as IamAppContext['authLevel'] | undefined;
  const actorId = resolveAppSdkActorId(session);
  const actorKind = resolveAppSdkActorKind(session);
  const deviceId = resolveAppSdkDeviceId(session);
  const dataScope = pickClaimStringArray(session, ['dataScope', 'data_scope'], context?.dataScope);
  const permissionScope = pickClaimStringArray(
    session,
    ['permissionScope', 'permission_scope', 'scope', 'scp'],
    context?.permissionScope,
  );
  const requestContext: SdkworkChatRequestContext = {
    ...(appId ? { appId } : {}),
    ...(tenantId ? { tenantId } : {}),
    ...(organizationId ? { organizationId } : {}),
    ...(userId ? { userId } : {}),
    ...(sessionId ? { sessionId } : {}),
    ...(environment ? { environment } : {}),
    ...(deploymentMode ? { deploymentMode } : {}),
    ...(authLevel ? { authLevel } : {}),
    ...(dataScope.length ? { dataScope } : {}),
    ...(permissionScope.length ? { permissionScope } : {}),
    ...(actorId ? { actorId } : {}),
    ...(actorKind ? { actorKind } : {}),
    ...(deviceId ? { deviceId } : {}),
  };
  return Object.keys(requestContext).length > 0 ? requestContext : undefined;
}

export function createSdkworkChatRequestContextInterceptors(
  _sessionOrReader?: SdkworkChatSession | null | (() => SdkworkChatSession | null),
): Interceptors {
  return {
    request: [
      (config: RequestConfig): RequestConfig => {
        return {
          ...config,
          headers: { ...(config.headers ?? {}) },
        };
      },
    ],
    response: [],
    error: [],
  };
}

export function createSdkworkChatSessionTokenManager(
  sessionOrReader?: SdkworkChatSession | null | (() => SdkworkChatSession | null),
): AuthTokenManager {
  let currentSession = typeof sessionOrReader === 'function' ? null : sessionOrReader ?? null;
  let transientAccessToken: string | undefined;
  const readConfiguredSession = () => (
    typeof sessionOrReader === 'function'
      ? sessionOrReader()
      : currentSession
  );
  const readCurrentSession = () => readConfiguredSession() ?? readAppSdkSessionTokens();
  const resolveManagedAccessToken = () => (
    resolveAppSdkAccessToken(readCurrentSession()) ?? transientAccessToken
  );
  const isExpired = () => {
    const expiresAt = readCurrentSession()?.expiresAt;
    return typeof expiresAt === 'number' && Number.isFinite(expiresAt) && Date.now() >= expiresAt;
  };
  const storeTransientAccessToken = (accessToken: string | undefined): void => {
    transientAccessToken = accessToken ? normalizeToken(accessToken) : undefined;
    currentSession = null;
  };
  const updateTokens = (tokens: AuthTokens): void => {
    const existing = readCurrentSession() ?? {};
    const nextAccessToken = tokens.accessToken ?? existing.accessToken;
    const nextAuthToken = tokens.authToken ?? existing.authToken;

    if (nextAccessToken && !nextAuthToken) {
      storeTransientAccessToken(nextAccessToken);
      return;
    }

    transientAccessToken = undefined;
    const expiresAt = typeof tokens.expiresAt === 'number' && Number.isFinite(tokens.expiresAt)
      ? tokens.expiresAt
      : typeof tokens.expiresIn === 'number' && Number.isFinite(tokens.expiresIn)
        ? Date.now() + tokens.expiresIn * 1000
        : existing.expiresAt;
    currentSession = applyAppSdkSessionTokens({
      ...existing,
      accessToken: nextAccessToken,
      authToken: nextAuthToken,
      refreshToken: tokens.refreshToken ?? existing.refreshToken,
      ...(expiresAt ? { expiresAt } : {}),
    });
  };
  const patchTokens = (tokens: Partial<SdkworkChatSessionTokens>): void => {
    const existing = readCurrentSession() ?? {};
    const next = {
      ...existing,
      ...tokens,
    };
    const normalizedSession = normalizeSession(next);
    if (!normalizedSession) {
      const accessOnly = normalizeToken(next.accessToken);
      const authOnly = normalizeToken(next.authToken);
      if (accessOnly && !authOnly) {
        storeTransientAccessToken(accessOnly);
        return;
      }

      transientAccessToken = undefined;
      currentSession = null;
      clearAppSdkSessionTokens();
      return;
    }
    transientAccessToken = undefined;
    currentSession = applyAppSdkSessionTokens(normalizedSession);
  };

  const hasDualTokens = () => {
    const current = readCurrentSession();
    return Boolean(current?.authToken && current?.accessToken);
  };

  return {
    getAuthToken: () => resolveAppSdkAuthToken(readCurrentSession()),
    getAccessToken: () => resolveManagedAccessToken(),
    getRefreshToken: () => resolveAppSdkRefreshToken(readCurrentSession()),
    getTokens: () => {
      const current = readCurrentSession();
      const accessToken = resolveManagedAccessToken();
      return {
        ...(accessToken ? { accessToken } : {}),
        ...(current?.authToken ? { authToken: current.authToken } : {}),
        ...(current?.refreshToken ? { refreshToken: current.refreshToken } : {}),
        ...(current?.expiresAt ? { expiresAt: current.expiresAt } : {}),
      };
    },
    setTokens: updateTokens,
    setAccessToken: (token: string) => patchTokens({ accessToken: normalizeToken(token) }),
    setAuthToken: (token: string) => patchTokens({ authToken: normalizeToken(token) }),
    setRefreshToken: (token: string) => patchTokens({ refreshToken: normalizeToken(token) }),
    clearTokens: () => {
      currentSession = null;
      transientAccessToken = undefined;
      clearAppSdkSessionTokens();
    },
    clearAuthToken: () => patchTokens({ authToken: undefined }),
    clearAccessToken: () => {
      transientAccessToken = undefined;
      patchTokens({ accessToken: undefined });
    },
    isExpired,
    isValid: () => hasDualTokens() && !isExpired(),
    hasToken: () => hasDualTokens(),
    hasAuthToken: () => Boolean(resolveAppSdkAuthToken(readCurrentSession())),
    hasAccessToken: () => Boolean(resolveManagedAccessToken()),
    willExpireIn: (seconds: number) => {
      const expiresAt = readCurrentSession()?.expiresAt;
      return typeof expiresAt === 'number' && Number.isFinite(expiresAt) && Date.now() + seconds * 1000 >= expiresAt;
    },
  };
}

export function syncSdkworkChatGlobalTokenManager(session: SdkworkChatSession | null = readAppSdkSessionTokens()): void {
  sdkworkChatGlobalTokenManagerSession = session ? normalizeSession(session) : null;
}

export function getSdkworkChatGlobalTokenManager(): AuthTokenManager {
  syncSdkworkChatGlobalTokenManager(readAppSdkSessionTokens());
  if (!sdkworkChatGlobalTokenManager) {
    sdkworkChatGlobalTokenManager = createSdkworkChatSessionTokenManager(
      () => sdkworkChatGlobalTokenManagerSession,
    );
  }
  return sdkworkChatGlobalTokenManager;
}

export function isAppSdkSessionExpired(session = readAppSdkSessionTokens()): boolean {
  const expiresAt = session?.expiresAt;
  return typeof expiresAt === 'number' && Number.isFinite(expiresAt) && Date.now() >= expiresAt;
}

export function isAppSdkSessionAuthenticated(session = readAppSdkSessionTokens()): boolean {
  return Boolean(session?.authToken && session?.accessToken) && !isAppSdkSessionExpired(session);
}
