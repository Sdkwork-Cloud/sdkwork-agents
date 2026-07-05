import {
  getVoiceAppSdkClient,
  isVoiceAppSdkConfigured,
} from "@sdkwork/agents-h5-core/sdk/voiceAppSdkClient";

import { DEFAULT_LIST_PAGE_SIZE } from "@sdkwork/agents-h5-core/sdk/pagination";

import { extractArray } from "./sdkEnvelope";

export interface VoiceConfig {
  id: string;
  name: string;
  description: string;
  categoryId: string;
  iconName?: string;
  color?: string;
  author?: string;
  users?: string;
  audioPreview?: string;
}

export interface VoiceCatalogPage {
  items: VoiceConfig[];
  page: number;
  hasMore: boolean;
}

function pickString(record: Record<string, unknown>, keys: string[]): string | undefined {
  for (const key of keys) {
    const value = record[key];
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return undefined;
}

function mapVoiceRecord(record: Record<string, unknown>, index: number): VoiceConfig {
  const id =
    pickString(record, ["voiceId", "voice_id", "assetId", "asset_id", "id"]) ??
    `voice.${index}`;
  return {
    id,
    name: pickString(record, ["displayName", "display_name", "name", "title"]) ?? id,
    description: pickString(record, ["description", "summary"]) ?? "Voice asset",
    categoryId: pickString(record, ["categoryId", "category_id", "category"]) ?? "market",
    author: pickString(record, ["author", "provider"]) ?? "sdkwork-voice",
    users: pickString(record, ["users", "usageCount"]) ?? "â€?,
    audioPreview: pickString(record, ["previewUrl", "preview_url", "audioPreview"]),
    iconName: "Mic",
    color: "bg-blue-500",
  };
}

function readHasMore(response: Record<string, unknown>): boolean {
  const pageInfo = response.pageInfo;
  if (pageInfo && typeof pageInfo === "object" && !Array.isArray(pageInfo)) {
    return Boolean((pageInfo as Record<string, unknown>).hasMore);
  }
  return false;
}

class VoiceService {
  private ensureVoiceSdk(): void {
    if (!isVoiceAppSdkConfigured()) {
      throw new Error("Voice catalog SDK is not configured for this deployment.");
    }
  }

  async listVoiceCatalogPage(page = 1, pageSize = DEFAULT_LIST_PAGE_SIZE): Promise<VoiceCatalogPage> {
    this.ensureVoiceSdk();
    const response = (await getVoiceAppSdkClient().voice.audioAssets.list({
      page,
      pageSize,
    })) as Record<string, unknown>;
    const items = extractArray(response)
      .map((item, index) =>
        item && typeof item === "object"
          ? mapVoiceRecord(item as Record<string, unknown>, index)
          : undefined,
      )
      .filter((item): item is VoiceConfig => Boolean(item));
    return {
      items,
      page,
      hasMore: readHasMore(response),
    };
  }

  async getMarketVoices(page = 1): Promise<VoiceCatalogPage> {
    const catalog = await this.listVoiceCatalogPage(page);
    return {
      ...catalog,
      items: catalog.items.filter((voice) => voice.categoryId !== "custom"),
    };
  }

  async getMyVoices(page = 1): Promise<VoiceCatalogPage> {
    const catalog = await this.listVoiceCatalogPage(page);
    return {
      ...catalog,
      items: catalog.items.filter((voice) => voice.categoryId === "custom"),
    };
  }
}

export const voiceService = new VoiceService();
