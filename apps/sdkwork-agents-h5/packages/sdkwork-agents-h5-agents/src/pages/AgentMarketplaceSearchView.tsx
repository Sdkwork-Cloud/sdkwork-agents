import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Search, X, Flame, Bot } from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import { agentService, type AgentConfig } from '../services/AgentService';
import { toast } from '../components/Toast';
import { t } from '../copy/mobileAgentTexts';
import { MarketAgentCard } from '../components/MarketAgentCard';

export interface AgentMarketplaceSearchViewProps {
  /** Host-navigated "start a chat with this agent" callback. */
  onStartChat?: (agent: AgentConfig) => void;
  /** Host-navigated back; when omitted the cancel button is hidden. */
  onBack?: () => void;
  /** Host toast port; defaults to the built-in agents toast. */
  notify?: (message: string, type?: 'info' | 'success' | 'error') => void;
}

type ViewStatus = 'idle' | 'loading' | 'error' | 'ready';

/**
 * Curated search suggestion chips kept from the original IM H5 agent search
 * design. They are pure suggestions that fill the query box; the result list
 * always comes from the real market search (`agents.list` with `q`).
 */
const HOT_SEARCHES = [
  'agents.mobile.market.hot.coding',
  'agents.mobile.market.hot.writing',
  'agents.mobile.market.hot.english',
  'agents.mobile.market.hot.office',
  'agents.mobile.market.hot.life',
];

const SEARCH_DEBOUNCE_MS = 300;
const PAGE_SIZE = 20;

export const AgentMarketplaceSearchView: React.FC<AgentMarketplaceSearchViewProps> = ({
  onStartChat,
  onBack,
  notify = toast,
}) => {
  const [query, setQuery] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);
  const [status, setStatus] = useState<ViewStatus>('idle');
  const [agents, setAgents] = useState<AgentConfig[]>([]);
  const [searchedQuery, setSearchedQuery] = useState('');

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const runSearch = useCallback(
    async (searchQuery: string) => {
      const trimmed = searchQuery.trim();
      if (!trimmed) {
        setStatus('idle');
        setAgents([]);
        setSearchedQuery('');
        return;
      }
      setStatus('loading');
      setSearchedQuery(trimmed);
      try {
        const result = await agentService.listAgentsPage({
          page: 1,
          pageSize: PAGE_SIZE,
          scope: 'market',
          q: trimmed,
        });
        setAgents(result.items);
        setStatus('ready');
      } catch (error) {
        console.error('Failed to search agents', error);
        notify(t('agents.mobile.market.toast.loadFailed'), 'error');
        setStatus('error');
      }
    },
    [notify],
  );

  // Debounced real backend search on query change.
  const skipInitial = useRef(true);
  useEffect(() => {
    if (skipInitial.current) {
      skipInitial.current = false;
      return undefined;
    }
    const timer = window.setTimeout(() => {
      void runSearch(query);
    }, SEARCH_DEBOUNCE_MS);
    return () => window.clearTimeout(timer);
  }, [query, runSearch]);

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg-color,#f5f5f7)] overflow-hidden">
      {/* Header */}
      <header className="h-[56px] flex items-center px-3 border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] bg-[var(--color-glass-bg,#f5f5f7)] backdrop-blur-xl sticky top-0 z-10 shrink-0 pt-safe gap-3">
        <div className="flex-1 flex items-center bg-[var(--color-chat-other-bg,#262626)] rounded-full h-9 px-3 border border-black/5 dark:border-white/10 transition-colors focus-within:border-[#2b5ce7]">
          <Search className="w-4 h-4 text-gray-400 shrink-0" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder={t('agents.mobile.market.search.placeholder')}
            className="flex-1 bg-transparent border-none outline-none px-2 text-[15px] text-[var(--color-text-main,#111827)] placeholder:text-gray-400 min-w-0"
          />
          {query && (
            <button
              type="button"
              aria-label={t('agents.mobile.market.search.clear')}
              onClick={() => setQuery('')}
              className="p-1 cursor-pointer shrink-0 text-gray-400"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
        <button
          type="button"
          onClick={onBack}
          className="text-[16px] text-[var(--color-primary-blue,#2b5ce7)] font-medium whitespace-nowrap shrink-0 active:opacity-70"
        >
          {t('agents.mobile.market.search.cancel')}
        </button>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        {status === 'idle' && (
          <div className="p-4">
            <div className="flex items-center gap-1.5 mb-4">
              <Flame className="w-4 h-4 text-orange-500" />
              <h3 className="text-[14px] font-bold text-[var(--color-text-main,#111827)]">
                {t('agents.mobile.market.hot.title')}
              </h3>
            </div>
            <div className="flex flex-wrap gap-2.5">
              {HOT_SEARCHES.map((key) => (
                <button
                  key={key}
                  type="button"
                  onClick={() => setQuery(t(key))}
                  className="px-3 py-1.5 bg-[var(--color-chat-other-bg,#262626)] border border-black/5 dark:border-white/10 rounded-full text-[13px] text-gray-700 dark:text-gray-300 cursor-pointer active:bg-black/5 dark:active:bg-white/10 transition-colors"
                >
                  {t(key)}
                </button>
              ))}
            </div>
          </div>
        )}

        {status === 'loading' && (
          <div className="px-4 pt-2 space-y-2" aria-busy="true">
            {Array.from({ length: 4 }, (_, index) => (
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
          <div className="flex flex-col items-center justify-center py-16 px-8">
            <Bot className="w-10 h-10 text-gray-300 dark:text-gray-600 mb-3" />
            <p className="text-[14px] text-[var(--color-text-sub,#6b7280)] mb-4">
              {t('agents.mobile.market.error.desc')}
            </p>
            <button
              type="button"
              onClick={() => void runSearch(searchedQuery)}
              className="px-6 h-10 rounded-full bg-[var(--color-primary-blue,#2b5ce7)] text-white text-[14px] font-medium active:scale-95 transition-transform"
            >
              {t('agents.mobile.market.error.retry')}
            </button>
          </div>
        )}

        {status === 'ready' && (
          <div className="py-2">
            {agents.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 px-8">
                <Bot className="w-10 h-10 text-gray-300 dark:text-gray-600 mb-3" />
                <p className="text-[14px] text-[var(--color-text-sub,#6b7280)]">
                  {t('agents.mobile.market.search.empty')}
                </p>
              </div>
            ) : (
              <div className="bg-[var(--color-chat-other-bg,#262626)] rounded-2xl mx-4 overflow-hidden divide-y divide-[var(--color-border-color,rgba(0,0,0,0.05))] dark:divide-[var(--color-border-color,rgba(255,255,255,0.05))]">
                {agents.map((agent) => (
                  <MarketAgentCard
                    key={agent.id}
                    agent={agent}
                    onStartChat={onStartChat}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
