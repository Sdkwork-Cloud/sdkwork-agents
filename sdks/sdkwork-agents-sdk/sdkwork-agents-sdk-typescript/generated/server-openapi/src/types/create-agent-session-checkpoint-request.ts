export interface CreateAgentSessionCheckpointRequest {
  checkpointId?: string;
  turnId?: string;
  runtimeBindingId?: string;
  checkpointKind: string;
  providerCheckpointRef?: string;
  driveSpaceId?: string;
  driveNodeId?: string;
  resumable: boolean;
  retentionUntil?: string;
  requestedAt: string;
}
