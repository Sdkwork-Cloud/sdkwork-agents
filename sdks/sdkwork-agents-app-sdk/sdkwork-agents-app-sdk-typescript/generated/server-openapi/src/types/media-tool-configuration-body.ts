export interface MediaToolConfigurationBody {
  enabled: boolean;
  saveToDriveDefault?: boolean;
  defaultArguments?: Record<string, unknown>;
  expectedVersion?: string;
}
