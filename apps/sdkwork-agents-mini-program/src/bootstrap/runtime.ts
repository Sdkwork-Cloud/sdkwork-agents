import { bootstrapSdkClients } from "./sdkClients";

export interface AgentsMiniProgramBootstrapOptions {
  appApiBaseUrl?: string;
}

export function bootstrap(options: AgentsMiniProgramBootstrapOptions = {}) {
  const sdk = bootstrapSdkClients(options);
  return { ready: true, ...sdk };
}
