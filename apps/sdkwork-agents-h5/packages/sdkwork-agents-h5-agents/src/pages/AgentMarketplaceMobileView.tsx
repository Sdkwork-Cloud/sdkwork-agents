import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Plus, RefreshCw, Search, Bot } from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import { agentService, type AgentConfig } from '../services/AgentService';
import { toast } from '../components/Toast';
import { t } from '../copy/mobileAgentTexts';
import { MarketAgentCard } from '../components/MarketAgentCard';

export interface AgentMarketplaceMobileViewProps {
  /** Host-navigated "start a chat with this agent" callback. */
  onStartChat?: (agent: AgentConfig) => void;
  /** Host-navigated create entry; when omitted the floating button is hidden. */
  onCreateAgent?: () => void;
  /** Host-navigated search entry; when omitted the header search icon is hidden. */
  onSearch?: () => void;
  /** Host toast port; defaults to the built-in agents toast. */
  notify?: (message: string, type?: 'info' | 'success' | 'error') => void;
}

type ViewStatus = 'loading' | 'error' | 'ready';

/**
 * Curated market categories aligned with the original IM H5 agent tab design.
 * A category tab is shown only when the real market catalog contains agents
 * carrying its `categoryId`; catalog categories without a curated entry are
 * appended dynamically with their raw token as the label.
 */
const DESIGN_CATEGORIES: readonly { id: string; labelKey: string }[] = [
  { id: 'study', labelKey: 'agents.mobile.market.category.study' },
  { id: 'work', labelKey: 'agents.mobile.market.category.work' },
  { id: 'create', labelKey: 'agents.mobile.market.category.create' },
  { id: 'life', labelKey: 'agents.mobile.market.category.life' },
  { id: 'fun', labelKey: 'agents.mobile.market.category.fun' },
  { id: 'emotion', labelKey: 'agents.mobile.market.category.emotion' },
  { id: 'game', labelKey: 'agents.mobile.market.category.game' },
  { id: 'coding', labelKey: 'agents.mobile.market.category.coding' },
  { id: 'drawing', labelKey: 'agents.mobile.market.category.drawing' },
  { id: 'photo', labelKey: 'agents.mobile.market.category.photo' },
  { id: 'device', labelKey: 'agents.mobile.market.category.device' },
];

/** Aliases seen on desktop AgentView category ids → curated design labels. */
const DESIGN_CATEGORY_ALIASES: Readonly<Record<string, string>> = {
  tech: 'coding',
  writing: 'create',
  design: 'drawing',
  office: 'work',
};

function isCuratedCategoryId(categoryId: string): boolean {
  return DESIGN_CATEGORIES.some((category) => category.id === categoryId);
}

/** Normalize backend category tokens onto the curated design ids. */
function normalizeCategoryId(categoryId: string | undefined): string | undefined {
  if (!categoryId) {
    return undefined;
  }
  return DESIGN_CATEGORY_ALIASES[categoryId] ?? categoryId;
}

const ALL_CATEGORY_ID = 'all';
const PAGE_SIZE = 20;

