import {
  configureKnowledgeSelectionAdapter,
  createKnowledgebaseSelectionAdapter,
  agentChatService,
  agentProjectService,
  agentService,
} from '@sdkwork/agents-pc-agents/services';
import {
  initAgentsAppSdkClient,
  agentsDriveUploadService,
  initDriveAppSdkClient,
  initKnowledgebaseAppSdkClient,
  initSkillsAppSdkClient,
  initVoiceAppSdkClient,
  isKnowledgebaseAppSdkConfigured,
  isSkillsAppSdkConfigured,
  isVoiceAppSdkConfigured,
} from '@sdkwork/agents-pc-core/sdk';
import type { SdkworkAppbasePcAuthRuntimeSdkClient } from '@sdkwork/auth-runtime-pc-react';
import { configureChatAgentPort, configureProjectPort } from '@sdkwork/agents-pc-chat';

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
    resolveOrCreateSession: (agentId, sessionId, title) =>
      agentChatService.resolveOrCreateNamedSession(agentId, sessionId, title),
    listSessions: (agentId) => agentChatService.listSessionSummaries(agentId),
    updateSession: (agentId, sessionId, patch) =>
      agentChatService.updateSession(agentId, sessionId, patch),
    deleteSession: (agentId, sessionId) => agentChatService.deleteSession(agentId, sessionId),
    listSessionUserStates: (agentId, pinnedOnly) =>
      agentChatService.listSessionUserStates(agentId, pinnedOnly),
    updateSessionUserState: (agentId, sessionId, patch) =>
      agentChatService.updateSessionUserState(agentId, sessionId, patch),
    listMessageFeedback: (agentId, sessionId) =>
      agentChatService.listMessageFeedback(agentId, sessionId),
    updateMessageFeedback: (agentId, sessionId, messageId, patch) =>
      agentChatService.updateMessageFeedback(agentId, sessionId, messageId, patch),
    listMessages: (agentId, sessionId) => agentChatService.listMessages(agentId, sessionId),
    resolveMediaPreviewUrl: (driveUri) => agentsDriveUploadService.resolvePreviewUrl(driveUri),
    sendMessage: (agentId, sessionId, content, model, media) =>
      agentChatService.sendMessage(agentId, sessionId, content, model, media),
  });
  configureProjectPort({
    list: () => agentProjectService.list(),
    retrieve: (projectId) => agentProjectService.retrieve(projectId),
    create: (input) => agentProjectService.create(input),
    update: (projectId, patch) => agentProjectService.update(projectId, patch),
    delete: (projectId) => agentProjectService.delete(projectId),
    listCompositionSlots: (projectId) => agentProjectService.listCompositionSlots(projectId),
    getInstructions: (slots) => agentProjectService.getInstructions(slots),
    saveInstructions: (project, slots, content) =>
      agentProjectService.saveInstructions(project, slots, content),
    listMemorySpaces: () => agentProjectService.listMemorySpaces(),
    saveMemorySpace: (projectId, slots, memorySpaceId) =>
      agentProjectService.saveMemorySpace(projectId, slots, memorySpaceId),
  });
  initialized = true;
}
