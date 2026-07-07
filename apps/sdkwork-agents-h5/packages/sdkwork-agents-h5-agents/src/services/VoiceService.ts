import {
  getVoiceAppSdkClient,
  isVoiceAppSdkConfigured,
} from "@sdkwork/agents-h5-core/sdk/voiceAppSdkClient";
import {
  DEFAULT_LIST_PAGE_SIZE,
  extractOffsetPageInfo,
} from "@sdkwork/agents-h5-core/sdk/pagination";

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
    users: pickString(record, ["users", "usageCount"]) ?? "—",
    audioPreview: pickString(record, ["previewUrl", "preview_url", "audioPreview"]),
    iconName: "Mic",
    color: "bg-blue-500",
  };
}

class VoiceService {
  private ensureVoiceSdk(): void {
    if (!isVoiceAppSdkConfigured()) {
      throw new Error("Voice catalog SDK is not configured for this deployment.");
    }
  }

  /** One interactive picker page (`PAGINATION_SPEC.md` §8). */
  async listVoiceCatalogPage(
    page = 1,
    pageSize = DEFAULT_LIST_PAGE_SIZE,
    categoryId?: string,
  ): Promise<VoiceCatalogPage> {
    this.ensureVoiceSdk();
    const response = await getVoiceAppSdkClient().voice.audioAssets.list({
      page,
      pageSize,
      ...(categoryId ? { categoryId } : {}),
    });
    const pageInfo = extractOffsetPageInfo(response);
    const items = extractArray(response)
      .map((item, index) =>
        item && typeof item === "object"
          ? mapVoiceRecord(item as Record<string, unknown>, index)
          : undefined,
      )
      .filter((item): item is VoiceConfig => Boolean(item));
    return {
      items,
      page: pageInfo.page,
      hasMore: pageInfo.hasMore,
    };
  }

  async getMarketVoices(page = 1, pageSize = DEFAULT_LIST_PAGE_SIZE): Promise<VoiceCatalogPage> {
    const catalog = await this.listVoiceCatalogPage(page, pageSize, "market");
    return {
      ...catalog,
      items: catalog.items.filter((voice) => voice.categoryId !== "custom"),
    };
  }

  async getMyVoices(page = 1, pageSize = DEFAULT_LIST_PAGE_SIZE): Promise<VoiceCatalogPage> {
    return this.listVoiceCatalogPage(page, pageSize, "custom");
  }
}

export const voiceService = new VoiceService();
