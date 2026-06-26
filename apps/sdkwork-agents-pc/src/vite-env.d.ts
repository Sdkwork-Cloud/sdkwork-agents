/// <reference types="vite/client" />

declare module "*.css" {}

interface ImportMetaEnv {
  readonly VITE_SDKWORK_AGENTS_PC_APPLICATION_PUBLIC_HTTP_URL?: string;
  readonly VITE_SDKWORK_AGENTS_PC_APP_API_BASE_URL?: string;
  readonly VITE_SDKWORK_AGENTS_PC_APPBASE_APP_API_BASE_URL?: string;
  readonly VITE_SDKWORK_AGENTS_PC_BACKEND_API_BASE_URL?: string;
  readonly VITE_SDKWORK_AGENTS_PC_APPBASE_LOGIN_URL?: string;
  readonly SDKWORK_ACCESS_TOKEN?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

