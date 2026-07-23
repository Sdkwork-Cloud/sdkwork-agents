import type { Int64String } from './int64-string';

export interface AgentItemDriveRefRecord {
  resourceRole: 'attachment' | 'image' | 'audio' | 'generated_output' | 'artifact';
  driveSpaceId: string;
  driveNodeId: string;
  mediaResourceId?: string | null;
  objectBlobId?: string | null;
  resourceHash?: string | null;
  altText?: string | null;
  sortOrder: number;
  status: 'active' | 'unavailable' | 'deleted';
  createdBy: Int64String;
  createdAt: string;
  updatedAt: string;
  deletedAt?: string;
  retentionUntil?: string;
}
