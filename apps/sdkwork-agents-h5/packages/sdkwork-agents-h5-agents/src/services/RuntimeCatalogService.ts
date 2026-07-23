import { Code2, Cpu, Globe } from "lucide-react";
import { createElement } from "react";

import {
  getAgentsAppSdkClientWithSession,
  type CodeEngineCatalogEngine,
  type CodeEngineModelCatalogEntry,
  type McpServerMarketplaceRecord,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";
import {
  DEFAULT_LIST_PAGE_SIZE,
  toOffsetPageInfo,
} from "@sdkwork/agents-h5-core/sdk/pagination";

import type { ToolItem } from "../components/SelectToolsModal";

export interface ModelCatalogItem {
  id: string;
  label: string;
  description: string;
  providerId: string;
  engineKey: string;
  bindingId: string;
  defaultForEngine: boolean;
}

export interface McpCatalogPage {
  items: ToolItem[];
  page: number;
  hasMore: boolean;
}

function mapMcpRecord(record: McpServerMarketplaceRecord): ToolItem {
  return {
    id: record.targetRef,
    name: record.serverId,
    description: `MCP server ${record.serverId}`,
    provider: "MCP Marketplace",
    category: "mcp",
    icon: createElement(Globe, { size: 20, className: "text-gray-300" }),
  };
}

function mapCodeEngineRecord(record: CodeEngineCatalogEngine): ToolItem {
  return {
    id: record.engineKey,
    name: engineKeyToVendorLabel(record.engineKey),
    description: "Code engine runtime",
    provider: "Code Engine",
    category: "official",
    icon: createElement(Code2, { size: 20, className: "text-emerald-500" }),
  };
}

function mapModelRecord(record: CodeEngineModelCatalogEntry): ModelCatalogItem {
  return {
    id: record.modelId,
    label: record.label,
    description: record.description,
    providerId: record.providerId,
    engineKey: record.engineKey,
    bindingId: record.bindingId,
    defaultForEngine: record.defaultForEngine,
  };
}

/** Load model catalog from agents code-engine runtime (`GET /app/v3/api/ai/code_engines`). */
export async function loadRuntimeModelCatalog(
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ModelCatalogItem[]> {
  const catalog = await client.ai.agents.codeEngines.list();
  if (!catalog.engines.length) {
    return [];
  }
  return catalog.engines.flatMap((engine) =>
    engine.models.map(mapModelRecord),
  );
}

/** One MCP marketplace page for interactive pickers (`PAGINATION_SPEC.md` §8). */
export async function loadMcpCatalogPage(
  page = 1,
  pageSize = DEFAULT_LIST_PAGE_SIZE,
  q?: string,
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<McpCatalogPage> {
  const response = await client.ai.agents.mcpServers.list({
    page,
    pageSize,
    ...(q?.trim() ? { q: q.trim() } : {}),
  });
  const pageInfo = toOffsetPageInfo(response.pageInfo);
  const items = response.items.map(mapMcpRecord);
  return { items, page: pageInfo.page, hasMore: pageInfo.hasMore };
}

/** Official code-engine tools (small catalog; not paginated at HTTP layer). */
export async function loadCodeEngineToolItems(
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ToolItem[]> {
  const codeEngines = await client.ai.agents.codeEngines.list();
  return codeEngines.engines.map(mapCodeEngineRecord);
}

export function engineKeyToVendorLabel(engineKey: string): string {
  const labels: Record<string, string> = {
    codex: "OpenAI Codex",
    "claude-code": "Anthropic",
    gemini: "Google",
    opencode: "OpenCode",
    openclaw: "OpenClaw",
    hermes: "Hermes",
  };
  return labels[engineKey] ?? engineKey;
}

export function modelCatalogVendorIcon(_engineKey: string): React.ReactNode {
  return createElement(Cpu, { size: 14 });
}
