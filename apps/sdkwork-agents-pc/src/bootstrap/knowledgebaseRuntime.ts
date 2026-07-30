import {
  configureKnowledgeSelectionAdapter,
  createKnowledgebaseSelectionAdapter,
} from '@sdkwork/agents-pc-agents/services';
import {
  initKnowledgebaseAppSdkClient,
  isKnowledgebaseAppSdkConfigured,
} from '@sdkwork/agents-pc-core/sdk/knowledgebaseAppSdkClient';

let initialized = false;

export function initializeAgentsKnowledgebaseRuntime(): void {
  if (initialized || !isKnowledgebaseAppSdkConfigured()) {
    return;
  }

  const knowledgebaseClient = initKnowledgebaseAppSdkClient();
  configureKnowledgeSelectionAdapter(createKnowledgebaseSelectionAdapter(knowledgebaseClient));
  initialized = true;
}
