import {
  configureKnowledgeSelectionAdapter,
  createKnowledgebaseSelectionAdapter,
} from "@sdkwork/agents-pc-agents";
import {
  initKnowledgebaseAppSdkClient,
  isKnowledgebaseAppSdkConfigured,
} from "@sdkwork/agents-pc-core/sdk/knowledgebaseAppSdkClient";

export function bootstrapKnowledgeSelection(): void {
  if (!isKnowledgebaseAppSdkConfigured()) {
    return;
  }

  const client = initKnowledgebaseAppSdkClient();
  configureKnowledgeSelectionAdapter(createKnowledgebaseSelectionAdapter(client));
}
