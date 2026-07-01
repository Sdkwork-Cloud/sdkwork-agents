import type { KnowledgeMarketCatalogItem, SdkworkKnowledgebaseAppClient } from "@sdkwork/agents-pc-core/sdk";

import type { KnowledgeSelectionAdapter } from "./knowledgeSelectionAdapter";
import type { KnowledgeBase } from "./KnowledgeSelectionService";

function mapMarketListingToKnowledgeBase(item: KnowledgeMarketCatalogItem): KnowledgeBase {
  return {
    id: `knowledge.market.${item.id}`,
    name: item.title,
    description: item.description,
    type: item.isSubscribed ? "team" : "all",
    documentCount: item.documentsCount,
    logo: item.icon,
    count: item.documentsCount,
  };
}

export function createKnowledgebaseSelectionAdapter(
  client: SdkworkKnowledgebaseAppClient,
): KnowledgeSelectionAdapter {
  return {
    async getBases(): Promise<KnowledgeBase[]> {
      const catalog = await client.knowledge.market.listings.list();
      return catalog.items.map(mapMarketListingToKnowledgeBase);
    },
  };
}
