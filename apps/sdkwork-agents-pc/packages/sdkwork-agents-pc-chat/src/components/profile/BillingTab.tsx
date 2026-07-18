import React from 'react';
import { CreditCard, Check } from 'lucide-react';

interface BillingTabProps {
  t: (key: string) => string;
  tCommon: (key: string) => string;
}

export const BillingTab: React.FC<BillingTabProps> = ({ t, tCommon }) => {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-gray-900 dark:text-white tracking-tight">{t('plan')}</h3>
        <p className="text-xs text-gray-500 mt-1">{t('billingSubtitle')}</p>
      </div>

      <div className="grid grid-cols-2 gap-5">
        <div className="p-6 rounded-3xl bg-gradient-to-br from-[#1890ff] to-cyan-500 text-white shadow-lg space-y-4 relative overflow-hidden">
          <div className="absolute right-0 bottom-0 translate-y-3 translate-x-3 opacity-15">
            <CreditCard size={180} />
          </div>
          <div>
            <span className="text-[9px] uppercase tracking-wider bg-white/20 px-2.5 py-0.5 rounded-full font-bold">{t('recommended')}</span>
            <h4 className="text-2xl font-black tracking-tight mt-1">{tCommon('proSubscription')}</h4>
            <p className="text-[11px] text-white/80 mt-1">{t('planDescription')}</p>
          </div>

          <div className="pt-2">
              <p className="text-sm font-semibold">$15.00 <span className="text-xs font-normal">/ month</span></p>
            <p className="text-[10px] text-white/70 mt-1">{t('billingCycleDesc')}</p>
          </div>
        </div>

        <div className="p-6 rounded-3xl bg-gray-50/60 dark:bg-zinc-900/30 border border-gray-100 dark:border-zinc-800 flex flex-col justify-between">
          <div>
            <h4 className="font-semibold text-sm text-gray-900 dark:text-zinc-100">{t('paymentMethod')}</h4>
            <div className="flex items-center gap-3 mt-4">
              <div className="w-10 h-7 rounded bg-white dark:bg-zinc-800 border border-gray-100 dark:border-zinc-700 flex items-center justify-center font-bold text-[10px] uppercase text-sky-800">
                Visa
              </div>
              <div>
                <p className="text-xs font-semibold text-gray-800 dark:text-zinc-200">{t('visaEnding')} •••• 9012</p>
                <p className="text-[10px] text-gray-500 mt-0.5">{t('expires')} 12 / 2029</p>
              </div>
            </div>
          </div>

          <div className="mt-4 flex gap-2">
            <button className="flex-1 py-2 text-center border border-gray-200 dark:border-zinc-800 text-gray-700 dark:text-zinc-300 text-xs font-bold rounded-xl hover:bg-gray-100 dark:hover:bg-zinc-800 transition-colors">
              {t('changeCard')}
            </button>
            <button className="flex-1 py-2 text-center text-[#1890ff] text-xs font-bold rounded-xl hover:bg-[#1890ff]/5 transition-colors">
              {t('invoices')}
            </button>
          </div>
        </div>
      </div>

      <div className="p-5 rounded-2xl bg-zinc-50 dark:bg-zinc-900/40 border border-zinc-100 dark:border-zinc-800 space-y-3">
        <h4 className="font-bold text-xs text-gray-900 dark:text-zinc-200 uppercase tracking-widest">{t('includedPremiumFeatures')}</h4>
        <ul className="grid grid-cols-2 gap-x-6 gap-y-2.5 text-xs text-gray-600 dark:text-zinc-400">
          <li className="flex items-center gap-2">
            <Check size={14} className="text-emerald-500" />
            {t('unlimitedArtifacts')}
          </li>
          <li className="flex items-center gap-2">
            <Check size={14} className="text-emerald-500" />
            {t('priorityApi')}
          </li>
          <li className="flex items-center gap-2">
            <Check size={14} className="text-emerald-500" />
            {t('extendedContext')}
          </li>
          <li className="flex items-center gap-2">
            <Check size={14} className="text-emerald-500" />
            {t('playgroundTools')}
          </li>
        </ul>
      </div>
    </div>
  );
};
