export interface MediaToolDirectoryEntry {
  toolId: string;
  category: string;
  name: string;
  displayName: string;
  version?: string;
  description?: string;
  inputSchema?: Record<string, unknown>;
  outputSchema?: Record<string, unknown>;
  sideEffectLevel?: string;
  policyCategories?: string[];
  timeoutMs?: string;
  availability?: string;
  enabled: boolean;
  saveToDriveDefault?: boolean;
  configured?: boolean;
}
