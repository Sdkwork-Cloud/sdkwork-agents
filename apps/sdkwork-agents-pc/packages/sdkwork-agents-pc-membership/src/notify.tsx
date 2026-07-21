import { useCallback, useEffect, useState } from 'react';
import { AlertCircle, CheckCircle2, Info, X } from 'lucide-react';

type NoticeTone = 'error' | 'info' | 'success';

interface Notice {
  id: number;
  message: string;
  tone: NoticeTone;
}

const NOTICE_STYLE: Record<NoticeTone, string> = {
  error: 'border-rose-500/25 bg-rose-950/90 text-rose-100',
  info: 'border-sky-500/25 bg-sky-950/90 text-sky-100',
  success: 'border-emerald-500/25 bg-emerald-950/90 text-emerald-100',
};

const NOTICE_ICON = {
  error: AlertCircle,
  info: Info,
  success: CheckCircle2,
} satisfies Record<NoticeTone, typeof AlertCircle>;

export function useAgentsTokenPlanNotify() {
  const [notices, setNotices] = useState<Notice[]>([]);
  const dismiss = useCallback((id: number) => {
    setNotices((current) => current.filter((notice) => notice.id !== id));
  }, []);
  const onNotify = useCallback((message: string, tone: NoticeTone) => {
    setNotices((current) => [...current, { id: Date.now() + current.length, message, tone }]);
  }, []);
  const NotifyOutlet = useCallback(() => (
    <div className="pointer-events-none fixed inset-x-0 top-4 z-[80] flex flex-col items-center gap-2 px-4">
      {notices.map((notice) => (
        <TokenPlanNotice key={notice.id} notice={notice} onDismiss={() => dismiss(notice.id)} />
      ))}
    </div>
  ), [dismiss, notices]);

  return { NotifyOutlet, onNotify };
}

function TokenPlanNotice({ notice, onDismiss }: { notice: Notice; onDismiss: () => void }) {
  const Icon = NOTICE_ICON[notice.tone];
  useEffect(() => {
    const timer = window.setTimeout(onDismiss, 3200);
    return () => window.clearTimeout(timer);
  }, [onDismiss]);

  return (
    <div className={`pointer-events-auto flex max-w-lg items-center gap-3 rounded-lg border px-4 py-3 shadow-2xl backdrop-blur ${NOTICE_STYLE[notice.tone]}`}>
      <Icon aria-hidden="true" className="h-5 w-5 shrink-0" />
      <span className="min-w-0 flex-1 text-sm font-medium">{notice.message}</span>
      <button aria-label="关闭通知" className="rounded-md p-1 hover:bg-white/10" onClick={onDismiss} type="button">
        <X aria-hidden="true" className="h-4 w-4" />
      </button>
    </div>
  );
}
