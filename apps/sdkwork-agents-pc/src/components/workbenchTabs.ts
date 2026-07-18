export const WORKBENCH_TABS = [
  'chat_session',
  'inspiration',
  'creative',
  'assets',
  'presentation',
  'canvas',
  'agents',
] as const;

export type WorkbenchTab = typeof WORKBENCH_TABS[number];

export const DEFAULT_WORKBENCH_TAB: WorkbenchTab = 'chat_session';

export function isWorkbenchTab(value: unknown): value is WorkbenchTab {
  return typeof value === 'string' && WORKBENCH_TABS.some((tab) => tab === value);
}
