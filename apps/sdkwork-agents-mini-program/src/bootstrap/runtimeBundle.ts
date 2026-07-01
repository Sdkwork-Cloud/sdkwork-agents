import { bootstrap } from "./runtime";

export interface AgentsMiniProgramRuntimeOptions {
  appApiBaseUrl?: string;
}

export function bootstrapAgentsMiniProgram(options: AgentsMiniProgramRuntimeOptions = {}) {
  return bootstrap(options);
}
