import { initAgentsAppSdkClient } from "@sdkwork/agents-pc-core/sdk";

import { registerHostAdapters } from "./hostAdapters";
import { bootstrapSdkClients } from "./sdkClients";

export function bootstrap() {
  registerHostAdapters();
  bootstrapSdkClients();
  initAgentsAppSdkClient();
}
