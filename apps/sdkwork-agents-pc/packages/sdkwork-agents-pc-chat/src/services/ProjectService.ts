import type { ChatSession } from '../types';

export type ProjectVisibility = 'private' | 'organization' | 'shared';
export type ProjectDriveAccessMode = 'disabled' | 'owner_library' | 'explicit_resources';

export interface ChatProject {
  projectId: string;
  name: string;
  description?: string;
  visibility: ProjectVisibility;
  driveAccessMode: ProjectDriveAccessMode;
  version: string;
  updatedAt: string;
}

export interface ProjectDetails extends ChatProject {
  chats: ChatSession[];
}

export interface ChatProjectCompositionSlot {
  id: string;
  projectId: string;
  slotId: string;
  slotKind: 'prompt' | 'memory' | 'knowledge' | 'skill' | 'mcp' | 'drive' | 'tool';
  targetModule: 'prompts' | 'memory' | 'knowledgebase' | 'skills' | 'mcp' | 'drive' | 'tools';
  targetRef: string;
  targetVersionRef?: string;
  priority: number;
  enabled: boolean;
  policyJson: string;
  version: string;
  updatedAt: string;
}

export interface ChatMemorySpaceOption {
  spaceId: string;
  displayName: string;
}

export interface ProjectSettingsData {
  project: ChatProject;
  slots: ChatProjectCompositionSlot[];
  instructions: string;
  memorySpaces: ChatMemorySpaceOption[];
  memorySpaceId?: string;
}

export interface ProjectPort {
  list(): Promise<ChatProject[]>;
  retrieve(projectId: string): Promise<ChatProject>;
  create(input: { name: string }): Promise<ChatProject>;
  update(
    projectId: string,
    patch: Partial<Pick<ChatProject, 'name' | 'description' | 'visibility' | 'driveAccessMode'>> & {
      expectedVersion: string;
    },
  ): Promise<ChatProject>;
  delete(projectId: string): Promise<void>;
  listCompositionSlots(projectId: string): Promise<ChatProjectCompositionSlot[]>;
  getInstructions(slots: ChatProjectCompositionSlot[]): Promise<string>;
  saveInstructions(
    project: ChatProject,
    slots: ChatProjectCompositionSlot[],
    content: string,
  ): Promise<ChatProjectCompositionSlot[]>;
  listMemorySpaces(): Promise<ChatMemorySpaceOption[]>;
  saveMemorySpace(
    projectId: string,
    slots: ChatProjectCompositionSlot[],
    memorySpaceId?: string,
  ): Promise<ChatProjectCompositionSlot[]>;
}

let projectPort: ProjectPort | null = null;

export function configureProjectPort(port: ProjectPort): void {
  projectPort = port;
}

function requireProjectPort(): ProjectPort {
  if (!projectPort) {
    throw new Error('Project port is not configured.');
  }
  return projectPort;
}

export class ProjectService {
  static getProjects(): Promise<ChatProject[]> {
    return requireProjectPort().list();
  }

  static async getProjectDetails(projectId: string): Promise<ProjectDetails> {
    const project = await requireProjectPort().retrieve(projectId);
    return { ...project, chats: [] };
  }

  static createProject(name: string): Promise<ChatProject> {
    return requireProjectPort().create({ name });
  }

  static updateProject(
    project: ChatProject,
    patch: Partial<Pick<ChatProject, 'name' | 'description' | 'visibility' | 'driveAccessMode'>>,
  ): Promise<ChatProject> {
    return requireProjectPort().update(project.projectId, {
      ...patch,
      expectedVersion: project.version,
    });
  }

  static deleteProject(projectId: string): Promise<void> {
    return requireProjectPort().delete(projectId);
  }

  static async getProjectSettings(projectId: string): Promise<ProjectSettingsData> {
    const port = requireProjectPort();
    const [project, slots, memorySpaces] = await Promise.all([
      port.retrieve(projectId),
      port.listCompositionSlots(projectId),
      port.listMemorySpaces(),
    ]);
    const instructions = await port.getInstructions(slots);
    const memorySpaceId = slots.find(
      (slot) => slot.slotId === 'slot.project.memory' && slot.enabled,
    )?.targetRef;
    return { project, slots, instructions, memorySpaces, memorySpaceId };
  }

  static saveProjectInstructions(
    project: ChatProject,
    slots: ChatProjectCompositionSlot[],
    content: string,
  ): Promise<ChatProjectCompositionSlot[]> {
    return requireProjectPort().saveInstructions(project, slots, content);
  }

  static saveProjectMemorySpace(
    projectId: string,
    slots: ChatProjectCompositionSlot[],
    memorySpaceId?: string,
  ): Promise<ChatProjectCompositionSlot[]> {
    return requireProjectPort().saveMemorySpace(projectId, slots, memorySpaceId);
  }
}
