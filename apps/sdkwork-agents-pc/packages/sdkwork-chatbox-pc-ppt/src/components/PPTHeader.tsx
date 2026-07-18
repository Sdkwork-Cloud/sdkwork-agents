import React from 'react';
import { Play, Download, Settings, FileText, SplitSquareHorizontal } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';

interface PPTHeaderProps {
  layout: 'split' | 'edit' | 'preview';
  setLayout: (l: 'split' | 'edit' | 'preview') => void;
  theme: string;
  setTheme: (t: string) => void;
  themes: string[];
  onExport: () => void;
}

export const PPTHeader: React.FC<PPTHeaderProps> = ({ layout, setLayout, theme, setTheme, themes, onExport }) => {
  const { t } = useTranslation(['common', 'ppt']);

  return (
    <header className="h-14 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between px-6 shrink-0 z-10 bg-white dark:bg-[#0f0f0f]">
      <div className="flex items-center gap-4">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-200">{t('ppt:webPPTCreator')}</h2>
        <div className="flex bg-gray-100 dark:bg-[#1a1a1a] rounded-lg p-1 border border-gray-200 dark:border-gray-800">
          <button
            onClick={() => setLayout('edit')}
            className={cn("px-3 py-1.5 rounded-md text-xs font-medium transition-colors flex items-center gap-2", layout === 'edit' ? "bg-white dark:bg-[#2f2f2f] text-blue-600 dark:text-white shadow-sm" : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200")}
          >
            <FileText size={14} /> {t('ppt:edit')}
          </button>
          <button
            onClick={() => setLayout('split')}
            className={cn("px-3 py-1.5 rounded-md text-xs font-medium transition-colors flex items-center gap-2", layout === 'split' ? "bg-white dark:bg-[#2f2f2f] text-blue-600 dark:text-white shadow-sm" : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200")}
          >
            <SplitSquareHorizontal size={14} /> {t('ppt:split')}
          </button>
          <button
            onClick={() => setLayout('preview')}
            className={cn("px-3 py-1.5 rounded-md text-xs font-medium transition-colors flex items-center gap-2", layout === 'preview' ? "bg-white dark:bg-[#2f2f2f] text-blue-600 dark:text-white shadow-sm" : "text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200")}
          >
            <Play size={14} /> {t('ppt:preview')}
          </button>
        </div>
      </div>
      
      <div className="flex items-center gap-4">
        <div className="relative group">
          <button className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 bg-gray-50 dark:bg-[#1a1a1a] px-3 py-1.5 rounded-lg border border-gray-200 dark:border-gray-800 transition-colors">
            <Settings size={14} /> {t('ppt:theme')}: {theme}
          </button>
          <div className="absolute right-0 top-full mt-1 w-32 bg-white dark:bg-[#1a1a1a] border border-gray-200 dark:border-gray-800 rounded-lg shadow-xl opacity-0 group-hover:opacity-100 pointer-events-none group-hover:pointer-events-auto transition-opacity z-50 overflow-hidden text-sm max-h-48 overflow-y-auto">
            {themes.map(tOption => (
              <button
                key={tOption}
                onClick={() => setTheme(tOption)}
                className={cn("w-full text-left px-3 py-2 hover:bg-gray-100 dark:hover:bg-[#2f2f2f] transition-colors", theme === tOption ? "text-blue-600 dark:text-indigo-400 font-medium" : "text-gray-600 dark:text-gray-300")}
              >
                {tOption}
              </button>
            ))}
          </div>
        </div>
        
        <button
          onClick={onExport}
          className="flex items-center gap-2 text-sm bg-blue-600 dark:bg-indigo-500 text-white px-3 py-1.5 rounded-lg shadow-md shadow-blue-500/20 dark:shadow-indigo-500/20 hover:bg-blue-500 dark:hover:bg-indigo-400 transition-colors"
        >
          <Download size={14} /> {t('ppt:exportHtml')}
        </button>
      </div>
    </header>
  );
};
