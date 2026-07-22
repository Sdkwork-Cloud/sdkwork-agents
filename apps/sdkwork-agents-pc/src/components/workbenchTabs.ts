export const WORKBENCH_TABS = [
  'chat_session',
  'inspiration',
  'creative',
  'assets',
  'canvas',
  'agents',
] as const;

export type WorkbenchTab = typeof WORKBENCH_TABS[number];

export type SidebarTab = WorkbenchTab;

export const SIDEBAR_TABS: readonly SidebarTab[] = WORKBENCH_TABS;

export const DEFAULT_WORKBENCH_TAB: WorkbenchTab = 'chat_session';

export function isWorkbenchTab(value: unknown): value is WorkbenchTab {
  return typeof value === 'string' && WORKBENCH_TABS.some((tab) => tab === value);
}
