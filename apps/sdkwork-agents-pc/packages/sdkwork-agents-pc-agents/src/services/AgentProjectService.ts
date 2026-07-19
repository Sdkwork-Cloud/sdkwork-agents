import {
  getAgentsAppSdkClientWithSession,
  type SdkworkAgentsAppClient,
} from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';
import { getMemoryAppSdkClientWithSession } from '@sdkwork/agents-pc-core/sdk/memoryAppSdkClient';
import { getPromptsAppSdkClientWithSession } from '@sdkwork/agents-pc-core/sdk/promptsAppSdkClient';

import { extractArray, extractResourceRecord, isRecord } from './sdkEnvelope';

export type ProjectVisibility = 'private' | 'organization' | 'shared';
export type ProjectStatus = 'active' | 'archived' | 'deleted';
export type ProjectDriveAccessMode = 'disabled' | 'owner_library' | 'explicit_resources';

export interface AgentProject {
  id: string;
  projectId: string;
  name: string;
  description?: string;
  visibility: ProjectVisibility;
  status: ProjectStatus;
  driveAccessMode: ProjectDriveAccessMode;
  defaultAgentId?: string;
  defaultModelId?: string;
  version: string;
  updatedAt: string;
}

export interface CreateAgentProjectInput {
  name: string;
  description?: string;
  visibility?: ProjectVisibility;
  driveAccessMode?: ProjectDriveAccessMode;
  defaultAgentId?: string;
  defaultModelId?: string;
}

export type ProjectCompositionSlotKind =
  | 'prompt'
  | 'memory'
  | 'knowledge'
  | 'skill'
  | 'mcp'
  | 'drive'
  | 'tool';
export type ProjectCompositionTargetModule =
  | 'prompts'
  | 'memory'
  | 'knowledgebase'
  | 'skills'
  | 'mcp'
  | 'drive'
  | 'tools';

export interface AgentProjectCompositionSlot {
  id: string;
  projectId: string;
  slotId: string;
  slotKind: ProjectCompositionSlotKind;
  targetModule: ProjectCompositionTargetModule;
  targetRef: string;
  targetVersionRef?: string;
  priority: number;
  enabled: boolean;
  policyJson: string;
  version: string;
  updatedAt: string;
}

export interface AgentMemorySpaceOption {
  spaceId: string;
  displayName: string;
}

export interface ProjectCompositionSlotInput {
  slotId: string;
  slotKind: ProjectCompositionSlotKind;
  targetModule: ProjectCompositionTargetModule;
  targetRef: string;
  targetVersionRef?: string;
  priority?: number;
  enabled?: boolean;
  policyJson?: string;
}

function optionalString(record: Record<string, unknown>, key: string): string | undefined {
  const value = record[key];
  return typeof value === 'string' && value.trim() ? value : undefined;
}

function projectFromRecord(record: Record<string, unknown>): AgentProject {
  const projectId = optionalString(record, 'projectId');
  const name = optionalString(record, 'name');
  if (!projectId || !name) {
    throw new Error('Project response is missing projectId or name.');
  }
  return {
    id: optionalString(record, 'id') ?? projectId,
    projectId,
    name,
    description: optionalString(record, 'description'),
    visibility: (optionalString(record, 'visibility') as ProjectVisibility) ?? 'private',
    status: (optionalString(record, 'status') as ProjectStatus) ?? 'active',
    driveAccessMode:
      (optionalString(record, 'driveAccessMode') as ProjectDriveAccessMode) ?? 'disabled',
    defaultAgentId: optionalString(record, 'defaultAgentId'),
    defaultModelId: optionalString(record, 'defaultModelId'),
    version: optionalString(record, 'version') ?? '0',
    updatedAt: optionalString(record, 'updatedAt') ?? new Date(0).toISOString(),
  };
}

