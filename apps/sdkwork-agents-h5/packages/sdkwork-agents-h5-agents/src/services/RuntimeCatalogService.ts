import { Code2, Cpu, Globe } from "lucide-react";
import { createElement } from "react";

import {
  getAgentsAppSdkClientWithSession,
  type AgentEngineCatalogEngine,
  type AgentEngineModelCatalogEntry,
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

function mapAgentEngineRecord(record: AgentEngineCatalogEngine): ToolItem {
  return {
    id: record.engineKey,
    name: engineKeyToVendorLabel(record.engineKey),
    description: "Agent engine runtime",
    provider: "Agent Engine",
    category: "official",
    icon: createElement(Code2, { size: 20, className: "text-emerald-500" }),
  };
}

function mapModelRecord(record: AgentEngineModelCatalogEntry): ModelCatalogItem {
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

/** Load model catalog from agents agent-engine runtime (`GET /app/v3/api/ai/agent_engines`). */
export async function loadRuntimeModelCatalog(
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ModelCatalogItem[]> {
  const catalog = await client.ai.agents.agentEngines.list();
  if (!catalog.engines.length) {
    return [];
  }
  return catalog.engines.flatMap((engine) =>
    engine.models.map(mapModelRecord),
  );
}

/**
 * Resolve the runtime model used for one chat turn from the agent-engine
 * catalog. Prefers the exact `modelId`; when absent (or not present in the
 * catalog, e.g. a stale hard-coded default), falls back to the engine default
 * model (`defaultForEngine`), then to the first catalog entry.
 *
 * The resolved entry carries the canonical binding identity (`bindingId`,
 * `providerId`, `engineKey`) required to bind a session for managed turns.
 */
export async function resolveChatRuntimeModel(
  modelId?: string,
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ModelCatalogItem> {
  const catalog = await loadRuntimeModelCatalog(client);
  if (!catalog.length) {
    throw new Error("Agent engine runtime catalog is unavailable.");
  }
  if (modelId) {
    const exact = catalog.find((item) => item.id === modelId);
    if (exact) {
      return exact;
    }
  }
  const engineDefault = catalog.find((item) => item.defaultForEngine);
  return engineDefault ?? catalog[0];
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

/** Official agent-engine tools (small catalog; not paginated at HTTP layer). */
export async function loadAgentEngineToolItems(
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ToolItem[]> {
  const agentEngines = await client.ai.agents.agentEngines.list();
  return agentEngines.engines.map(mapAgentEngineRecord);
}

export function engineKeyToVendorLabel(engineKey: string): string {
  const labels: Record<string, string> = {
    codex: "OpenAI Codex",
    "claude-code": "Anthropic",
    gemini: "Google",
    opencode: "OpenCode",
    openclaw: "OpenClaw",
    hermes: "Hermes",
    "mimo-code": "MiMo Code",
    rig: "Rig",
  };
  return labels[engineKey] ?? engineKey;
}

export function modelCatalogVendorIcon(_engineKey: string): React.ReactNode {
  return createElement(Cpu, { size: 14 });
}
