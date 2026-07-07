import {
  resolveAppSdkOrganizationId,
  resolveAppSdkTenantId,
} from "@sdkwork/agents-h5-core/session";
import type { SdkworkAgentsAppClient } from "@sdkwork/agents-h5-core/sdk/agentsAppSdkClient";

import type { AgentConfig } from "./AgentService";
import { extractArray, extractResourceRecord, isRecord, syncAllOffsetPages } from "./sdkEnvelope";

type CompositionSlotKind = "memory" | "knowledge" | "skill" | "prompt" | "drive" | "tool";
type CompositionTargetModule = "memory" | "knowledgebase" | "skills" | "prompts" | "drive";

interface DesiredCompositionSlot {
  slotId: string;
  slotKind: CompositionSlotKind;
  targetModule: CompositionTargetModule;
  targetRef: string;
}

function slotIdForRef(targetRef: string): string {
  const normalized = targetRef.replace(/[^a-zA-Z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
  return `slot.${normalized || "resource"}`;
}

function buildDesiredCompositionSlots(config: AgentConfig): DesiredCompositionSlot[] {
  const slots: DesiredCompositionSlot[] = [];
  let priority = 0;

  for (const knowledgeId of config.knowledgeBaseIds ?? []) {
    const targetRef = knowledgeId.startsWith("kb.") ? knowledgeId : `kb.space.${knowledgeId}`;
    slots.push({
      slotId: slotIdForRef(targetRef),
      slotKind: "knowledge",
      targetModule: "knowledgebase",
      targetRef,
    });
    priority += 1;
  }

  for (const skillId of config.skillIds ?? []) {
    const targetRef = skillId.startsWith("skill.") ? skillId : `skill.${skillId}`;
    slots.push({
      slotId: slotIdForRef(targetRef),
      slotKind: "skill",
      targetModule: "skills",
      targetRef,
    });
    priority += 1;
  }

  for (const toolId of config.toolIds ?? []) {
    const targetRef = toolId.includes(".") ? toolId : `tool.${toolId}`;
    slots.push({
      slotId: slotIdForRef(targetRef),
      slotKind: "tool",
      targetModule: "drive",
      targetRef,
    });
    priority += 1;
  }

  return slots;
}

export async function syncAgentCompositionSlots(
  client: SdkworkAgentsAppClient,
  agentId: string,
  config: AgentConfig,
): Promise<void> {
  const tenantId = resolveAppSdkTenantId() ?? "100001";
  const organizationId = resolveAppSdkOrganizationId() ?? "0";
  const desired = buildDesiredCompositionSlots(config);
  const desiredIds = new Set(desired.map((slot) => slot.slotId));

  // Batch diff sync on agent save — `syncAllOffsetPages` is allowed per `PAGINATION_SPEC.md` §7.
  const existingItems = await syncAllOffsetPages<
    Record<string, unknown> & { slotId: string }
  >(
    (params) => client.ai.agents.compositionSlots.list(agentId, params),
    {
      mapItems: (response) =>
        extractArray(response)
          .map((item) => extractResourceRecord(item))
          .filter(
            (item): item is Record<string, unknown> & { slotId: string } =>
              isRecord(item) && typeof item.slotId === "string",
          ),
    },
  );

  for (const item of existingItems) {
    if (!desiredIds.has(item.slotId)) {
      await client.ai.agents.compositionSlots.delete(agentId, item.slotId);
    }
  }

  const existingById = new Map(existingItems.map((item) => [item.slotId, item]));
  for (const [index, slot] of desired.entries()) {
    const current = existingById.get(slot.slotId);
    if (current) {
      continue;
    }

    await client.ai.agents.compositionSlots.create(agentId, {
      data: {
        tenantId,
        organizationId,
        slotId: slot.slotId,
        slotKind: slot.slotKind,
        targetModule: slot.targetModule,
        targetRef: slot.targetRef,
        priority: String(index + 1),
        enabled: true,
        policyJson: "{}",
      },
      requestedAt: new Date().toISOString(),
    });
  }
}
