import {
  configureKnowledgeSelectionAdapter,
  createKnowledgebaseSelectionAdapter,
} from '@sdkwork/agents-pc-agents/services';
import {
  initAgentsAppSdkClient,
  initDriveAppSdkClient,
  initKnowledgebaseAppSdkClient,
  initSkillsAppSdkClient,
  initVoiceAppSdkClient,
  isKnowledgebaseAppSdkConfigured,
  isSkillsAppSdkConfigured,
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
  if (isKnowledgebaseAppSdkConfigured()) {
    const knowledgebaseClient = initKnowledgebaseAppSdkClient();
    configureKnowledgeSelectionAdapter(createKnowledgebaseSelectionAdapter(knowledgebaseClient));
    sdkClients.push(knowledgebaseClient);
  }
  if (isSkillsAppSdkConfigured()) {
    sdkClients.push(initSkillsAppSdkClient());
  }
  if (isVoiceAppSdkConfigured()) {
    sdkClients.push(initVoiceAppSdkClient());
  }
  initializeAgentsPcIamRuntime(sdkClients);
  configureAgentsWorkbenchPorts();
  initialized = true;
}
