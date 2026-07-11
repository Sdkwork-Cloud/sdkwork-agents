import { Code2, Cpu, Globe } from "lucide-react";
import { createElement } from "react";

import {
  getAgentsAppSdkClientWithSession,
  type CodeEngineCatalog,
  type SdkworkAgentsAppClient,
} from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";
import {
  DEFAULT_LIST_PAGE_SIZE,
  extractOffsetPageInfo,
} from "@sdkwork/agents-h5-core/sdk/pagination";

import type { ToolItem } from "../components/SelectToolsModal";

import { extractArray, extractResourceRecord } from "./sdkEnvelope";

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

function mapModelRecord(record: Record<string, unknown>): ModelCatalogItem | undefined {
  const modelId = pickString(record, ["modelId", "model_id", "id"]);
  if (!modelId) {
    return undefined;
  }
  return {
    id: modelId,
    label: pickString(record, ["label", "name", "displayName", "display_name"]) ?? modelId,
    description: pickString(record, ["description", "summary"]) ?? "",
    providerId: pickString(record, ["providerId", "provider_id"]) ?? "",
    engineKey: pickString(record, ["engineKey", "engine_key"]) ?? "",
    bindingId: pickString(record, ["bindingId", "binding_id"]) ?? "",
    defaultForEngine: Boolean(record.defaultForEngine ?? record.default_for_engine),
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

/** Load model catalog from agents code-engine runtime (`GET /app/v3/api/ai/code_engines`). */
export async function loadRuntimeModelCatalog(
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ModelCatalogItem[]> {
  const response = await client.ai.agents.codeEngines.list();
  const catalog = extractResourceRecord(response) as unknown as CodeEngineCatalog | undefined;
  if (!catalog?.engines?.length) {
    return [];
  }
  return catalog.engines.flatMap((engine) =>
    (engine.models ?? [])
      .map((model) =>
        mapModelRecord({
          ...model,
          engineKey: engine.engineKey,
          bindingId: engine.bindingId,
        } as unknown as Record<string, unknown>),
      )
      .filter((item): item is ModelCatalogItem => Boolean(item)),
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
  const pageInfo = extractOffsetPageInfo(response);
  const items = extractArray(response)
    .map((record, index) =>
      record && typeof record === "object"
        ? mapMcpRecord(record as Record<string, unknown>, index)
        : undefined,
    )
    .filter((item): item is ToolItem => Boolean(item));
  return { items, page: pageInfo.page, hasMore: pageInfo.hasMore };
}

/** Official code-engine tools (small catalog; not paginated at HTTP layer). */
export async function loadCodeEngineToolItems(
  client: SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession(),
): Promise<ToolItem[]> {
  const codeEngines = await client.ai.agents.codeEngines.list();
  return extractItems(codeEngines)
    .map(mapCodeEngineRecord)
    .filter((item): item is ToolItem => Boolean(item));
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
