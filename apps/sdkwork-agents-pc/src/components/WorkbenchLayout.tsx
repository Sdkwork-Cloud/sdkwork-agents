import React, { lazy, Suspense, useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { GlobalSidebar, type WorkbenchTab } from './GlobalSidebar';
import { AgentStatusIndicator } from './AgentStatusIndicator';

const AgentWorkspace = lazy(() => import('@/src/agents').then((module) => ({ default: module.AgentWorkspace })));

const AVATAR_COLOR_TEMPLATES = [
  'bg-[#1890ff] text-white',
  'bg-emerald-500 text-white',
  'bg-violet-500 text-white',
  'bg-orange-500 text-white',
  'bg-rose-500 text-white',
  'bg-zinc-800 border border-zinc-700 text-white'
];

export const WorkbenchLayout = () => {
  const [activeTab, setActiveTab] = useState<WorkbenchTab>('agents');
  const { t: tCommon } = useTranslation('common');

  const [username, setUsername] = useState(() => localStorage.getItem('profile_username') || tCommon('mockUserName'));
  const [avatarIndex, setAvatarIndex] = useState(() => parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));

  useEffect(() => {
    const handleStorageChange = () => {
      setUsername(localStorage.getItem('profile_username') || tCommon('mockUserName'));
      setAvatarIndex(parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));
    };
    const handleSwitchTab = (e: Event) => {
      const customEvent = e as CustomEvent;
      if (customEvent.detail && customEvent.detail.tab) {
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
          {activeTab === 'agents' && <AgentWorkspace />}
        </Suspense>
      </div>
    </div>
  );
};
