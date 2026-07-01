import { Blocks, Briefcase, Network, Activity, GitBranch, KeySquare } from "lucide-react";
import { createElement } from "react";

import {
  getSkillsAppSdkClient,
  isSkillsAppSdkConfigured,
} from "@sdkwork/agents-pc-core/sdk/skillsAppSdkClient";

import type { SkillItem } from "../components/SelectSkillsModal";
import { extractArray } from "./sdkEnvelope";

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

/** Curated workflow presets when skills app SDK is not configured. */
export function loadSkillPresetCatalog(): SkillItem[] {
  return [
    {
      id: "skill.planning",
      name: "步骤规划 (ReAct)",
      description: "对复杂任务进行自动拆解并分步执行，提高推理准确性。",
      provider: "高级心智",
      category: "workflow",
      icon: createElement(Blocks, { size: 20, className: "text-cyan-500" }),
    },
    {
      id: "skill.reflection",
      name: "自我反思 (Reflection)",
      description: "在得出最终答案前，生成多个草案并选取最优解。",
      provider: "高级心智",
      category: "workflow",
      icon: createElement(Activity, { size: 20, className: "text-pink-500" }),
    },
    {
      id: "skill.multi-route",
      name: "多模型路由 (Routing)",
      description: "根据问题难度自动切换模型以降低成本。",
      provider: "成本优化",
      category: "workflow",
      icon: createElement(GitBranch, { size: 20, className: "text-violet-500" }),
    },
    {
      id: "skill.multi-agent",
      name: "多智能体协作 (Swarm)",
      description: "主模型可创建子 Agent 并行解决问题。",
      provider: "高级心智",
      category: "workflow",
      icon: createElement(Network, { size: 20, className: "text-blue-500" }),
    },
    {
      id: "skill.cot",
      name: "思维链 (CoT)",
      description: "强制要求模型输出完整的内部思考过程后再作答。",
      provider: "基础心智",
      category: "workflow",
      icon: createElement(KeySquare, { size: 20, className: "text-amber-500" }),
    },
    {
      id: "skill.domain-expert",
      name: "行业专家预设",
      description: "自动加载法律、医疗、金融等行业的默认系统级术语限制。",
      provider: "特定场景",
      category: "preset",
      icon: createElement(Briefcase, { size: 20, className: "text-rose-500" }),
    },
  ];
}

/** Load skills from sdkwork-skills app SDK when configured; otherwise preset catalog. */
export async function loadSkillCatalog(): Promise<SkillItem[]> {
  if (!isSkillsAppSdkConfigured()) {
    return loadSkillPresetCatalog();
  }

  try {
    const client = getSkillsAppSdkClient();
    const [skillsPage, packagesPage] = await Promise.all([
      client.skills.list({ page: 1, pageSize: 100 }),
      client.skills.skillPackages.list({ page: 1, pageSize: 100 }),
    ]);
    const merged = [...extractArray(skillsPage), ...extractArray(packagesPage)]
      .map((item, index) =>
        item && typeof item === "object"
          ? mapSkillRecord(item as Record<string, unknown>, index)
          : undefined,
      )
      .filter((item): item is SkillItem => Boolean(item));

    return merged.length > 0 ? merged : loadSkillPresetCatalog();
  } catch {
    return loadSkillPresetCatalog();
  }
}
