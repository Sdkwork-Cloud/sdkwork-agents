export const AGENTS_PC_MODULES = [
  'agents',
  'chat',
  'inspiration',
  'creative',
  'assets',
  'presentation',
  'canvas',
] as const;

export type AgentsPcModuleId = typeof AGENTS_PC_MODULES[number];
