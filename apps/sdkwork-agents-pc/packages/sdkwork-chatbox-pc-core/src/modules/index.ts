export const CHATBOX_PC_MODULES = [
  'chat',
  'inspiration',
  'creative',
  'assets',
  'ppt',
  'canvas',
] as const;

export type ChatboxPcModuleId = typeof CHATBOX_PC_MODULES[number];
