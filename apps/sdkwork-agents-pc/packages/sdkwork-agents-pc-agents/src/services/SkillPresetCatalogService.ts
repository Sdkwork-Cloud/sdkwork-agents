import { Blocks } from "lucide-react";
import { createElement } from "react";

import type {
  SkillPackageRecord,
  SkillRecord,
} from "@sdkwork/agents-pc-core/sdk/skillsAppSdkClient";

import type { SkillItem } from "../components/SelectSkillsModal";
import { DEFAULT_LIST_PAGE_SIZE, toOffsetPageInfo } from "@sdkwork/agents-pc-core/sdk/pagination";

export interface SkillCatalogPage {
  items: SkillItem[];
  page: number;
  hasMore: boolean;
}

function mapSkillRecord(record: SkillRecord): SkillItem {
  return {
    id: record.skillKey.startsWith("skill.") ? record.skillKey : `skill.${record.skillKey}`,
    name: record.name,
    description: record.description ?? record.summary ?? "Skill from sdkwork-skills",
    provider: "sdkwork-skills",
    category: "preset",
    icon: createElement(Blocks, { size: 20, className: "text-cyan-500" }),
  };
}

function mapSkillPackageRecord(record: SkillPackageRecord): SkillItem {
  return {
    id: record.skillKey.startsWith("skill.") ? record.skillKey : `skill.${record.skillKey}`,
    name: record.displayName,
    description: record.description ?? record.summary ?? "Skill package from sdkwork-skills",
    provider: "sdkwork-skills",
    category: "workflow",
    icon: createElement(Blocks, { size: 20, className: "text-cyan-500" }),
  };
}

/** Load one picker page for a single skills tab (`PAGINATION_SPEC.md` §8). */
export async function loadSkillCatalogPageByCategory(
  category: SkillItem["category"],
  page = 1,
  pageSize = DEFAULT_LIST_PAGE_SIZE,
  q?: string,
): Promise<SkillCatalogPage> {
  const {
    getSkillsAppSdkClient,
    isSkillsAppSdkConfigured,
  } = await import("@sdkwork/agents-pc-core/sdk/skillsAppSdkClient");
  if (!isSkillsAppSdkConfigured()) {
    throw new Error("Skills catalog SDK is not configured for this deployment.");
  }

  const client = getSkillsAppSdkClient();
  const listParams = {
    page,
    pageSize,
    ...(q?.trim() ? { q: q.trim() } : {}),
  };
  const response = category === "preset"
    ? await client.skills.marketplace.list(listParams)
    : await client.skills.skillPackages.list(listParams);
  const pageInfo = toOffsetPageInfo(response.pageInfo);
  const items = category === "preset"
    ? (response.items as SkillRecord[]).map(mapSkillRecord)
    : (response.items as SkillPackageRecord[]).map(mapSkillPackageRecord);

  return { items, page: pageInfo.page, hasMore: pageInfo.hasMore };
}
