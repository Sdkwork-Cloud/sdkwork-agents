import { Blocks } from "lucide-react";
import { createElement } from "react";

import {
  getSkillsAppSdkClient,
  isSkillsAppSdkConfigured,
} from "@sdkwork/agents-pc-core/sdk/skillsAppSdkClient";

import type { SkillItem } from "../components/SelectSkillsModal";
import { DEFAULT_LIST_PAGE_SIZE, extractOffsetPageInfo } from "@sdkwork/agents-pc-core/sdk/pagination";

import { extractArray } from "./sdkEnvelope";

export interface SkillCatalogPage {
  items: SkillItem[];
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

function mapSkillRecord(
  record: Record<string, unknown>,
  index: number,
  category: SkillItem["category"],
): SkillItem | undefined {
  const id =
    pickString(record, ["skillKey", "skill_key", "skillId", "skill_id", "id"]) ??
    `skill.${category}.${index}`;
  const name = pickString(record, ["displayName", "display_name", "name", "title"]) ?? id;
  const description =
    pickString(record, ["description", "summary"]) ?? "Skill package from sdkwork-skills";
  return {
    id: id.startsWith("skill.") ? id : `skill.${id}`,
    name,
    description,
    provider: pickString(record, ["provider", "author"]) ?? "sdkwork-skills",
    category,
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
  if (!isSkillsAppSdkConfigured()) {
    throw new Error("Skills catalog SDK is not configured for this deployment.");
  }

  const client = getSkillsAppSdkClient();
  const listParams = {
    page,
    pageSize,
    ...(q?.trim() ? { q: q.trim() } : {}),
  };
  const response =
    category === "preset"
      ? await client.skills.list(listParams)
      : await client.skills.skillPackages.list(listParams);
  const pageInfo = extractOffsetPageInfo(response);
  const items = extractArray(response)
    .map((item, index) =>
      item && typeof item === "object"
        ? mapSkillRecord(item as Record<string, unknown>, index, category)
        : undefined,
    )
    .filter((item): item is SkillItem => Boolean(item));

  return { items, page: pageInfo.page, hasMore: pageInfo.hasMore };
}
