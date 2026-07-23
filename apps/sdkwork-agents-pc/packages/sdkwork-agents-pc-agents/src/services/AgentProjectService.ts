import {
  getAgentsAppSdkClientWithSession,
  type AgentProjectCompositionSlotRecord,
  type AgentProjectRecord,
  type SdkworkAgentsAppClient,
} from '@sdkwork/agents-pc-core/sdk/agentsAppSdkClient';
import {
  getMemoryAppSdkClientWithSession,
  type MemorySpaceList,
} from '@sdkwork/agents-pc-core/sdk/memoryAppSdkClient';
import {
  getPromptsAppSdkClientWithSession,
  type PromptTemplatePage,
  type PromptTemplateVersionPage,
} from '@sdkwork/agents-pc-core/sdk/promptsAppSdkClient';

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

function projectFromRecord(record: AgentProjectRecord): AgentProject {
  if (!record.projectId || !record.name) {
    throw new Error('Project response is missing projectId or name.');
  }
  return {
    id: record.id,
    projectId: record.projectId,
    name: record.name,
    description: record.description ?? undefined,
    visibility: record.visibility,
    status: record.status,
    driveAccessMode: record.driveAccessMode,
    defaultAgentId: record.defaultAgentId ?? undefined,
    defaultModelId: record.defaultModelId ?? undefined,
    version: record.version,
    updatedAt: record.updatedAt,
  };
}

function projectCompositionSlotFromRecord(
  record: AgentProjectCompositionSlotRecord,
): AgentProjectCompositionSlot {
  if (!record.projectId || !record.slotId || !record.targetRef) {
    throw new Error('Project composition slot response is incomplete.');
  }
  return {
    id: record.id,
    projectId: record.projectId,
    slotId: record.slotId,
    slotKind: record.slotKind,
    targetModule: record.targetModule,
    targetRef: record.targetRef,
    targetVersionRef: record.targetVersionRef ?? undefined,
    priority: record.priority,
    enabled: record.enabled,
    policyJson: record.policyJson,
    version: record.version,
    updatedAt: record.updatedAt,
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
    return response.items.map(projectFromRecord);
  }

  async retrieve(projectId: string): Promise<AgentProject> {
    return projectFromRecord(await this.getClient().ai.agents.projects.retrieve(projectId));
  }

  async create(input: CreateAgentProjectInput): Promise<AgentProject> {
    return projectFromRecord(await this.getClient().ai.agents.projects.create(input));
  }

  async update(
    projectId: string,
    patch: Partial<CreateAgentProjectInput> & { expectedVersion: string },
  ): Promise<AgentProject> {
    return projectFromRecord(await this.getClient().ai.agents.projects.update(projectId, patch));
  }

  async archive(projectId: string, expectedVersion: string): Promise<AgentProject> {
    return projectFromRecord(
      await this.getClient().ai.agents.projects.archive(projectId, { expectedVersion }),
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
    return response.items.map(projectCompositionSlotFromRecord);
  }

  async createCompositionSlot(
    projectId: string,
    input: ProjectCompositionSlotInput,
  ): Promise<AgentProjectCompositionSlot> {
    return projectCompositionSlotFromRecord(
      await this.getClient().ai.agents.projectCompositionSlots.create(projectId, input),
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
      await this.getClient().ai.agents.projectCompositionSlots.update(
        projectId,
        slot.slotId,
        { ...patch, expectedVersion: slot.version },
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
    const versions = (response as unknown as PromptTemplateVersionPage).items;
    const selected = slot.targetVersionRef
      ? versions.find((record) => record.id === slot.targetVersionRef)
      : versions[0];
    return selected?.content ?? '';
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
    const templatePage = await prompts.templates.list({ limit: 200 });
    const templates = (templatePage as unknown as PromptTemplatePage).items;
    let template = templates.find((record) => record.key === templateKey);
    if (!template) {
      template = await prompts.templates.create({
        key: templateKey,
        name: `${project.name} instructions`,
        description: `Managed instructions for ${project.projectId}`,
        tags: ['sdkwork-agents', 'project-instructions'],
      });
    }
    const templateId = template.id;
    if (!templateId) throw new Error('Prompt template creation did not return an id.');

    const versionLabel = `project-slot-v${existingSlot ? Number(existingSlot.version) + 1 : 0}`;
    const versionPage = await prompts.templateVersions.list(templateId);
    const versions = (versionPage as unknown as PromptTemplateVersionPage).items;
    let version = versions.find(
      (record) => record.version_label === versionLabel,
    );
    if (!version) {
      version = await prompts.templateVersions.create(templateId, {
        version_label: versionLabel,
        content: normalizedContent,
        variables: [],
      });
    }
    const versionId = version.id;
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
    return (response as unknown as MemorySpaceList).items.map((space) => ({
      spaceId: space.spaceId,
      displayName: space.displayName,
    }));
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

export const agentProjectService = new AgentProjectService();
