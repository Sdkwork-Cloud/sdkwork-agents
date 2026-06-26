export interface AgentsEnvironment {
  apiBaseUrl: string;
  appbaseAppApiBaseUrl: string;
  backendApiBaseUrl: string;
  appbaseLoginUrl: string;
}

function normalizeBaseUrl(value: string | undefined, fallback: string): string {
  const normalized = String(value ?? "").trim();
  return normalized || fallback;
}

function deriveAppApiBaseUrl(applicationPublicHttpUrl: string): string {
  return `${applicationPublicHttpUrl.replace(/\/+$/u, "")}/app/v3/api`;
}

function deriveBackendApiBaseUrl(applicationPublicHttpUrl: string): string {
  return `${applicationPublicHttpUrl.replace(/\/+$/u, "")}/backend/v3/api`;
}

export function resolveEnvironment(): AgentsEnvironment {
  const applicationPublicHttpUrl = normalizeBaseUrl(
    import.meta.env.VITE_SDKWORK_AGENTS_PC_APPLICATION_PUBLIC_HTTP_URL,
    "http://127.0.0.1:8095",
  );

  return {
    apiBaseUrl: normalizeBaseUrl(
      import.meta.env.VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL,
      deriveAppApiBaseUrl(applicationPublicHttpUrl),
    ),
    appbaseAppApiBaseUrl: normalizeBaseUrl(
      import.meta.env.VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL,
      deriveAppApiBaseUrl(applicationPublicHttpUrl),
    ),
    backendApiBaseUrl: normalizeBaseUrl(
      import.meta.env.VITE_SDKWORK_AGENTS_PC_BACKEND_API_BASE_URL,
      deriveBackendApiBaseUrl(applicationPublicHttpUrl),
    ),
    appbaseLoginUrl: normalizeBaseUrl(
      import.meta.env.VITE_SDKWORK_AGENTS_PC_APPBASE_LOGIN_URL,
      "http://127.0.0.1:3900",
    ),
  };
}
