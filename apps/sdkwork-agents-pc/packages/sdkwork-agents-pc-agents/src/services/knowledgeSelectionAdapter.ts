import type { KnowledgeBase } from "./KnowledgeSelectionService";

export interface KnowledgeSelectionAdapter {
  getBases(): Promise<KnowledgeBase[]>;
}

let activeKnowledgeSelectionAdapter: KnowledgeSelectionAdapter | null = null;

export function configureKnowledgeSelectionAdapter(adapter: KnowledgeSelectionAdapter): void {
  activeKnowledgeSelectionAdapter = adapter;
}

export function resolveKnowledgeSelectionAdapter(): KnowledgeSelectionAdapter | null {
  return activeKnowledgeSelectionAdapter;
}
