import React, { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowLeft, Plus, MoreHorizontal, Bot, RefreshCw, ChevronDown, RotateCcw } from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import { agentService, type AgentConfig, type AgentLifecycleStatus } from '../services/AgentService';
import { createDefaultAvatar } from '../services/DefaultAvatarService';
import { toast } from '../components/Toast';
import { MobileActionSheet, MobileConfirmDialog } from '../components/MobileSheets';
import { t } from '../copy/mobileAgentTexts';

export interface AgentMobileViewProps {
  /** Host-navigated "start a chat with this agent" callback. */
  onStartChat?: (agent: AgentConfig) => void;
  /** Host-navigated create entry; when omitted the header "+" is hidden. */
  onCreateAgent?: () => void;
  /** Host-navigated edit entry (route with agent id). */
  onEditAgent?: (id: string) => void;
  /** Host-navigated back; when omitted the back chevron is hidden. */
  onBack?: () => void;
  /** Host toast port; defaults to the built-in agents toast. */
  notify?: (message: string, type?: 'info' | 'success' | 'error') => void;
}

type ViewStatus = 'loading' | 'error' | 'ready';

const STATUS_BADGE_STYLES: Record<AgentLifecycleStatus, string> = {
  draft: 'bg-amber-500/15 text-amber-600 dark:text-amber-400',
  active: 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400',
  disabled: 'bg-gray-500/15 text-[var(--color-text-sub,#6b7280)]',
  archived: 'bg-gray-500/15 text-[var(--color-text-sub,#6b7280)]',
  deleted: 'bg-red-500/15 text-red-500 dark:text-red-400',
};

function statusLabel(status: AgentLifecycleStatus | undefined): string {
  switch (status) {
    case 'draft':
      return t('agents.mobile.status.draft');
    case 'active':
      return t('agents.mobile.status.active');
    case 'disabled':
      return t('agents.mobile.status.disabled');
    case 'archived':
      return t('agents.mobile.status.archived');
    case 'deleted':
      return t('agents.mobile.status.deleted');
    default:
      return t('agents.mobile.status.published');
  }
}

function isDeletedAgent(agent: AgentConfig): boolean {
  return agent.status === 'deleted' || Boolean(agent.deletedAt);
}

const PAGE_SIZE = 20;

