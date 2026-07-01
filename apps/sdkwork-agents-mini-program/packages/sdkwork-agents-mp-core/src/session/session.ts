const SDKWORK_AGENTS_MP_SESSION_KEY = "sdkwork-agents-mp:session:v1";

export interface SdkworkAgentsMpSessionTokens {
  accessToken?: string;
  authToken?: string;
}

export interface SdkworkAgentsMpSession extends SdkworkAgentsMpSessionTokens {
  sessionId?: string;
}

function readWxStorage(key: string): string | null {
  try {
    const wxApi = (globalThis as { wx?: { getStorageSync?: (k: string) => unknown } }).wx;
    const value = wxApi?.getStorageSync?.(key);
    if (typeof value === "string" && value.trim().length > 0) {
      return value.trim();
    }
  } catch {
    return null;
  }
  return null;
}

function writeWxStorage(key: string, value: string): void {
  try {
    const wxApi = (globalThis as { wx?: { setStorageSync?: (k: string, v: string) => void } }).wx;
    wxApi?.setStorageSync?.(key, value);
  } catch {
    // ignore mini program storage failures in bootstrap-only paths
  }
}

function removeWxStorage(key: string): void {
  try {
    const wxApi = (globalThis as { wx?: { removeStorageSync?: (k: string) => void } }).wx;
    wxApi?.removeStorageSync?.(key);
  } catch {
    // ignore
  }
}

export function readAppSdkSessionTokens(): SdkworkAgentsMpSession | null {
  const raw = readWxStorage(SDKWORK_AGENTS_MP_SESSION_KEY);
  if (!raw) {
    return null;
  }
  try {
    const parsed = JSON.parse(raw) as SdkworkAgentsMpSession;
    if (!parsed || typeof parsed !== "object") {
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}

export function writeAppSdkSessionTokens(session: SdkworkAgentsMpSession | null): void {
  if (!session) {
    removeWxStorage(SDKWORK_AGENTS_MP_SESSION_KEY);
    return;
  }
  writeWxStorage(SDKWORK_AGENTS_MP_SESSION_KEY, JSON.stringify(session));
}

export function resolveAppSdkAccessToken(session: SdkworkAgentsMpSession | null | undefined): string | undefined {
  const token = session?.accessToken?.trim();
  return token && token.length > 0 ? token : undefined;
}

export function resolveAppSdkAuthToken(session: SdkworkAgentsMpSession | null | undefined): string | undefined {
  const token = session?.authToken?.trim();
  return token && token.length > 0 ? token : undefined;
}
