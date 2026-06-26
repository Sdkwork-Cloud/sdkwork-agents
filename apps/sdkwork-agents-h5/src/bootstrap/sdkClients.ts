import { resolveEnvironment } from "./environment";

export function bootstrapSdkClients() {
  const environment = resolveEnvironment();
  return {
    apiBaseUrl: environment.apiBaseUrl,
    backendApiBaseUrl: environment.backendApiBaseUrl,
  };
}