export const AgentMarketplaceMobileView: React.FC<AgentMarketplaceMobileViewProps> = ({
  onStartChat,
  onCreateAgent,
  onSearch,
  notify = toast,
}) => {
  const [status, setStatus] = useState<ViewStatus>('loading');
  const [agents, setAgents] = useState<AgentConfig[]>([]);
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);
  const [activeCategory, setActiveCategory] = useState(ALL_CATEGORY_ID);

  // Pull-to-refresh.
  const [pullDistance, setPullDistance] = useState(0);
  const [refreshing, setRefreshing] = useState(false);
  const pullStartY = useRef<number | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);

  const loadPage = useCallback(
    async (targetPage: number, append: boolean, silent = false) => {
      try {
        const result = await agentService.listAgentsPage({
          page: targetPage,
          pageSize: PAGE_SIZE,
          scope: 'market',
        });
        setAgents((prev) => (append ? [...prev, ...result.items] : result.items));
        setPage(targetPage);
        setHasMore(result.pageInfo.hasMore);
        setStatus('ready');
      } catch (error) {
        console.error('Failed to load agent market', error);
        if (!silent) {
          notify(t('agents.mobile.market.toast.loadFailed'), 'error');
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

  // Category tabs derived from the real catalog: curated design order first,
  // then any categoryId the backend returned that we do not know (raw token).
  const categories = useMemo(() => {
    const presentIds = new Set(
      agents
        .map((agent) => normalizeCategoryId(agent.categoryId))
        .filter((id): id is string => Boolean(id)),
    );
    const curated = DESIGN_CATEGORIES
      .filter((category) => presentIds.has(category.id))
      .map((category) => ({ id: category.id, label: t(category.labelKey) }));
    const dynamic = Array.from(presentIds)
      .filter((id) => !isCuratedCategoryId(id))
      .map((id) => ({ id, label: id }));
    return [
      { id: ALL_CATEGORY_ID, label: t('agents.mobile.market.category.all') },
      ...curated,
      ...dynamic,
    ];
  }, [agents]);

  // Clicking a tab centers it inside the scrollable bar (original tab UI).
  const categoryBarRef = useRef<HTMLDivElement>(null);
  const handleCategoryClick = (categoryId: string) => {
    setActiveCategory(categoryId);
    const container = categoryBarRef.current;
    const element = container?.querySelector(`[data-category="${categoryId}"]`);
    if (container && element instanceof HTMLElement) {
      const scrollPos =
        element.offsetLeft - container.offsetWidth / 2 + element.offsetWidth / 2;
      container.scrollTo({ left: scrollPos, behavior: 'smooth' });
    }
  };

  const visibleAgents =
    activeCategory === ALL_CATEGORY_ID
      ? agents
      : agents.filter((agent) => normalizeCategoryId(agent.categoryId) === activeCategory);

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg-color,#f5f5f7)] overflow-hidden">
      {/* Header with category tabs + search entry */}
      <header className="bg-[var(--color-bg-color,#f5f5f7)] sticky top-0 z-10 shrink-0 pt-safe">
        <div className="flex items-center relative border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))]">
          <div
            ref={categoryBarRef}
            className="flex items-center gap-6 px-4 pr-16 overflow-x-auto flex-1 min-w-0"
            style={{ scrollbarWidth: 'none' }}
          >
            {categories.map((category) => (
              <button
                key={category.id}
                type="button"
                data-category={category.id}
                onClick={() => handleCategoryClick(category.id)}
                className={cn(
                  'shrink-0 py-3 text-[15px] font-medium whitespace-nowrap transition-colors',
                  activeCategory === category.id
                    ? 'font-semibold text-[var(--color-primary-blue,#2b5ce7)]'
                    : 'text-[var(--color-text-sub,#6b7280)]',
                )}
              >
                {category.label}
              </button>
            ))}
          </div>

          {/* Right search icon with fade */}
          <div className="absolute right-0 top-0 bottom-0 flex items-center justify-end w-20 bg-gradient-to-l from-[#f5f5f7] via-[#f5f5f7] to-transparent dark:from-[#111113] dark:via-[#111113] pr-4 pointer-events-none">
            {onSearch && (
              <button
                type="button"
                aria-label={t('agents.mobile.market.search')}
                onClick={onSearch}
                className="pointer-events-auto flex items-center justify-center w-8 h-8 rounded-full text-gray-700 dark:text-gray-200 active:bg-black/5 dark:active:bg-white/10 transition-colors"
              >
                <Search className="w-5 h-5" strokeWidth={2.5} />
              </button>
            )}
          </div>
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

      {/* List content */}
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
                <div className="w-[52px] h-[52px] rounded-full bg-black/5 dark:bg-white/10 shrink-0" />
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
              {t('agents.mobile.market.error.title')}
            </h3>
            <p className="text-[13px] text-[var(--color-text-sub,#6b7280)] mb-6 text-center leading-relaxed">
              {t('agents.mobile.market.error.desc')}
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
              {t('agents.mobile.market.error.retry')}
            </button>
          </div>
        )}

        {status === 'ready' && (
          <div className="pb-[84px]">
            {visibleAgents.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 px-8">
                <div className="w-16 h-16 rounded-3xl bg-[var(--color-chat-other-bg,#262626)] flex items-center justify-center mb-4">
                  <Bot className="w-8 h-8 text-gray-300 dark:text-gray-600" />
                </div>
                <h3 className="text-[16px] font-semibold text-[var(--color-text-main,#111827)] mb-1">
                  {t('agents.mobile.market.empty.title')}
                </h3>
                <p className="text-[13px] text-[var(--color-text-sub,#6b7280)] mb-6 text-center leading-relaxed max-w-[240px]">
                  {t('agents.mobile.market.empty.desc')}
                </p>
                {activeCategory === ALL_CATEGORY_ID && onCreateAgent && (
                  <button
                    type="button"
                    onClick={onCreateAgent}
                    className="px-8 h-11 rounded-full bg-[var(--color-primary-blue,#2b5ce7)] text-white text-[15px] font-medium active:scale-95 transition-transform"
                  >
                    {t('agents.mobile.market.empty.create')}
                  </button>
                )}
              </div>
            ) : (
              <div className="bg-[var(--color-chat-other-bg,#262626)] rounded-2xl mx-4 mt-3 overflow-hidden divide-y divide-[var(--color-border-color,rgba(0,0,0,0.05))] dark:divide-[var(--color-border-color,rgba(255,255,255,0.05))]">
                {visibleAgents.map((agent) => (
                  <MarketAgentCard
                    key={agent.id}
                    agent={agent}
                    onStartChat={onStartChat}
                  />
                ))}
              </div>
            )}

            {/* Load more footer */}
            <div ref={sentinelRef} className="flex justify-center py-4">
              {loadingMore && (
                <span className="text-[13px] text-[var(--color-text-sub,#9ca3af)]">
                  {t('agents.mobile.loadingMore')}
                </span>
              )}
              {!hasMore && !loadingMore && agents.length > 0 && (
                <span className="text-[13px] text-[var(--color-text-sub,#9ca3af)]">
                  {t('agents.mobile.noMore')}
                </span>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Floating create button */}
      {onCreateAgent && (
        <div className="absolute bottom-[calc(68px+env(safe-area-inset-bottom))] left-1/2 -translate-x-1/2 z-20">
          <button
            type="button"
            onClick={onCreateAgent}
            className="flex items-center gap-1.5 bg-[var(--color-primary-blue,#2b5ce7)] text-white px-5 py-3 rounded-full shadow-lg shadow-[var(--color-primary-blue,#2b5ce7)]/25 cursor-pointer active:scale-95 transition-transform"
          >
            <Plus className="w-5 h-5" strokeWidth={2.5} />
            <span className="text-[15px] font-medium">{t('agents.mobile.market.create')}</span>
          </button>
        </div>
      )}
    </div>
  );
};
