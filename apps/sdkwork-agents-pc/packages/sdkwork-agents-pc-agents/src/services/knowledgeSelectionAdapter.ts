import type { KnowledgeBase } from "./KnowledgeSelectionService";

export interface KnowledgeBasesPage {
  items: KnowledgeBase[];
  hasMore: boolean;
  nextCursor?: string;
}

export interface KnowledgeSelectionAdapter {
  getBasesPage(params?: { cursor?: string; pageSize?: number }): Promise<KnowledgeBasesPage>;
}

let activeKnowledgeSelectionAdapter: KnowledgeSelectionAdapter | null = null;

export function configureKnowledgeSelectionAdapter(adapter: KnowledgeSelectionAdapter): void {
  activeKnowledgeSelectionAdapter = adapter;
}

export function resolveKnowledgeSelectionAdapter(): KnowledgeSelectionAdapter | null {
  return activeKnowledgeSelectionAdapter;
}
