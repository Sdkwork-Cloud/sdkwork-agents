export interface AgentEngineConfigFileView {
  engineId: string;
  configFilePath: string;
  format: 'toml' | 'json' | 'env' | 'text';
  content: string;
  exists: boolean;
}
