import {
  configureKnowledgeSelectionAdapter,
  createKnowledgebaseSelectionAdapter,
  agentChatService,
  agentService,
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
import { configureChatAgentPort } from '@sdkwork/chatbox-pc-core';

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
  configureChatAgentPort({
    getAgent: (agentId) => agentService.getAgent(agentId),
    createAgent: (agent) => agentService.createAgent(agent),
    updateAgent: (agentId, patch) => agentService.updateAgent(agentId, patch),
    resolveOrCreateSession: (agentId, title) => agentChatService.resolveOrCreateSession(agentId, title),
    sendMessage: (agentId, sessionId, content, model, media) =>
      agentChatService.sendMessage(agentId, sessionId, content, model, media),
  });
  initialized = true;
}
