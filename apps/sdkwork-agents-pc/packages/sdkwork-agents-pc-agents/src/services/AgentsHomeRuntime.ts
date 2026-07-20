import {
  configureAgentsAppSdkClientProvider,
  type SdkworkAgentsAppClient,
} from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';
import {
  configureDriveAppSdkClientProvider,
  type SdkworkAgentsDriveAppClient,
} from '@sdkwork/agents-pc-core/sdk/driveAppSdkClient';

export interface AgentsHomeRuntime {
  getAgentsAppSdkClient: () => SdkworkAgentsAppClient;
  getDriveAppSdkClient: () => SdkworkAgentsDriveAppClient;
}

export function configureAgentsHomeRuntime(runtime: AgentsHomeRuntime): void {
  configureAgentsAppSdkClientProvider(runtime.getAgentsAppSdkClient);
  configureDriveAppSdkClientProvider(runtime.getDriveAppSdkClient);
}
