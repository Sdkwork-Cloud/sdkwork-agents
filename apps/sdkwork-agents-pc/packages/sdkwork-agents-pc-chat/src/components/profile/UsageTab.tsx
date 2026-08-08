import React from 'react';
import { Info } from 'lucide-react';

interface UsageTabProps {
  t: (key: string) => string;
}

/**
 * Usage statistics panel.
 *
 * Token/message/image quotas are owned by the platform billing service
 * (membership privileges). Real usage is exposed through the membership SDK
 * (`AppMembershipPrivilegeUsageResponse`); until that data is wired into this
 * surface we render an honest empty state instead of fake numbers.
 */
export const UsageTab: React.FC<UsageTabProps> = ({ t }) => {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-gray-900 dark:text-white tracking-tight">{t('usage')}</h3>
        <p className="text-xs text-gray-500 mt-1">{t('usageSubtitle')}</p>
      </div>

      <div className="p-6 rounded-2xl border border-gray-100 dark:border-zinc-850 bg-gray-50/20 dark:bg-zinc-900/10 flex flex-col items-center justify-center gap-3 text-center">
        <Info size={20} className="text-[#1890ff]" />
        <p className="text-xs font-bold text-gray-800 dark:text-zinc-200">{t('usageEmptyTitle')}</p>
        <p className="text-[11px] text-gray-500 dark:text-zinc-400 max-w-xs leading-relaxed">
          {t('usageEmptyDescription')}
        </p>
      </div>
    </div>
  );
};
