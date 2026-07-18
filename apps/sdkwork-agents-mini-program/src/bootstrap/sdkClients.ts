import {
  configureAgentsAppSdkBaseUrl,
  configureAgentsAppSdkBootstrapAccessToken,
  initAgentsAppSdkClient,
} from "@sdkwork/agents-mp-core/sdk";

export interface AgentsMpSdkBootstrapOptions {
  appApiBaseUrl?: string;
  accessToken?: string;
}

export function bootstrapSdkClients(options: AgentsMpSdkBootstrapOptions = {}) {
  if (options.appApiBaseUrl) {
    configureAgentsAppSdkBaseUrl(options.appApiBaseUrl);
  }
  configureAgentsAppSdkBootstrapAccessToken(options.accessToken);
  const client = initAgentsAppSdkClient();
  return { agentsAppSdk: client };
}
