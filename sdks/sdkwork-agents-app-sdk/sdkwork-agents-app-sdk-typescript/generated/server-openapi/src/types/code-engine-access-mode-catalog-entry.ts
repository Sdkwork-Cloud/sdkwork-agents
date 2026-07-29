export interface CodeEngineAccessModeCatalogEntry {
  modeId: string;
  displayName: string;
  description: string;
  approvalBehavior: 'user_review' | 'automatic_review' | 'never' | 'provider_default';
  workspaceAccess: 'read_only' | 'workspace_write' | 'full_access' | 'provider_default';
  networkAccess: 'restricted' | 'enabled' | 'provider_default';
  riskLevel: 'scoped' | 'elevated' | 'unrestricted';
  enabled: boolean;
  disabledReason?: string;
}
