import {
  getVoiceAppSdkClient,
  isVoiceAppSdkConfigured,
} from "@sdkwork/agents-h5-core/sdk/voiceAppSdkClient";

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

const FALLBACK_MARKET_VOICES: VoiceConfig[] = [
  {
    id: "voice-1",
    name: "甜美女生-小悠",
    description: "适合有声书、电台、温馨风格阅读。",
    categoryId: "reading",
    iconName: "Mic",
    color: "bg-pink-500",
    author: "Sdkwork Voice",
    users: "12K",
  },
  {
    id: "voice-2",
    name: "沉稳男声-老赵",
    description: "适合新闻播报、商业解说或历史纪实。",
    categoryId: "news",
    iconName: "Radio",
    color: "bg-indigo-500",
    author: "Official",
    users: "8.5K",
  },
];

const FALLBACK_MY_VOICES: VoiceConfig[] = [
  {
    id: "voice-my-1",
    name: "自定义克隆声",
    description: "基于上传样本训练的声音。",
    categoryId: "custom",
    iconName: "User",
    color: "bg-purple-500",
    author: "我",
    users: "1",
  },
];

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

async function loadVoicesFromSdk(): Promise<VoiceConfig[] | null> {
  if (!isVoiceAppSdkConfigured()) {
    return null;
  }
  try {
    const response = await getVoiceAppSdkClient().voice.audioAssets.list({
      page: 1,
      pageSize: 100,
    });
    const items = extractArray(response)
      .map((item, index) =>
        item && typeof item === "object"
          ? mapVoiceRecord(item as Record<string, unknown>, index)
          : undefined,
      )
      .filter((item): item is VoiceConfig => Boolean(item));
    return items.length > 0 ? items : null;
  } catch {
    return null;
  }
}

class VoiceService {
  async getMarketVoices(): Promise<VoiceConfig[]> {
    const fromSdk = await loadVoicesFromSdk();
    if (fromSdk) {
      return fromSdk.filter((voice) => voice.categoryId !== "custom");
    }
    return FALLBACK_MARKET_VOICES;
  }

  async getMyVoices(): Promise<VoiceConfig[]> {
    const fromSdk = await loadVoicesFromSdk();
    if (fromSdk) {
      return fromSdk.filter((voice) => voice.categoryId === "custom");
    }
    return FALLBACK_MY_VOICES;
  }
}

export const voiceService = new VoiceService();