export const MyAgentsView: React.FC<AgentMobileViewProps> = ({
  onStartChat,
  onCreateAgent,
  onEditAgent,
  onBack,
  notify = toast,
}) => {
  const [status, setStatus] = useState<ViewStatus>('loading');
  const [agents, setAgents] = useState<AgentConfig[]>([]);
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  // Pull-to-refresh.
  const [pullDistance, setPullDistance] = useState(0);
  const [refreshing, setRefreshing] = useState(false);
  const pullStartY = useRef<number | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Action sheet / confirm.
  const [actionAgent, setActionAgent] = useState<AgentConfig | null>(null);
  const [confirmAgent, setConfirmAgent] = useState<AgentConfig | null>(null);
  const [deleting, setDeleting] = useState(false);

  const loadPage = useCallback(
    async (targetPage: number, append: boolean, silent = false) => {
      try {
        const result = await agentService.listAgentsPage({
          page: targetPage,
          pageSize: PAGE_SIZE,
          scope: 'mine',
          includeDeleted: true,
        });
        setAgents((prev) => (append ? [...prev, ...result.items] : result.items));
        setPage(targetPage);
        setHasMore(result.pageInfo.hasMore);
        setStatus('ready');
      } catch (error) {
        console.error('Failed to load my agents', error);
        if (!silent) {
          notify(t('agents.mobile.toast.loadFailed'), 'error');
        }
        if (!append) {
          setStatus('error');
        }
      }
    },
    [notify],
  );

  const refresh = useCallback(async () => {
    await loadPage(1, false);
  }, [loadPage]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const loadMore = useCallback(async () => {
    if (loadingMore || !hasMore || status !== 'ready') return;
    setLoadingMore(true);
    try {
      await loadPage(page + 1, true, true);
    } finally {
      setLoadingMore(false);
    }
  }, [loadingMore, hasMore, status, page, loadPage]);

  // Infinite scroll sentinel.
  const sentinelRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel || typeof IntersectionObserver === 'undefined') return undefined;
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) {
          void loadMore();
        }
      },
      { root: scrollRef.current, rootMargin: '120px 0px' },
    );
    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [loadMore]);

  const handleDelete = async () => {
    if (!confirmAgent || deleting) return;
    setDeleting(true);
    try {
      await agentService.deleteAgent(confirmAgent.id!);
      notify(t('agents.mobile.toast.deleted'), 'success');
      setConfirmAgent(null);
      void refresh();
    } catch (error) {
      console.error('Failed to delete agent', error);
      notify(t('agents.mobile.toast.deleteFailed'), 'error');
    } finally {
      setDeleting(false);
    }
  };

  const handleRestore = async (agent: AgentConfig) => {
    if (!agent.id) return;
    try {
      await agentService.restoreAgent(agent.id);
      notify(t('agents.mobile.toast.restored'), 'success');
      void refresh();
    } catch (error) {
      console.error('Failed to restore agent', error);
      notify(t('agents.mobile.toast.restoreFailed'), 'error');
    }
  };

  const handleStartChat = (agent: AgentConfig) => {
    if (onStartChat) {
      onStartChat(agent);
      return;
    }
    notify(t('agents.mobile.toast.chatPending'), 'info');
  };

  // Long-press to open the action sheet (touch + pointer).
  const longPressTimer = useRef<number | null>(null);
  const openActionSheet = (agent: AgentConfig) => setActionAgent(agent);
  const beginLongPress = (agent: AgentConfig) => {
    longPressTimer.current = window.setTimeout(() => openActionSheet(agent), 500);
  };
  const cancelLongPress = () => {
    if (longPressTimer.current !== null) {
      window.clearTimeout(longPressTimer.current);
      longPressTimer.current = null;
    }
  };

  // Pull-to-refresh gesture handlers.
  const onTouchStart = (event: React.TouchEvent) => {
    if (scrollRef.current && scrollRef.current.scrollTop <= 0 && !refreshing) {
      pullStartY.current = event.touches[0]?.clientY ?? null;
    }
  };
  const onTouchMove = (event: React.TouchEvent) => {
    if (pullStartY.current === null || refreshing) return;
    const delta = (event.touches[0]?.clientY ?? pullStartY.current) - pullStartY.current;
    if (delta > 0) {
      setPullDistance(Math.min(delta * 0.45, 80));
    }
  };
  const onTouchEnd = () => {
    pullStartY.current = null;
    if (pullDistance >= 56 && !refreshing) {
      setRefreshing(true);
      void refresh().finally(() => {
        setRefreshing(false);
        setPullDistance(0);
      });
    } else {
      setPullDistance(0);
    }
  };

  const activeAgents = agents.filter((agent) => !isDeletedAgent(agent));
  const deletedAgents = agents.filter((agent) => isDeletedAgent(agent));

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg-color,#f5f5f7)] overflow-hidden">
      {/* Header */}
      <header className="h-[56px] shrink-0 flex items-center justify-between px-2 bg-[var(--color-glass-bg,#f5f5f7)] backdrop-blur-xl border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] z-10">
        <div className="flex items-center flex-1 gap-1">
          {onBack && (
            <button
              type="button"
              aria-label={t('agents.mobile.back')}
              onClick={onBack}
              className="w-10 h-10 flex items-center justify-center rounded-full active:bg-black/5 dark:active:bg-white/10 transition-colors"
            >
              <ArrowLeft className="w-6 h-6 text-[var(--color-text-main,#111827)]" />
            </button>
          )}
        </div>
        <h1 className="absolute left-1/2 -translate-x-1/2 text-[17px] font-semibold text-[var(--color-text-main,#111827)]">
          {t('agents.mobile.title')}
        </h1>
        <div className="flex items-center flex-1 justify-end">
          {onCreateAgent && (
            <button
              type="button"
              aria-label={t('agents.mobile.create')}
              onClick={onCreateAgent}
              className="w-10 h-10 flex items-center justify-center rounded-full active:bg-black/5 dark:active:bg-white/10 transition-colors"
            >
              <Plus className="w-6 h-6 text-[var(--color-text-main,#111827)]" />
            </button>
          )}
        </div>
      </header>

      {/* Pull-to-refresh indicator */}
      {pullDistance > 0 && (
        <div
          className="shrink-0 flex items-center justify-center gap-2 text-[13px] text-[var(--color-text-sub,#6b7280)] transition-[height] overflow-hidden"
          style={{ height: pullDistance }}
        >
          <RefreshCw className={cn('w-4 h-4', refreshing ? 'animate-spin' : '')} />
          {refreshing
            ? t('agents.mobile.refreshing')
            : pullDistance >= 56
              ? t('agents.mobile.releaseToRefresh')
              : t('agents.mobile.pullToRefresh')}
        </div>
      )}

      {/* Content */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto overscroll-contain"
        onTouchStart={onTouchStart}
        onTouchMove={onTouchMove}
        onTouchEnd={onTouchEnd}
      >
        {status === 'loading' && (
          <div className="px-4 pt-2 space-y-2" aria-busy="true">
            {Array.from({ length: 6 }, (_, index) => (
              <div
                key={index}
                className="flex items-center gap-3 rounded-2xl bg-[var(--color-chat-other-bg,#262626)] px-4 py-3.5 animate-pulse"
              >
                <div className="w-14 h-14 rounded-2xl bg-black/5 dark:bg-white/10 shrink-0" />
                <div className="flex-1 space-y-2">
                  <div className="h-4 w-2/5 rounded bg-black/5 dark:bg-white/10" />
                  <div className="h-3 w-4/5 rounded bg-black/5 dark:bg-white/10" />
                </div>
              </div>
            ))}
          </div>
        )}

        {status === 'error' && (
          <div className="flex flex-col items-center justify-center h-full px-8">
            <div className="w-16 h-16 rounded-3xl bg-[var(--color-chat-other-bg,#262626)] flex items-center justify-center mb-4">
              <Bot className="w-8 h-8 text-gray-400" />
            </div>
            <h3 className="text-[16px] font-semibold text-[var(--color-text-main,#111827)] mb-1">
              {t('agents.mobile.error.title')}
            </h3>
            <p className="text-[13px] text-[var(--color-text-sub,#6b7280)] mb-6 text-center leading-relaxed">
              {t('agents.mobile.error.desc')}
            </p>
            <button
              type="button"
              onClick={() => {
                setStatus('loading');
                void refresh();
              }}
              className="flex items-center gap-2 px-7 h-11 rounded-full bg-[var(--color-primary-blue,#2b5ce7)] text-white text-[15px] font-medium active:scale-95 transition-transform"
            >
              <RefreshCw className="w-4 h-4" />
              {t('agents.mobile.error.retry')}
            </button>
          </div>
        )}

        {status === 'ready' && agents.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full px-8">
            <div className="w-20 h-20 rounded-[24px] bg-[var(--color-chat-other-bg,#262626)] flex items-center justify-center mb-5 shadow-sm">
              <Bot className="w-10 h-10 text-[var(--color-primary-blue,#2b5ce7)]" />
            </div>
            <h3 className="text-[17px] font-semibold text-[var(--color-text-main,#111827)] mb-2">
              {t('agents.mobile.empty.title')}
            </h3>
            <p className="text-[13px] text-[var(--color-text-sub,#6b7280)] mb-8 max-w-[240px] text-center leading-relaxed">
              {t('agents.mobile.empty.desc')}
            </p>
            {onCreateAgent && (
              <button
                type="button"
                onClick={onCreateAgent}
                className="px-8 h-12 rounded-full bg-[var(--color-primary-blue,#2b5ce7)] text-white text-[15px] font-medium active:scale-95 transition-transform shadow-lg shadow-[var(--color-primary-blue,#2b5ce7)]/25"
              >
                {t('agents.mobile.empty.create')}
              </button>
            )}
          </div>
        )}

        {status === 'ready' && agents.length > 0 && (
          <div className="px-4 py-3 space-y-6">
            {/* Active agents */}
            <section>
              {activeAgents.length > 0 && (
                <div className="rounded-2xl bg-[var(--color-chat-other-bg,#262626)] overflow-hidden divide-y divide-[var(--color-border-color,rgba(0,0,0,0.05))] dark:divide-[var(--color-border-color,rgba(255,255,255,0.05))]">
                  {activeAgents.map((agent) => (
                    <AgentRow
                      key={agent.id}
                      agent={agent}
                      showStatusBadge
                      onPress={() => handleStartChat(agent)}
                      onLongPress={() => openActionSheet(agent)}
                      onMore={() => openActionSheet(agent)}
                    />
                  ))}
                </div>
              )}
            </section>

            {/* Deleted agents */}
            {deletedAgents.length > 0 && (
              <section>
                <div className="px-1 mb-2 text-[12px] text-[var(--color-text-sub,#9ca3af)]">
                  {t('agents.mobile.group.deleted')}
                </div>
                <div className="rounded-2xl bg-[var(--color-chat-other-bg,#262626)] overflow-hidden divide-y divide-[var(--color-border-color,rgba(0,0,0,0.05))] dark:divide-[var(--color-border-color,rgba(255,255,255,0.05))]">
                  {deletedAgents.map((agent) => (
                    <AgentRow
                      key={agent.id}
                      agent={agent}
                      showStatusBadge
                      onPress={undefined}
                      onLongPress={() => openActionSheet(agent)}
                      onMore={() => openActionSheet(agent)}
                      trailing={
                        <button
                          type="button"
                          onClick={() => void handleRestore(agent)}
                          className="flex items-center gap-1 shrink-0 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 px-2.5 py-1.5 text-[12px] font-medium active:bg-emerald-500/20 transition-colors"
                        >
                          <RotateCcw className="w-3.5 h-3.5" />
                          {t('agents.mobile.menu.restore')}
                        </button>
                      }
                    />
                  ))}
                </div>
              </section>
            )}

            {/* Load more footer */}
            <div ref={sentinelRef} className="flex justify-center pb-4">
              {loadingMore && (
                <span className="text-[13px] text-[var(--color-text-sub,#9ca3af)]">
                  {t('agents.mobile.loadingMore')}
                </span>
              )}
              {!hasMore && !loadingMore && agents.length > 0 && (
                <button
                  type="button"
                  onClick={() => void loadMore()}
                  className="flex items-center gap-1 text-[13px] text-[var(--color-text-sub,#9ca3af)] active:opacity-70"
                >
                  {t('agents.mobile.noMore')}
                  <ChevronDown className="w-3.5 h-3.5" />
                </button>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Action sheet */}
      <MobileActionSheet
        isOpen={actionAgent !== null}
        onClose={() => setActionAgent(null)}
        options={
          actionAgent
            ? [
                {
                  label: t('agents.mobile.menu.edit'),
                  onClick: () => {
                    if (actionAgent.id && onEditAgent) onEditAgent(actionAgent.id);
                  },
                },
                {
                  label: t('agents.mobile.menu.delete'),
                  danger: true,
                  onClick: () => setConfirmAgent(actionAgent),
                },
              ]
            : []
        }
      />

      {/* Delete confirm */}
      <MobileConfirmDialog
        isOpen={confirmAgent !== null}
        title={t('agents.mobile.confirm.delete.title')}
        description={t('agents.mobile.confirm.delete.desc', {
          name: confirmAgent?.name ?? '',
        })}
        confirmText={t('agents.mobile.confirm.ok')}
        cancelText={t('agents.mobile.confirm.cancel')}
        danger
        onConfirm={() => void handleDelete()}
        onCancel={() => setConfirmAgent(null)}
      />
    </div>
  );
};

const AgentRow: React.FC<{
  agent: AgentConfig;
  showStatusBadge?: boolean;
  onPress?: () => void;
  onLongPress?: () => void;
  onMore?: () => void;
  trailing?: React.ReactNode;
}> = ({ agent, showStatusBadge, onPress, onLongPress, onMore, trailing }) => {
  const avatar = agent.avatar || createDefaultAvatar('agent');
  const longPressTimer = useRef<number | null>(null);
  const longPressTriggered = useRef(false);

  const beginLongPress = () => {
    if (!onLongPress) return;
    longPressTriggered.current = false;
    longPressTimer.current = window.setTimeout(() => {
      longPressTriggered.current = true;
      onLongPress();
    }, 500);
  };
  const cancelLongPress = () => {
    if (longPressTimer.current !== null) {
      window.clearTimeout(longPressTimer.current);
      longPressTimer.current = null;
    }
  };

  return (
    <div
      role="button"
      tabIndex={0}
      className="flex items-center gap-3 px-4 py-3.5 select-none touch-callout-none transition-colors active:bg-black/5 dark:active:bg-white/5 cursor-pointer"
      onClick={() => {
        if (longPressTriggered.current) {
          longPressTriggered.current = false;
          return;
        }
        onPress?.();
      }}
      onPointerDown={beginLongPress}
      onPointerUp={cancelLongPress}
      onPointerLeave={cancelLongPress}
      onContextMenu={(event) => {
        event.preventDefault();
        cancelLongPress();
        onLongPress?.();
      }}
      onKeyDown={(event) => {
        if (event.key === 'Enter' && onPress) onPress();
      }}
    >
      <div className="w-14 h-14 rounded-2xl overflow-hidden bg-black/5 dark:bg-white/10 shrink-0 border border-black/5 dark:border-white/10">
        <img src={avatar} alt={agent.name} className="w-full h-full object-cover" draggable={false} />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <span className="font-medium text-[16px] text-[var(--color-text-main,#111827)] truncate">
            {agent.name}
          </span>
          {showStatusBadge && agent.status && (
            <span
              className={cn(
                'shrink-0 text-[10px] px-1.5 py-0.5 rounded-full font-medium',
                STATUS_BADGE_STYLES[agent.status] ?? STATUS_BADGE_STYLES.active,
              )}
            >
              {statusLabel(agent.status)}
            </span>
          )}
        </div>
        <p className="text-[13px] text-[var(--color-text-sub,#6b7280)] truncate">
          {agent.description || agent.systemPrompt || ''}
        </p>
      </div>
      {trailing}
      {onMore && (
        <button
          type="button"
          aria-label={t('agents.mobile.menu.edit')}
          onClick={(event) => {
            event.stopPropagation();
            onMore();
          }}
          className="shrink-0 w-8 h-8 flex items-center justify-center rounded-full text-[var(--color-text-sub,#9ca3af)] active:bg-black/5 dark:active:bg-white/10"
        >
          <MoreHorizontal className="w-5 h-5" />
        </button>
      )}
    </div>
  );
};
