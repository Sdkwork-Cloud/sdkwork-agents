import {
  configureKnowledgeSelectionAdapter,
  createKnowledgebaseSelectionAdapter,
} from "@sdkwork/agents-h5-agents";
import {
  initKnowledgebaseAppSdkClient,
  isKnowledgebaseAppSdkConfigured,
} from "@sdkwork/agents-h5-core/sdk/knowledgebaseAppSdkClient";

export function bootstrapKnowledgeSelection(): void {
  if (!isKnowledgebaseAppSdkConfigured()) {
    return;
  }

  const client = initKnowledgebaseAppSdkClient();
  configureKnowledgeSelectionAdapter(createKnowledgebaseSelectionAdapter(client));
}
