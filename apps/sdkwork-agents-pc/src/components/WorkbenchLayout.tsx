import { lazy, Suspense, useEffect, useState, type ComponentType, type LazyExoticComponent } from 'react';
import { useTranslation } from 'react-i18next';
import { GlobalSidebar } from './GlobalSidebar';
import { AgentStatusIndicator } from './AgentStatusIndicator';
import { DEFAULT_WORKBENCH_TAB, isWorkbenchTab, type WorkbenchTab } from './workbenchTabs';

const AgentWorkspace = lazy(() => import('@/src/agents').then((module) => ({ default: module.AgentWorkspace })));
const ChatView = lazy(() => import('@sdkwork/agents-pc-chat').then((module) => ({ default: module.ChatView })));
const InspirationView = lazy(() => import('@sdkwork/agents-pc-inspiration').then((module) => ({ default: module.InspirationView })));
const CreativeView = lazy(() => import('@sdkwork/agents-pc-creative').then((module) => ({ default: module.CreativeView })));
const AssetsView = lazy(() => import('@sdkwork/agents-pc-assets').then((module) => ({ default: module.AssetsView })));
const PresentationView = lazy(() => import('@sdkwork/agents-pc-presentation').then((module) => ({ default: module.PresentationView })));
const CanvasView = lazy(() => import('@sdkwork/agents-pc-canvas').then((module) => ({ default: module.CanvasView })));

const WORKBENCH_VIEW_BY_TAB: Record<WorkbenchTab, LazyExoticComponent<ComponentType>> = {
  agents: AgentWorkspace,
  chat_session: ChatView,
  inspiration: InspirationView,
  creative: CreativeView,
  assets: AssetsView,
  presentation: PresentationView,
  canvas: CanvasView,
};

const AVATAR_COLOR_TEMPLATES = [
  'bg-[#1890ff] text-white',
  'bg-emerald-500 text-white',
  'bg-violet-500 text-white',
  'bg-orange-500 text-white',
  'bg-rose-500 text-white',
  'bg-zinc-800 border border-zinc-700 text-white'
];

export const WorkbenchLayout = () => {
  const [activeTab, setActiveTab] = useState<WorkbenchTab>(DEFAULT_WORKBENCH_TAB);
  const { t: tCommon } = useTranslation('common');

  const [username, setUsername] = useState(() => localStorage.getItem('profile_username') || tCommon('mockUserName'));
  const [avatarIndex, setAvatarIndex] = useState(() => parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));

  useEffect(() => {
    const handleStorageChange = () => {
      setUsername(localStorage.getItem('profile_username') || tCommon('mockUserName'));
      setAvatarIndex(parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));
    };
    const handleSwitchTab = (e: Event) => {
      const customEvent = e as CustomEvent<{ tab?: unknown }>;
      if (isWorkbenchTab(customEvent.detail?.tab)) {
        setActiveTab(customEvent.detail.tab);
      }
    };
    window.addEventListener('storage', handleStorageChange);
    window.addEventListener('switch-tab', handleSwitchTab);
    const interval = setInterval(handleStorageChange, 800);
    return () => {
      window.removeEventListener('storage', handleStorageChange);
      window.removeEventListener('switch-tab', handleSwitchTab);
      clearInterval(interval);
    };
  }, [tCommon]);

  const avatarBg = AVATAR_COLOR_TEMPLATES[avatarIndex] || AVATAR_COLOR_TEMPLATES[0];
  const ActiveWorkspace = WORKBENCH_VIEW_BY_TAB[activeTab];

  return (
    <div className="flex h-[100dvh] w-full bg-[#f5f5f5] dark:bg-[#191919] font-sans text-gray-900 dark:text-gray-100 overflow-hidden">
      <AgentStatusIndicator />
      <GlobalSidebar 
        activeTab={activeTab} 
        setActiveTab={setActiveTab} 
        avatarBg={avatarBg} 
        username={username} 
      />

      {/* Main Content Area */}
      <div className="flex-1 w-0 flex flex-col relative overflow-hidden bg-[#f5f5f5] dark:bg-[#191919]">
        <Suspense fallback={<div className="flex flex-1 items-center justify-center text-sm text-gray-500">正在加载工作台…</div>}>
          <ActiveWorkspace />
        </Suspense>
      </div>
    </div>
  );
};
