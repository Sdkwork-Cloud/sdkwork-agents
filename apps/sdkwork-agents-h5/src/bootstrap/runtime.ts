import { initAgentsAppSdkClient } from "@sdkwork/agents-h5-core/sdk";

import { registerHostAdapters } from "./hostAdapters";
import { bootstrapKnowledgeSelection } from "./knowledgeSelection";
import { bootstrapSdkClients } from "./sdkClients";

export function bootstrap() {
  registerHostAdapters();
  bootstrapSdkClients();
  bootstrapKnowledgeSelection();
  initAgentsAppSdkClient();
}
