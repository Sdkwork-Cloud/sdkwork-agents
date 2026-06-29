export interface McpServerMarketplaceRecord {
  agentId: string;
  slotId: string;
  serverId: string;
  targetModule: string;
  targetRef: string;
  targetVersionRef?: string;
  enabled: boolean;
  priority: number;
}
