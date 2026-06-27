import { resolveKnowledgeSelectionAdapter } from "./knowledgeSelectionAdapter";

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

class KnowledgeSelectionService {
  async getBases(): Promise<KnowledgeBase[]> {
    const adapter = resolveKnowledgeSelectionAdapter();
    if (!adapter) {
      return [];
    }
    return adapter.getBases();
  }
}

export const knowledgeSelectionService = new KnowledgeSelectionService();
