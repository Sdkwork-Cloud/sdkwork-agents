import { initAgentsAppSdkClient } from "@sdkwork/agents-pc-core/sdk";

import { registerHostAdapters } from "./hostAdapters";
import { bootstrapKnowledgeSelection } from "./knowledgeSelection";
import { bootstrapSiblingAppSdks } from "./siblingAppSdks";
import { bootstrapSdkClients } from "./sdkClients";

export function bootstrap() {
  registerHostAdapters();
  bootstrapSdkClients();
  bootstrapSiblingAppSdks();
  bootstrapKnowledgeSelection();
  initAgentsAppSdkClient();
}
