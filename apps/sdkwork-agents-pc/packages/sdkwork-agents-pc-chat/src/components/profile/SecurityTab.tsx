import React from 'react';
import { Laptop, Smartphone, AlertCircle } from 'lucide-react';

interface ActiveSession {
  id: string;
  device: string;
  browser: string;
  ip: string;
  location: string;
  activeNow: boolean;
  type: 'desktop' | 'mobile';
}

interface SecurityTabProps {
  t: (key: string) => string;
  sessions: ActiveSession[];
  setSessions: React.Dispatch<React.SetStateAction<ActiveSession[]>>;
  handleRevokeSessions: () => void;
  triggerToast: (msg: string) => void;
}

export const SecurityTab: React.FC<SecurityTabProps> = ({
  t,
  sessions,
  setSessions,
  handleRevokeSessions,
  triggerToast
}) => {
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-start">
        <div>
          <h3 className="text-lg font-bold text-gray-900 dark:text-white tracking-tight">{t('sessions')}</h3>
          <p className="text-xs text-gray-500 mt-1">{t('sessionsSubtitle')}</p>
        </div>
        {sessions.length > 1 && (
          <button
            onClick={handleRevokeSessions}
            className="px-4 py-2 bg-rose-500/10 hover:bg-rose-500 text-rose-500 hover:text-white text-xs font-bold rounded-xl transition-all cursor-pointer"
          >
            {t('revokeAll')}
          </button>
        )}
      </div>

      <div className="rounded-2xl border border-gray-100 dark:border-zinc-800 divide-y divide-gray-100 dark:divide-zinc-850 overflow-hidden bg-gray-50/10 dark:bg-[#1a1a1a]/5">
        {sessions.map((session) => (
          <div key={session.id} className="p-4 flex items-center justify-between hover:bg-gray-50/50 dark:hover:bg-zinc-900/30 transition-colors">
            <div className="flex items-center gap-4.5">
              <div className="w-10 h-10 rounded-xl bg-gray-100 dark:bg-zinc-800 flex items-center justify-center text-gray-500 dark:text-zinc-400 shrink-0">
                {session.type === 'desktop' ? <Laptop size={20} /> : <Smartphone size={20} />}
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-bold text-gray-800 dark:text-zinc-200">{session.device}</span>
                  {session.activeNow ? (
                    <span className="px-2 py-0.5 rounded-full text-[9px] font-semibold bg-emerald-500/15 text-emerald-500 leading-none">
                      {t('statusActive')}
                    </span>
                  ) : (
                    <span className="text-[10px] text-gray-400 font-mono">ID: {session.id}</span>
                  )}
                </div>
                <p className="text-[11px] text-gray-500/90 mt-0.5 font-mono">
                  {session.browser} • {session.ip} • <span className="font-sans font-medium text-gray-400">{session.location}</span>
                </p>
              </div>
            </div>

            {!session.activeNow && (
              <button
                onClick={() => {
                  setSessions(prev => prev.filter(s => s.id !== session.id));
                  triggerToast(t('sessionDisconnected'));
                }}
                className="px-3 py-1.5 border border-gray-200 dark:border-zinc-850 text-gray-500 hover:text-red-500 hover:border-red-200 dark:hover:border-red-900 rounded-xl text-[10px] font-bold transition-all bg-white dark:bg-zinc-900"
              >
                {t('revokeAccess')}
              </button>
            )}
          </div>
        ))}
      </div>

      <div className="p-4 rounded-xl bg-orange-500/5 border border-orange-500/10 text-orange-500 text-xs flex items-start gap-2.5">
        <AlertCircle size={15} className="shrink-0 mt-0.5" />
        <p className="leading-relaxed font-medium">{t('suspiciousAlert')}</p>
      </div>
    </div>
  );
};
