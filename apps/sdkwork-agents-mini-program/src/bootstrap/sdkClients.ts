import { configureAgentsAppSdkBaseUrl, initAgentsAppSdkClient } from "@sdkwork/agents-mp-core/sdk";

export interface AgentsMpSdkBootstrapOptions {
  appApiBaseUrl?: string;
}

export function bootstrapSdkClients(options: AgentsMpSdkBootstrapOptions = {}) {
  if (options.appApiBaseUrl) {
    configureAgentsAppSdkBaseUrl(options.appApiBaseUrl);
  }
  const client = initAgentsAppSdkClient();
  return { agentsAppSdk: client };
}
