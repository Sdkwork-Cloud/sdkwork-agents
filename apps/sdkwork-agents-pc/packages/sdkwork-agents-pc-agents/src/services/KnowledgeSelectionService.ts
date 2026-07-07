import {
  DEFAULT_LIST_PAGE_SIZE,
  type CursorPageInfo,
} from "@sdkwork/agents-pc-core/sdk/pagination";

import {
  resolveKnowledgeSelectionAdapter,
  type KnowledgeBasesPage,
} from "./knowledgeSelectionAdapter";

export interface KnowledgeBase {
  id: string;
  name: string;
  description?: string;
  type?: "personal" | "team" | "all";
  updatedAt?: string;
  documentCount?: number;
  logo?: string;
  count?: number;
}

export type { KnowledgeBasesPage, CursorPageInfo };

class KnowledgeSelectionService {
  async getBasesPage(
    cursor?: string,
    pageSize = DEFAULT_LIST_PAGE_SIZE,
  ): Promise<KnowledgeBasesPage> {
    const adapter = resolveKnowledgeSelectionAdapter();
    if (!adapter) {
      return { items: [], hasMore: false };
    }
    return adapter.getBasesPage({ cursor, pageSize });
  }
}

export const knowledgeSelectionService = new KnowledgeSelectionService();