function numberValue(record: Record<string, unknown>, key: string, fallback = 0): number {
  const value = record[key];
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function booleanValue(record: Record<string, unknown>, key: string, fallback = false): boolean {
  const value = record[key];
  return typeof value === 'boolean' ? value : fallback;
}

function projectCompositionSlotFromRecord(
  record: Record<string, unknown>,
): AgentProjectCompositionSlot {
  const projectId = optionalString(record, 'projectId');
  const slotId = optionalString(record, 'slotId');
  const targetRef = optionalString(record, 'targetRef');
  if (!projectId || !slotId || !targetRef) {
    throw new Error('Project composition slot response is incomplete.');
  }
  return {
    id: optionalString(record, 'id') ?? slotId,
    projectId,
    slotId,
    slotKind:
      (optionalString(record, 'slotKind') as ProjectCompositionSlotKind) ?? 'prompt',
    targetModule:
      (optionalString(record, 'targetModule') as ProjectCompositionTargetModule) ?? 'prompts',
    targetRef,
    targetVersionRef: optionalString(record, 'targetVersionRef'),
    priority: numberValue(record, 'priority'),
    enabled: booleanValue(record, 'enabled', true),
    policyJson: optionalString(record, 'policyJson') ?? '{}',
    version: optionalString(record, 'version') ?? '0',
    updatedAt: optionalString(record, 'updatedAt') ?? new Date(0).toISOString(),
  };
}

function memorySpaceFromRecord(record: Record<string, unknown>): AgentMemorySpaceOption | null {
  const spaceId = optionalString(record, 'spaceId');
  if (!spaceId) return null;
  return {
    spaceId,
    displayName: optionalString(record, 'displayName') ?? spaceId,
  };
}

const PROJECT_INSTRUCTIONS_SLOT_ID = 'slot.project.instructions';
const PROJECT_MEMORY_SLOT_ID = 'slot.project.memory';

export class AgentProjectService {
  constructor(
    private readonly getClient: () => SdkworkAgentsAppClient = getAgentsAppSdkClientWithSession,
  ) {}

  async list(page = 1, pageSize = 50): Promise<AgentProject[]> {
    const response = await this.getClient().ai.agents.projects.list({ page, pageSize });
    return extractArray(response)
      .filter(isRecord)
      .map(projectFromRecord);
  }

  async retrieve(projectId: string): Promise<AgentProject> {
    return projectFromRecord(
      extractResourceRecord(await this.getClient().ai.agents.projects.retrieve(projectId)),
    );
  }

  async create(input: CreateAgentProjectInput): Promise<AgentProject> {
    return projectFromRecord(
      extractResourceRecord(await this.getClient().ai.agents.projects.create(input)),
    );
  }

  async update(
    projectId: string,
    patch: Partial<CreateAgentProjectInput> & { expectedVersion: string },
  ): Promise<AgentProject> {
    return projectFromRecord(
      extractResourceRecord(await this.getClient().ai.agents.projects.update(projectId, patch)),
    );
  }

  async archive(projectId: string, expectedVersion: string): Promise<AgentProject> {
    return projectFromRecord(
      extractResourceRecord(
        await this.getClient().ai.agents.projects.archive(projectId, { expectedVersion }),
      ),
    );
  }

  async delete(projectId: string): Promise<void> {
    await this.getClient().ai.agents.projects.delete(projectId);
  }

  async listCompositionSlots(projectId: string): Promise<AgentProjectCompositionSlot[]> {
    const response = await this.getClient().ai.agents.projectCompositionSlots.list(projectId, {
      page: 1,
      pageSize: 200,
    });
    return extractArray(response)
      .filter(isRecord)
      .map(projectCompositionSlotFromRecord);
  }

  async createCompositionSlot(
    projectId: string,
    input: ProjectCompositionSlotInput,
  ): Promise<AgentProjectCompositionSlot> {
    return projectCompositionSlotFromRecord(
      extractResourceRecord(
        await this.getClient().ai.agents.projectCompositionSlots.create(projectId, input),
      ),
    );
  }

  async updateCompositionSlot(
    projectId: string,
    slot: AgentProjectCompositionSlot,
    patch: Partial<Omit<ProjectCompositionSlotInput, 'slotId'>> & {
      clearTargetVersionRef?: boolean;
    },
  ): Promise<AgentProjectCompositionSlot> {
    return projectCompositionSlotFromRecord(
      extractResourceRecord(
        await this.getClient().ai.agents.projectCompositionSlots.update(
          projectId,
          slot.slotId,
          { ...patch, expectedVersion: slot.version },
        ),
      ),
    );
  }

  async deleteCompositionSlot(
    projectId: string,
    slot: AgentProjectCompositionSlot,
  ): Promise<void> {
    await this.getClient().ai.agents.projectCompositionSlots.delete(projectId, slot.slotId, {
      expectedVersion: slot.version,
    });
  }

  async getInstructions(
    slots: AgentProjectCompositionSlot[],
  ): Promise<string> {
    const slot = slots.find(
      (candidate) => candidate.slotId === PROJECT_INSTRUCTIONS_SLOT_ID && candidate.enabled,
    );
    if (!slot) return '';
    const response = await getPromptsAppSdkClientWithSession()
      .prompts.templateVersions.list(slot.targetRef);
    const versions = extractArray(response).filter(isRecord);
    const selected = slot.targetVersionRef
      ? versions.find((record) => optionalString(record, 'id') === slot.targetVersionRef)
      : versions[0];
    return selected ? optionalString(selected, 'content') ?? '' : '';
  }

  async saveInstructions(
    project: Pick<AgentProject, 'projectId' | 'name'>,
    slots: AgentProjectCompositionSlot[],
    content: string,
  ): Promise<AgentProjectCompositionSlot[]> {
    const existingSlot = slots.find(
      (candidate) => candidate.slotId === PROJECT_INSTRUCTIONS_SLOT_ID,
    );
    const normalizedContent = content.trim();
    if (!normalizedContent) {
      if (existingSlot) await this.deleteCompositionSlot(project.projectId, existingSlot);
      return slots.filter((candidate) => candidate.slotId !== PROJECT_INSTRUCTIONS_SLOT_ID);
    }

    const prompts = getPromptsAppSdkClientWithSession().prompts;
    const templateKey = `agents.${project.projectId}.instructions`;
    const templates = extractArray(await prompts.templates.list({ limit: 200 }))
      .filter(isRecord);
    let template = templates.find((record) => optionalString(record, 'key') === templateKey);
    if (!template) {
      template = promptsTemplateRecord(await prompts.templates.create({
        key: templateKey,
        name: `${project.name} instructions`,
        description: `Managed instructions for ${project.projectId}`,
        tags: ['sdkwork-agents', 'project-instructions'],
      }));
    }
    const templateId = optionalString(template, 'id');
    if (!templateId) throw new Error('Prompt template creation did not return an id.');

    const versionLabel = `project-slot-v${existingSlot ? Number(existingSlot.version) + 1 : 0}`;
    const versions = extractArray(await prompts.templateVersions.list(templateId)).filter(isRecord);
    let version = versions.find(
      (record) => optionalString(record, 'version_label') === versionLabel,
    );
    if (!version) {
      version = promptsTemplateRecord(await prompts.templateVersions.create(templateId, {
        version_label: versionLabel,
        content: normalizedContent,
        variables: [],
      }));
    }
    const versionId = optionalString(version, 'id');
    if (!versionId) throw new Error('Prompt template version creation did not return an id.');

    const savedSlot = existingSlot
      ? await this.updateCompositionSlot(project.projectId, existingSlot, {
          slotKind: 'prompt',
          targetModule: 'prompts',
          targetRef: templateId,
          targetVersionRef: versionId,
          enabled: true,
        })
      : await this.createCompositionSlot(project.projectId, {
          slotId: PROJECT_INSTRUCTIONS_SLOT_ID,
          slotKind: 'prompt',
          targetModule: 'prompts',
          targetRef: templateId,
          targetVersionRef: versionId,
          priority: -1000,
          enabled: true,
          policyJson: '{"role":"system"}',
        });
    return [
      ...slots.filter((candidate) => candidate.slotId !== PROJECT_INSTRUCTIONS_SLOT_ID),
      savedSlot,
    ];
  }

  async listMemorySpaces(): Promise<AgentMemorySpaceOption[]> {
    const response = await getMemoryAppSdkClientWithSession().memory.spaces.list({
      pageSize: 200,
    });
    return extractArray(response)
      .filter(isRecord)
      .map(memorySpaceFromRecord)
      .filter((item): item is AgentMemorySpaceOption => item !== null);
  }

  async saveMemorySpace(
    projectId: string,
    slots: AgentProjectCompositionSlot[],
    memorySpaceId?: string,
  ): Promise<AgentProjectCompositionSlot[]> {
    const existingSlot = slots.find((candidate) => candidate.slotId === PROJECT_MEMORY_SLOT_ID);
    if (!memorySpaceId) {
      if (existingSlot) await this.deleteCompositionSlot(projectId, existingSlot);
      return slots.filter((candidate) => candidate.slotId !== PROJECT_MEMORY_SLOT_ID);
    }
    const savedSlot = existingSlot
      ? await this.updateCompositionSlot(projectId, existingSlot, {
          slotKind: 'memory',
          targetModule: 'memory',
          targetRef: memorySpaceId,
          clearTargetVersionRef: true,
          enabled: true,
        })
      : await this.createCompositionSlot(projectId, {
          slotId: PROJECT_MEMORY_SLOT_ID,
          slotKind: 'memory',
          targetModule: 'memory',
          targetRef: memorySpaceId,
          priority: -500,
          enabled: true,
          policyJson: '{}',
        });
    return [
      ...slots.filter((candidate) => candidate.slotId !== PROJECT_MEMORY_SLOT_ID),
      savedSlot,
    ];
  }
}

function promptsTemplateRecord(value: unknown): Record<string, unknown> {
  return extractResourceRecord(value);
}

export const agentProjectService = new AgentProjectService();
