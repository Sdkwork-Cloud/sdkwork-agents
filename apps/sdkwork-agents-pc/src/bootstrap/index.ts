import {
  initAgentsAppSdkClient,
  initDriveAppSdkClient,
  initVoiceAppSdkClient,
  isVoiceAppSdkConfigured,
} from '@sdkwork/agents-pc-core/sdk';
import type { SdkworkAppbasePcAuthRuntimeSdkClient } from '@sdkwork/auth-runtime-pc-react';

import { configureAgentsWorkbenchPorts } from '../workbench/ports';
import { initializeAgentsPcIamRuntime } from './iamRuntime';

let initialized = false;

export function bootstrapAgentsSdk(): void {
  if (initialized) {
    return;
  }

  const sdkClients: SdkworkAppbasePcAuthRuntimeSdkClient[] = [
    initAgentsAppSdkClient(),
    initDriveAppSdkClient(),
  ];
  if (isVoiceAppSdkConfigured()) {
    sdkClients.push(initVoiceAppSdkClient());
  }
  initializeAgentsPcIamRuntime(sdkClients);
  configureAgentsWorkbenchPorts();
  initialized = true;
}
