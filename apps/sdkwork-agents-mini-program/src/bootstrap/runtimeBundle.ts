import { bootstrap } from "./runtime";
import {
  configureAgentsAppSdkBaseUrl,
  getAgentsAppSdkClient,
} from "@sdkwork/agents-mp-core/sdk";

export interface AgentsMiniProgramRuntimeOptions {
  appApiBaseUrl?: string;
}

export function bootstrapAgentsMiniProgram(options: AgentsMiniProgramRuntimeOptions = {}) {
  return bootstrap(options);
}

export function getAgentsMpSdkClient() {
  return getAgentsAppSdkClient();
}

export { configureAgentsAppSdkBaseUrl };
