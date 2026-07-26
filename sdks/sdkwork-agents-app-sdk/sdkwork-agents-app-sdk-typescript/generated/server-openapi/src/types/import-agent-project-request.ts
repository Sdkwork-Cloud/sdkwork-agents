export interface ImportAgentProjectRequest {
  workspaceId: string;
  projectId?: string;
  name: string;
  description?: string;
  sourceKind: string;
  sourceRef: string;
  driveSpaceId: string;
  driveRootEntryId: string;
  driveLogicalPath?: string;
}
