import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { X, Search, Check, Database, FileText, Clock } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { knowledgeSelectionService, type KnowledgeBase } from '../services/KnowledgeSelectionService';

export interface SelectKnowledgeModalProps {
  isOpen: boolean;
  onClose: () => void;
  selectedKbIds: string[];
  onSave: (kbIds: string[]) => void;
}

export const SelectKnowledgeModal: React.FC<SelectKnowledgeModalProps> = ({
  isOpen,
  onClose,
  selectedKbIds,
  onSave,
}) => {
  const [activeTab, setActiveTab] = useState<'all' | 'personal' | 'team'>('all');
  const [kbs, setKbs] = useState<KnowledgeBase[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentSelection, setCurrentSelection] = useState<string[]>([]);

  useEffect(() => {
    if (isOpen) {
      setCurrentSelection(selectedKbIds);
      setSearchQuery('');
      setActiveTab('all');
      loadKbs();
    }
  }, [isOpen, selectedKbIds]);

  const loadKbs = async () => {
    setLoading(true);
    try {
      const data = await knowledgeSelectionService.getBases();
      setKbs(data);
    } catch (error) {
      console.error('Failed to load knowledge bases:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = (id: string) => {
    setCurrentSelection(prev => 
      prev.includes(id) ? prev.filter(v => v !== id) : [...prev, id]
    );
  };

  const filteredKbs = kbs.filter(kb => {
    const matchesSearch = !searchQuery.trim() || 
      kb.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
      (kb.description || '').toLowerCase().includes(searchQuery.toLowerCase());
    const matchesTab = activeTab === 'all' || kb.type === activeTab;
    return matchesSearch && matchesTab;
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
            {/* Header */}
            <div className="flex items-center justify-between p-6 border-b border-white/5 bg-[#202020] shrink-0">
              <div>
                <h2 className="text-xl font-bold text-gray-100 mb-1">关联知识库</h2>
                <p className="text-xs text-gray-400">选择要挂载到智能体的结构化语料，增强其生成质量和事实准确性。</p>
              </div>
              <button
                onClick={onClose}
                className="w-10 h-10 flex items-center justify-center rounded-xl hover:bg-white/10 text-gray-400 hover:text-white transition-colors"
              >
                <X size={20} />
              </button>
            </div>

            <div className="flex flex-1 min-h-0">
              {/* Sidebar: Tabs */}
              <div className="w-[180px] bg-[#151515] border-r border-white/5 py-5 flex flex-col shrink-0">
                <div className="px-4 space-y-1.5 flex-1">
                  {[
                    { id: 'all', name: '全部空间' },
                    { id: 'personal', name: '个人知识库' },
                    { id: 'team', name: '团队知识库' },
                  ].map(tab => (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id as any)}
                      className={cn(
                        "w-full flex items-center px-4 py-3 rounded-xl text-[14px] font-medium transition-all text-left",
                        activeTab === tab.id 
                          ? "bg-blue-600/10 border border-blue-500/30 text-blue-400 shadow-sm" 
                          : "text-gray-400 hover:bg-white/5 hover:text-gray-200 border border-transparent"
                      )}
                    >
                      {tab.name}
                    </button>
                  ))}
                </div>
              </div>

              {/* Object List */}
              <div className="flex-1 flex flex-col bg-[#1a1a1a] min-w-0">
                <div className="p-6 pb-4 shrink-0 border-b border-white/5 flex items-center justify-between">
                  <div className="relative w-72">
                    <input 
                      type="text" 
                      placeholder="搜索知识库名称或描述..." 
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="w-full bg-[#202020] border border-white/10 rounded-xl pl-10 pr-4 py-2.5 text-sm text-gray-200 outline-none focus:border-blue-500 transition-colors shadow-inner"
                    />
                    <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500" size={16} />
                  </div>
                  <div className="text-[13px] text-gray-500 font-medium">
                    共 {filteredKbs.length} 个项目
                  </div>
                </div>
                <div className="flex-1 overflow-y-auto custom-scrollbar p-6">
                  {loading ? (
                    <div className="text-gray-500 text-sm text-center py-20 flex flex-col items-center justify-center gap-3">
                      <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                      正在加载知识库列表...
                    </div>
                  ) : filteredKbs.length === 0 ? (
                    <div className="text-gray-500 text-sm text-center py-32 flex flex-col items-center justify-center">
                      <Database size={32} className="mb-4 text-gray-600 opacity-50" />
                      未找到符合条件的知识库
                    </div>
                  ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-5 pb-20">
                      {filteredKbs.map(kb => {
                        const isSelected = currentSelection.includes(kb.id);
                        return (
                          <div
                            key={kb.id}
                            onClick={() => handleSelect(kb.id)}
                            className={cn(
                              "relative group bg-[#252528] rounded-xl border p-5 cursor-pointer transition-all hover:-translate-y-1 flex flex-col h-full",
                              isSelected 
                                ? "border-blue-500 shadow-md shadow-blue-500/10 bg-blue-500/5" 
                                : "border-white/5 hover:border-white/20 hover:bg-[#2a2a2d]"
                            )}
                          >
                            {isSelected && (
                              <div className="absolute top-4 right-4 text-blue-500">
                                <Check size={18} strokeWidth={3} />
                              </div>
                            )}
                            
                            <div className="flex items-start gap-4 mb-4">
                              <div className="w-12 h-12 rounded-xl bg-[#2a2a2d] border border-white/5 flex items-center justify-center text-2xl shadow-inner shrink-0 group-hover:scale-105 transition-transform">
                                {kb.logo}
                              </div>
                              <div className="flex-1 min-w-0 pr-6">
                                <h3 className={cn("text-base font-semibold mb-1 truncate", isSelected ? "text-blue-400" : "text-gray-100 group-hover:text-white transition-colors")}>
                                  {kb.name}
                                </h3>
                                <div className="text-xs text-gray-500 flex items-center gap-1.5 flex-wrap">
                                  <span className={cn("px-1.5 py-0.5 rounded uppercase tracking-wider text-[10px]", kb.type === 'team' ? 'bg-[#00b42a]/10 text-[#00b42a]' : 'bg-purple-500/10 text-purple-400')}>
                                    {kb.type}
                                  </span>
                                </div>
                              </div>
                            </div>
                            
                            <p className="text-[13px] text-gray-400 leading-relaxed mb-4 flex-1 line-clamp-2">
                              {kb.description || '暂无描述'}
                            </p>
                            
                            <div className="mt-auto pt-3 border-t border-white/5 flex items-center justify-between text-xs text-gray-500">
                              <div className="flex items-center gap-1.5">
                                <FileText size={14} /> {kb.count} 篇文档
                              </div>
                              <div className="flex items-center gap-1.5">
                                <Clock size={14} /> {kb.updatedAt ? new Date(kb.updatedAt).toLocaleDateString() : '-'}
                              </div>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Footer */}
            <div className="p-5 border-t border-white/5 bg-[#202020] flex items-center justify-between shrink-0">
              <div className="text-sm text-gray-400">
                已选中 <span className="text-blue-400 font-semibold">{currentSelection.length}</span> 个知识库
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
                    onSave(currentSelection);
                    onClose();
                  }}
                  className="px-6 py-2.5 rounded-xl text-sm font-medium bg-blue-600 hover:bg-blue-500 text-white shadow-lg shadow-blue-500/20 transition-all font-semibold"
                >
                  确认挂载
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
};
