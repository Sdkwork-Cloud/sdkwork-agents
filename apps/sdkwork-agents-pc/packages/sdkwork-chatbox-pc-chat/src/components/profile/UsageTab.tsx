import React from 'react';
import { RefreshCw } from 'lucide-react';

interface UsageTabProps {
  t: (key: string) => string;
  tokenUsage: number;
  messageUsage: number;
  imageUsage: number;
  handleSimulateCall: () => void;
}

export const UsageTab: React.FC<UsageTabProps> = ({
  t,
  tokenUsage,
  messageUsage,
  imageUsage,
  handleSimulateCall
}) => {
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-start">
        <div>
          <h3 className="text-lg font-bold text-gray-900 dark:text-white tracking-tight">{t('usage')}</h3>
          <p className="text-xs text-gray-500 mt-1">{t('usageSubtitle')}</p>
        </div>
        <button
          onClick={handleSimulateCall}
          className="flex items-center gap-1.5 px-4 py-2 bg-gray-100 dark:bg-zinc-850 hover:bg-gray-200 dark:hover:bg-zinc-800 text-gray-800 dark:text-zinc-200 text-xs font-bold rounded-xl transition-all border border-gray-200/50 dark:border-transparent cursor-pointer"
        >
          <RefreshCw size={13} className="text-[#1890ff]" />
          {t('simulateUsage')}
        </button>
      </div>

      <div className="space-y-5">
        <div className="p-5 rounded-2xl border border-gray-100 dark:border-zinc-850 bg-gray-50/20 dark:bg-zinc-900/10">
          <div className="flex justify-between items-center mb-2">
            <div className="flex items-center gap-2">
              <div className="w-2.5 h-2.5 rounded-full bg-[#1890ff]" />
              <span className="text-xs font-bold text-gray-800 dark:text-zinc-200">{t('tokenUsage')}</span>
            </div>
            <span className="text-xs font-mono text-gray-600 dark:text-zinc-400">
              {(tokenUsage / 1000000).toFixed(2)}M / 5.00M Token
            </span>
          </div>
          <div className="w-full bg-gray-100 dark:bg-zinc-800 h-2.5 rounded-full overflow-hidden">
            <div 
              className="bg-gradient-to-r from-blue-500 to-indigo-500 h-full rounded-full transition-all duration-500" 
              style={{ width: `${(tokenUsage / 5000000) * 100}%` }}
            />
          </div>
          <div className="flex justify-between text-[10px] text-gray-400 mt-1.5 font-mono">
            <span>0%</span>
            <span>{t('warningThreshold')}</span>
            <span>{( (tokenUsage/5000000) * 100 ).toFixed(1)}% {t('used')}</span>
          </div>
        </div>

        <div className="p-5 rounded-2xl border border-gray-100 dark:border-zinc-850 bg-gray-50/20 dark:bg-zinc-900/10">
          <div className="flex justify-between items-center mb-2">
            <div className="flex items-center gap-2">
              <div className="w-2.5 h-2.5 rounded-full bg-violet-500" />
              <span className="text-xs font-bold text-gray-800 dark:text-zinc-200">{t('messageQuota')}</span>
            </div>
            <span className="text-xs font-mono text-gray-600 dark:text-zinc-400">
              {messageUsage} / 1000
            </span>
          </div>
          <div className="w-full bg-gray-100 dark:bg-zinc-800 h-2.5 rounded-full overflow-hidden">
            <div 
              className="bg-gradient-to-r from-violet-500 to-purple-500 h-full rounded-full transition-all duration-500" 
              style={{ width: `${(messageUsage / 1000) * 100}%` }}
            />
          </div>
          <div className="flex justify-between text-[10px] text-gray-400 mt-1.5 font-mono">
            <span>0%</span>
            <span>{t('restartBillingDate')}</span>
            <span>{((messageUsage / 1000) * 100).toFixed(1)}% {t('used')}</span>
          </div>
        </div>

        <div className="p-5 rounded-2xl border border-gray-100 dark:border-zinc-850 bg-gray-50/20 dark:bg-zinc-900/10">
          <div className="flex justify-between items-center mb-2">
            <div className="flex items-center gap-2">
              <div className="w-2.5 h-2.5 rounded-full bg-rose-500" />
              <span className="text-xs font-bold text-gray-800 dark:text-zinc-200">{t('imageQuota')}</span>
            </div>
            <span className="text-xs font-mono text-gray-600 dark:text-zinc-400">
              {imageUsage} / 200
            </span>
          </div>
          <div className="w-full bg-gray-100 dark:bg-zinc-800 h-2.5 rounded-full overflow-hidden">
            <div 
              className="bg-gradient-to-r from-pink-500 to-rose-500 h-full rounded-full transition-all duration-500" 
              style={{ width: `${(imageUsage / 200) * 100}%` }}
            />
          </div>
          <div className="flex justify-between text-[10px] text-gray-400 mt-1.5 font-mono">
            <span>0%</span>
            <span>{t('limitSingleCycle')}</span>
            <span>{((imageUsage / 200) * 100).toFixed(1)}% {t('used')}</span>
          </div>
        </div>
      </div>
    </div>
  );
};
