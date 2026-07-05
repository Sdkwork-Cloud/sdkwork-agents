import { Blocks, Briefcase, Network, Activity, GitBranch, KeySquare } from "lucide-react";
import { createElement } from "react";

import {
  getSkillsAppSdkClient,
  isSkillsAppSdkConfigured,
} from "@sdkwork/agents-h5-core/sdk/skillsAppSdkClient";

import type { SkillItem } from "../components/SelectSkillsModal";
import { DEFAULT_LIST_PAGE_SIZE } from "@sdkwork/agents-h5-core/sdk/pagination";

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

function mapSkillRecord(record: Record<string, unknown>, index: number): SkillItem | undefined {
  const id =
    pickString(record, ["skillKey", "skill_key", "skillId", "skill_id", "id"]) ??
    `skill.preset.${index}`;
  const name = pickString(record, ["displayName", "display_name", "name", "title"]) ?? id;
  const description =
    pickString(record, ["description", "summary"]) ?? "Skill package from sdkwork-skills";
  const categoryRaw = pickString(record, ["category", "categoryId", "category_id"]);
  const category: SkillItem["category"] =
    categoryRaw === "preset" || categoryRaw === "workflow" ? categoryRaw : "workflow";
  return {
    id: id.startsWith("skill.") ? id : `skill.${id}`,
    name,
    description,
    provider: pickString(record, ["provider", "author"]) ?? "sdkwork-skills",
    category,
    icon: createElement(Blocks, { size: 20, className: "text-cyan-500" }),
  };
}

function readHasMore(response: Record<string, unknown>): boolean {
  const pageInfo = response.pageInfo;
  if (pageInfo && typeof pageInfo === "object" && !Array.isArray(pageInfo)) {
    return Boolean((pageInfo as Record<string, unknown>).hasMore);
  }
  return false;
}

/** Load one interactive picker page from sdkwork-skills (`PAGINATION_SPEC.md` §8). */
export async function loadSkillCatalogPage(page = 1): Promise<SkillCatalogPage> {
  if (!isSkillsAppSdkConfigured()) {
    throw new Error("Skills catalog SDK is not configured for this deployment.");
  }

  const client = getSkillsAppSdkClient();
  const [skillsResponse, packagesResponse] = await Promise.all([
    client.skills.list({ page, pageSize: DEFAULT_LIST_PAGE_SIZE }),
    client.skills.skillPackages.list({ page, pageSize: DEFAULT_LIST_PAGE_SIZE }),
  ]);

  const skills = extractArray(skillsResponse as Record<string, unknown>)
    .map((item, index) =>
      item && typeof item === "object"
        ? mapSkillRecord(item as Record<string, unknown>, index)
        : undefined,
    )
    .filter((item): item is SkillItem => Boolean(item));
  const packages = extractArray(packagesResponse as Record<string, unknown>)
    .map((item, index) =>
      item && typeof item === "object"
        ? mapSkillRecord(item as Record<string, unknown>, skills.length + index)
        : undefined,
    )
    .filter((item): item is SkillItem => Boolean(item));

  const items = [...skills, ...packages];
  const hasMore =
    readHasMore(skillsResponse as Record<string, unknown>) ||
    readHasMore(packagesResponse as Record<string, unknown>);

  return { items, page, hasMore };
}

/** @deprecated Use `loadSkillCatalogPage` for paginated pickers. */
export async function loadSkillCatalog(): Promise<SkillItem[]> {
  const page = await loadSkillCatalogPage(1);
  return page.items;
}
