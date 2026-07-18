import React from 'react';
import { PanelLeftClose, PanelLeftOpen, Settings } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ModelSelector } from './ModelSelector';

interface ChatHeaderProps {
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
  selectedModel: string;
  selectedVendor: string;
  isModelSelectorOpen: boolean;
  setIsModelSelectorOpen: (v: boolean) => void;
  setSelectedVendor: (v: string) => void;
  setSelectedModel: (m: string) => void;
  onOpenSettings: () => void;
}

export const ChatHeader: React.FC<ChatHeaderProps> = ({
  isSidebarOpen,
  toggleSidebar,
  selectedModel,
  selectedVendor,
  isModelSelectorOpen,
  setIsModelSelectorOpen,
  setSelectedVendor,
  setSelectedModel,
  onOpenSettings
}) => {
  const { t } = useTranslation('common');

  return (
    <header className="absolute top-0 left-0 right-0 h-14 flex items-center justify-between px-4 shrink-0 z-20 bg-transparent pointer-events-none">
      <div className="flex items-center gap-1 pointer-events-auto mt-2">
        {!isSidebarOpen && (
          <button
            onClick={toggleSidebar}
            className="p-1.5 mr-1 text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 rounded-md transition-colors hover:bg-gray-200/50 dark:hover:bg-[#2f2f2f]/50"
            title={t('openSidebar')}
          >
            <PanelLeftOpen size={18} />
          </button>
        )}
        <ModelSelector 
          selectedModel={selectedModel}
          setSelectedModel={setSelectedModel}
          selectedVendor={selectedVendor}
          setSelectedVendor={setSelectedVendor}
          isOpen={isModelSelectorOpen}
          setIsOpen={setIsModelSelectorOpen}
        />
      </div>
      <div className="flex items-center gap-2 pointer-events-auto mt-2">
        <button 
          onClick={onOpenSettings}
          className="text-gray-400 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 transition-colors p-1.5 rounded-lg hover:bg-gray-200/50 dark:hover:bg-[#2f2f2f]/50"
          title={t('settings')}
        >
          <Settings size={18} />
        </button>
      </div>
    </header>
  );
}
