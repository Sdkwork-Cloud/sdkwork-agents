import type {
  KnowledgeMarketCatalogItem,
  SdkworkKnowledgebaseAppClient,
} from "@sdkwork/agents-pc-core/sdk/knowledgebaseAppSdkClient";
import {
  DEFAULT_LIST_PAGE_SIZE,
  extractCursorPageInfo,
  extractListItems,
} from "@sdkwork/agents-pc-core/sdk/pagination";

import type { KnowledgeBasesPage, KnowledgeSelectionAdapter } from "./knowledgeSelectionAdapter";
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
    async getBasesPage(params?: { cursor?: string; pageSize?: number }): Promise<KnowledgeBasesPage> {
      const response = await client.knowledge.market.listings.list({
        cursor: params?.cursor,
        pageSize: params?.pageSize ?? DEFAULT_LIST_PAGE_SIZE,
      });
      const pageInfo = extractCursorPageInfo(response);
      const items = extractListItems(response)
        .map((item) => mapMarketListingToKnowledgeBase(item as KnowledgeMarketCatalogItem));
      return {
        items,
        hasMore: pageInfo.hasMore,
        nextCursor: pageInfo.nextPageToken,
      };
    },
  };
}
