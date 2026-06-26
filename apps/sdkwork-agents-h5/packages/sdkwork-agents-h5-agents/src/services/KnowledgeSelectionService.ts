import { getAgentsAppSdkClientWithSession } from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";

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

function asString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : undefined;
}

function normalizeKnowledgeBase(record: Record<string, unknown>): KnowledgeBase | null {
  const id = asString(record.knowledgeBaseId) ?? asString(record.id);
  const name = asString(record.name);
  if (!id || !name) {
    return null;
  }

  return {
    id,
    name,
    description: asString(record.description),
    type: (asString(record.visibility) as KnowledgeBase["type"]) ?? "all",
    updatedAt: asString(record.updatedAt) ?? asString(record.updated_at),
    documentCount:
      typeof record.documentCount === "number"
        ? record.documentCount
        : typeof record.document_count === "number"
          ? record.document_count
          : undefined,
    count:
      typeof record.count === "number"
        ? record.count
        : typeof record.documentCount === "number"
          ? record.documentCount
          : undefined,
    logo: asString(record.logo),
  };
}

class KnowledgeSelectionService {
  async getBases(): Promise<KnowledgeBase[]> {
    const adapter = resolveKnowledgeSelectionAdapter();
    if (adapter) {
      return adapter.getBases();
    }

    const response = await getAgentsAppSdkClientWithSession().ai.knowledgeBases.list({
      page: 1,
      pageSize: 100,
    });
    const items = (Array.isArray(response.data) ? response.data : []) as Record<string, unknown>[];
    return items
      .map((item) => normalizeKnowledgeBase(item))
      .filter((item): item is KnowledgeBase => item !== null);
  }
}

export const knowledgeSelectionService = new KnowledgeSelectionService();
