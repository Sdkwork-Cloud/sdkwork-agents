import { Code2, Globe, Wrench } from "lucide-react";
import { createElement } from "react";

import {
  getAgentsAppSdkClientWithSession,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";

import type { ToolItem } from "../components/SelectToolsModal";

function asRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" ? (value as Record<string, unknown>) : undefined;
}

function pickString(record: Record<string, unknown> | undefined, keys: string[]): string | undefined {
  if (!record) {
    return undefined;
  }
  for (const key of keys) {
    const value = record[key];
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return undefined;
}

function mapMcpRecord(record: Record<string, unknown>, index: number): ToolItem | undefined {
  const id =
    pickString(record, ["targetRef", "target_ref", "serverId", "server_id", "id"]) ??
    `mcp.${index}`;
  const name =
    pickString(record, ["displayName", "display_name", "name", "title"]) ?? id;
  const description =
    pickString(record, ["description", "summary"]) ?? "MCP marketplace entry";
  return {
    id,
    name,
    description,
    provider: "MCP Marketplace",
    category: "mcp",
    icon: createElement(Globe, { size: 20, className: "text-gray-300" }),
  };
}

function mapCodeEngineRecord(record: Record<string, unknown>, index: number): ToolItem | undefined {
  const id = pickString(record, ["engineKey", "engine_key", "id", "key"]) ?? `engine.${index}`;
  const name = pickString(record, ["displayName", "display_name", "name", "label"]) ?? id;
  const description =
    pickString(record, ["description", "summary"]) ?? "Code engine runtime";
  return {
    id,
    name,
    description,
    provider: "Code Engine",
    category: "official",
    icon: createElement(Code2, { size: 20, className: "text-emerald-500" }),
  };
}

function extractItems(snapshot: unknown): Record<string, unknown>[] {
  const root = asRecord(snapshot);
  if (!root) {
    return [];
  }
  const data = asRecord(root.data) ?? root;
  for (const key of ["items", "engines", "servers", "catalog"]) {
    const value = data[key];
    if (Array.isArray(value)) {
      return value.filter((item): item is Record<string, unknown> => Boolean(asRecord(item)));
    }
  }
  return [];
}

export async function loadRuntimeToolCatalog(
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ToolItem[]> {
  const [codeEngines, mcpServers] = await Promise.all([
    client.ai.agents.codeEngines.list(),
    client.ai.agents.mcpServers.list(),
  ]);

  const engineItems = extractItems(codeEngines)
    .map(mapCodeEngineRecord)
    .filter((item): item is ToolItem => Boolean(item));
  const mcpItems = extractItems(mcpServers)
    .map(mapMcpRecord)
    .filter((item): item is ToolItem => Boolean(item));

  if (engineItems.length === 0 && mcpItems.length === 0) {
    return [];
  }

  return [
    ...engineItems,
    ...mcpItems,
    {
      id: "custom.api",
      name: "自建 API",
      description: "通过 composition slot 引用企业内部 API。",
      provider: "Custom",
      category: "custom",
      icon: createElement(Wrench, { size: 20, className: "text-emerald-400" }),
    },
  ];
}
