import React, { useState } from 'react';
import { ChevronDown, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '@sdkwork/agents-pc-commons';

const VENDORS = [
  { name: 'Google', models: ['gemini-2.5-flash', 'gemini-1.5-pro', 'gemini-1.5-flash', 'gemini-2.5-pro'] },
  { name: 'OpenAI', models: ['gpt-4o', 'gpt-4-turbo', 'gpt-3.5-turbo'] },
  { name: 'Anthropic', models: ['claude-3.5-sonnet', 'claude-3-opus', 'claude-3-haiku'] }
];

interface ModelSelectorProps {
  selectedModel: string;
  setSelectedModel: (m: string) => void;
  selectedVendor: string;
  setSelectedVendor: (v: string) => void;
  isOpen: boolean;
  setIsOpen: (v: boolean) => void;
}

export const ModelSelector: React.FC<ModelSelectorProps> = ({
  selectedModel,
  setSelectedModel,
  selectedVendor,
  setSelectedVendor,
  isOpen,
  setIsOpen
}) => {
  const { t } = useTranslation('chat');

  return (
    <div className="relative">
      <button 
        onClick={() => setIsOpen(!isOpen)} 
        className="flex items-center gap-1 group px-2 py-1 rounded-lg hover:bg-[#0000000a] dark:hover:bg-[#ffffff10] transition-all opacity-80 hover:opacity-100"
        title={t('model')}
      >
        <span className="text-[12px] font-medium text-gray-500 dark:text-gray-400 group-hover:text-gray-700 dark:group-hover:text-gray-300 transition-colors flex items-center gap-1">
          {selectedModel}
          <ChevronDown size={14} className="opacity-40 transition-transform group-hover:opacity-100" />
        </span>
      </button>
      
      {isOpen && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setIsOpen(false)} />
          <div className="absolute top-full left-0 mt-2 w-96 bg-white dark:bg-[#282828] border border-[#d9d9d9] dark:border-[#1a1a1a] rounded-xl shadow-xl z-50 overflow-hidden flex h-64">
            <div className="w-[120px] bg-[#f7f7f7] dark:bg-[#202020] border-r border-[#d9d9d9] dark:border-[#1a1a1a] flex flex-col py-2 shrink-0">
               {VENDORS.map(v => (
                 <button
                   key={v.name}
                   onMouseEnter={() => setSelectedVendor(v.name)}
                   onClick={() => setSelectedVendor(v.name)}
                   className={cn(
                     "px-4 py-3 text-sm text-left font-medium transition-colors",
                     selectedVendor === v.name ? "bg-[#e5f0ff] dark:bg-[#1890ff]/20 text-[#1890ff] dark:text-[#1890ff] border-l-2 border-[#1890ff]" : "text-gray-600 dark:text-gray-400 hover:bg-[#e6e6e6] dark:hover:bg-[#383838] hover:text-gray-900 dark:hover:text-gray-200 border-l-2 border-transparent"
                   )}
                 >
                   {v.name}
                 </button>
               ))}
            </div>
            <div className="flex-1 flex flex-col py-2 overflow-y-auto">
              {VENDORS.find(v => v.name === selectedVendor)?.models.map(m => (
                 <button
                   key={m}
                   onClick={() => { setSelectedModel(m); setIsOpen(false); }}
                   className={cn(
                     "px-4 py-2.5 text-sm font-medium transition-colors border-l-2 flex items-center justify-between",
                     selectedModel === m ? "text-gray-900 dark:text-gray-100 bg-[#f0f0f0] dark:bg-[#333] border-[#1890ff] dark:border-[#1890ff]" : "text-gray-600 dark:text-gray-400 hover:bg-[#f5f5f5] dark:hover:bg-[#383838] hover:text-gray-900 dark:hover:text-gray-200 border-transparent"
                   )}
                 >
                   <span>{m}</span>
                   {selectedModel === m && <Check size={16} className="text-[#1890ff] dark:text-[#1890ff]" />}
                 </button>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
};
