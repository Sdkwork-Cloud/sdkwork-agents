import {
  configureAgentsAppSdkClientProvider,
  type SdkworkAgentsAppClient,
} from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';
import {
  configureAssetsAppSdkClientProvider,
  type SdkworkAgentsAssetsAppClient,
} from '@sdkwork/agents-pc-core/sdk/assetsAppSdkClient';
import {
  configureDriveAppSdkClientProvider,
  type SdkworkAgentsDriveAppClient,
} from '@sdkwork/agents-pc-core/sdk/driveAppSdkClient';

export interface AgentsHomeRuntime {
  getAgentsAppSdkClient: () => SdkworkAgentsAppClient;
  getAssetsAppSdkClient: () => SdkworkAgentsAssetsAppClient;
  getDriveAppSdkClient: () => SdkworkAgentsDriveAppClient;
}

export function configureAgentsHomeRuntime(runtime: AgentsHomeRuntime): void {
  configureAgentsAppSdkClientProvider(runtime.getAgentsAppSdkClient);
  configureAssetsAppSdkClientProvider(runtime.getAssetsAppSdkClient);
  configureDriveAppSdkClientProvider(runtime.getDriveAppSdkClient);
}
