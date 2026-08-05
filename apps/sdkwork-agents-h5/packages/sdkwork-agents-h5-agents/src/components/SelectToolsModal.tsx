import React, { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { X, Search, Check, Wrench } from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import { loadAgentEngineToolItems, loadMcpCatalogPage } from '../services/RuntimeCatalogService';

export interface ToolItem {
  id: string;
  name: string;
  description: string;
  provider: string;
  icon: React.ReactNode;
  category: 'official' | 'custom' | 'mcp';
}

export interface SelectToolsModalProps {
  isOpen: boolean;
  onClose: () => void;
  selectedToolIds: string[];
  onSave: (toolIds: string[], selectedItems: ToolItem[]) => void;
}

type ToolTab = ToolItem['category'];

export const SelectToolsModal: React.FC<SelectToolsModalProps> = ({
  isOpen,
  onClose,
  selectedToolIds,
  onSave,
}) => {
  const [activeTab, setActiveTab] = useState<ToolTab>('official');
  const [tools, setTools] = useState<ToolItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [page, setPage] = useState(1);
  const [hasMore, setHasMore] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [debouncedQuery, setDebouncedQuery] = useState('');
  const [currentSelection, setCurrentSelection] = useState<string[]>([]);
  const [selectionSnapshots, setSelectionSnapshots] = useState<Map<string, ToolItem>>(new Map());

  useEffect(() => {
    const timer = window.setTimeout(() => setDebouncedQuery(searchQuery.trim()), 300);
    return () => window.clearTimeout(timer);
  }, [searchQuery]);

  const loadTools = useCallback(async (tab: ToolTab, nextPage = 1, append = false, q = '') => {
    if (tab === 'custom') {
      setTools([]);
      setHasMore(false);
      setLoading(false);
      setLoadingMore(false);
      setError(null);
      return;
    }

    if (append) {
      setLoadingMore(true);
    } else {
      setLoading(true);
    }
    setError(null);
    try {
      if (tab === 'official') {
        const items = await loadAgentEngineToolItems();
        setTools(items);
        setPage(1);
        setHasMore(false);
      } else {
        const catalog = await loadMcpCatalogPage(nextPage, undefined, q || undefined);
        setTools((prev) => (append ? [...prev, ...catalog.items] : catalog.items));
        setPage(catalog.page);
        setHasMore(catalog.hasMore);
      }
    } catch (loadError) {
      console.error('Failed to load tools catalog:', loadError);
      setError(loadError instanceof Error ? loadError.message : '无法加载工具目录');
      if (!append) {
        setTools([]);
        setHasMore(false);
      }
    } finally {
      setLoading(false);
      setLoadingMore(false);
    }
  }, []);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    setCurrentSelection(selectedToolIds);
    setSearchQuery('');
    setDebouncedQuery('');
    setActiveTab('official');
  }, [isOpen, selectedToolIds]);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    const query = activeTab === 'mcp' ? debouncedQuery : '';
    void loadTools(activeTab, 1, false, query);
  }, [activeTab, debouncedQuery, isOpen, loadTools]);

  const handleSelect = (item: ToolItem) => {
    setSelectionSnapshots((prev) => {
      const next = new Map(prev);
      next.set(item.id, item);
      return next;
    });
    setCurrentSelection(prev =>
      prev.includes(item.id) ? prev.filter(v => v !== item.id) : [...prev, item.id]
    );
  };

  const visibleItems = tools.filter((item) => {
    if (activeTab === 'custom') {
      return item.category === 'custom';
    }
    if (activeTab === 'mcp') {
      return item.category === 'mcp';
    }
    const matchesSearch = !searchQuery.trim() ||
      item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      item.description.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesSearch && item.category === activeTab;
  });

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100]"
            onClick={onClose}
          />
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 20 }}
            className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[1024px] h-[720px] bg-[#1e1e1e] rounded-2xl shadow-2xl flex flex-col z-[101] overflow-hidden border border-white/10"
          >
            <div className="flex items-center justify-between p-6 border-b border-white/5 bg-[#202020] shrink-0">
              <div>
                <h2 className="text-xl font-bold text-gray-100 mb-1">选择工具与 MCP</h2>
                <p className="text-xs text-gray-400">为智能体配置外部交互能力，增强处理真实世界任务的效率。</p>
              </div>
              <button
                onClick={onClose}
                className="w-10 h-10 flex items-center justify-center rounded-xl hover:bg-white/10 text-gray-400 hover:text-white transition-colors"
              >
                <X size={20} />
              </button>
            </div>

            <div className="flex flex-1 min-h-0">
              <div className="w-[200px] bg-[#151515] border-r border-white/5 py-5 flex flex-col shrink-0">
                <div className="px-5 pb-5">
                  <div className="relative">
                    <input
                      type="text"
                      placeholder="搜索工具..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="w-full bg-[#202020] border border-white/10 rounded-xl pl-9 pr-3 py-2.5 text-sm text-gray-200 outline-none focus:border-emerald-500 transition-colors shadow-inner"
                    />
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={16} />
                  </div>
                </div>

                <div className="flex-1 overflow-y-auto custom-scrollbar px-4 space-y-6">
                  <div>
                    <h3 className="text-[11px] font-semibold text-gray-500 uppercase tracking-wider mb-2 px-4">工具类型</h3>
                    <div className="space-y-1.5">
                      {[
                        { id: 'official' as const, name: '官方预置工具' },
                        { id: 'mcp' as const, name: '第三方 MCP' },
                        { id: 'custom' as const, name: '自建 API (Custom)' },
                      ].map(tab => (
                        <button
                          key={tab.id}
                          onClick={() => setActiveTab(tab.id)}
                          className={cn(
                            "w-full flex items-center justify-between px-4 py-2.5 rounded-xl text-[14px] font-medium transition-all text-left group",
                            activeTab === tab.id
                              ? "bg-emerald-600/10 text-emerald-400"
                              : "text-gray-400 hover:bg-white/5 hover:text-gray-200"
                          )}
                        >
                          {tab.name}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>
              </div>

              <div className="flex-1 overflow-y-auto custom-scrollbar p-8 bg-[#1a1a1a]">
                {loading ? (
                  <div className="text-gray-500 text-sm text-center py-32 flex flex-col items-center justify-center gap-3">
                    <div className="w-6 h-6 border-2 border-emerald-500 border-t-transparent rounded-full animate-spin" />
                    正在加载工具目录...
                  </div>
                ) : error ? (
                  <div className="text-center py-32 flex flex-col items-center justify-center gap-4">
                    <p className="text-sm text-red-400 max-w-md">{error}</p>
                    <button
                      type="button"
                      onClick={() => void loadTools(activeTab, 1, false, activeTab === 'mcp' ? debouncedQuery : '')}
                      className="px-4 py-2 rounded-xl text-sm bg-white/5 hover:bg-white/10 text-gray-200"
                    >
                      重试
                    </button>
                  </div>
                ) : activeTab === 'custom' ? (
                  <div className="text-gray-500 text-sm text-center py-32 flex flex-col items-center justify-center">
                    <Wrench size={32} className="mb-4 text-gray-600 opacity-50" />
                    请通过 MCP Marketplace 接入自建工具服务，完成后可在第三方 MCP 列表选择
                  </div>
                ) : visibleItems.length === 0 ? (
                  <div className="text-gray-500 text-sm text-center py-32 flex flex-col items-center justify-center">
                    <Wrench size={32} className="mb-4 text-gray-600 opacity-50" />
                    没有找到匹配的项
                  </div>
                ) : (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-5 pb-20">
                    {visibleItems.map(item => {
                      const isSelected = currentSelection.includes(item.id);
                      return (
                        <div
                          key={item.id}
                          onClick={() => handleSelect(item)}
                          className={cn(
                            "relative group bg-[#252528] rounded-xl border p-5 cursor-pointer transition-all hover:-translate-y-1 flex flex-col",
                            isSelected
                              ? "border-emerald-500 shadow-md shadow-emerald-500/10 bg-emerald-500/5"
                              : "border-white/5 hover:border-white/20 hover:bg-[#2a2a2d]"
                          )}
                        >
                          {isSelected && (
                            <div className="absolute top-4 right-4 text-emerald-500">
                              <Check size={18} strokeWidth={3} />
                            </div>
                          )}

                          <div className="flex items-center gap-3 mb-3">
                            <div className="w-10 h-10 rounded-lg bg-[#1e1e1e] border border-white/5 flex items-center justify-center shadow-inner shrink-0 group-hover:scale-105 transition-transform">
                              {item.icon}
                            </div>
                            <div className="flex-1 min-w-0 pr-4">
                              <h3 className={cn("text-[15px] font-semibold truncate", isSelected ? "text-emerald-400" : "text-gray-100 group-hover:text-white transition-colors")}>
                                {item.name}
                              </h3>
                              <span className="inline-block mt-1 text-[10px] px-2 py-0.5 rounded-full bg-white/5 border border-white/10 text-gray-400">
                                {item.provider}
                              </span>
                            </div>
                          </div>

                          <p className="text-[13px] text-gray-400 leading-relaxed">
                            {item.description}
                          </p>
                        </div>
                      );
                    })}
                  </div>
                )}
                {!loading && !error && activeTab === 'mcp' && hasMore && (
                  <div className="flex justify-center pb-6">
                    <button
                      type="button"
                      disabled={loadingMore}
                      onClick={() => void loadTools('mcp', page + 1, true, debouncedQuery)}
                      className="px-5 py-2.5 rounded-xl text-sm font-medium bg-white/5 hover:bg-white/10 text-gray-300 transition-colors disabled:opacity-50"
                    >
                      {loadingMore ? '加载中...' : '加载更多'}
                    </button>
                  </div>
                )}
              </div>
            </div>

            <div className="p-5 border-t border-white/5 bg-[#202020] flex items-center justify-between shrink-0">
              <div className="text-sm text-gray-400">
                已选中 <span className="text-emerald-400 font-semibold">{currentSelection.length}</span> 项
              </div>
              <div className="flex items-center gap-3">
                <button
                  onClick={onClose}
                  className="px-5 py-2.5 rounded-xl text-sm font-medium text-gray-300 hover:bg-white/10 transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={() => {
                    const selectedItems = currentSelection
                      .map((id) => selectionSnapshots.get(id))
                      .filter((item): item is ToolItem => Boolean(item));
                    onSave(currentSelection, selectedItems);
                    onClose();
                  }}
                  className="px-6 py-2.5 rounded-xl text-sm font-medium bg-emerald-600 hover:bg-emerald-500 text-white shadow-lg shadow-emerald-500/20 transition-all"
                >
                  确认启用
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
};
