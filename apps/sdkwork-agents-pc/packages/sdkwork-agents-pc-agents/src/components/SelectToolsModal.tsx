import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { X, Search, Check, Wrench, Search as SearchIcon, Image as ImageIcon, Code2, Calculator, Cloud, Calendar, Globe } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

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
  onSave: (toolIds: string[]) => void;
}

export const AVAILABLE_TOOLS: ToolItem[] = [
  { id: 'web-search', name: '全网搜索', description: '提供实时联网搜索能力，获取最新资讯。', provider: '系统官方', category: 'official', icon: <SearchIcon size={20} className="text-blue-500" /> },
  { id: 'code-runner', name: '代码解释器', description: '在安全的沙箱环境中执行 Python 代码，用于数据分析、图表绘制或数学计算。', provider: '系统官方', category: 'official', icon: <Code2 size={20} className="text-emerald-500" /> },
  { id: 'image-gen', name: 'AI 绘画 (DALLE-3)', description: '根据文本描述生成高质量的图像。', provider: '系统官方', category: 'official', icon: <ImageIcon size={20} className="text-purple-500" /> },
  { id: 'calculator', name: '精准计算器', description: '规避大语言模型计算幻觉，提供精确运算。', provider: '系统官方', category: 'official', icon: <Calculator size={20} className="text-orange-500" /> },
  { id: 'weather', name: '全球天气', description: '获取全球各地的实时天气和预报信息。', provider: '系统官方', category: 'official', icon: <Cloud size={20} className="text-cyan-500" /> },
  { id: 'calendar', name: '日历助手', description: '管理日程、创建会议和提醒。', provider: '系统官方', category: 'official', icon: <Calendar size={20} className="text-red-500" /> },
  { id: 'mcp-github', name: 'GitHub MCP', description: '连接 GitHub 个人账户，读取仓库、提 PR 或审查代码。', provider: '第三方接入 (MCP)', category: 'mcp', icon: <Globe size={20} className="text-gray-300" /> },
  { id: 'custom-api', name: '内部 OA 接口', description: '触发内部企业办公自动化接口，调取员工状态。', provider: '自建 API', category: 'custom', icon: <Wrench size={20} className="text-emerald-400" /> },
];

export const SelectToolsModal: React.FC<SelectToolsModalProps> = ({
  isOpen,
  onClose,
  selectedToolIds,
  onSave,
}) => {
  const [activeTab, setActiveTab] = useState<'official' | 'custom' | 'mcp'>('official');
  const [searchQuery, setSearchQuery] = useState('');
  const [currentSelection, setCurrentSelection] = useState<string[]>([]);

  useEffect(() => {
    if (isOpen) {
      setCurrentSelection(selectedToolIds);
      setSearchQuery('');
    }
  }, [isOpen, selectedToolIds]);

  const handleSelect = (id: string) => {
    setCurrentSelection(prev => 
      prev.includes(id) ? prev.filter(v => v !== id) : [...prev, id]
    );
  };

  const filteredItems = AVAILABLE_TOOLS.filter(item => {
    const matchesSearch = !searchQuery.trim() || 
      item.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
      item.description.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesTab = item.category === activeTab;
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
              {/* Sidebar */}
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
                        { id: 'official', name: '官方预置工具' },
                        { id: 'mcp', name: '第三方 MCP' },
                        { id: 'custom', name: '自建 API (Custom)' },
                      ].map(tab => (
                        <button
                          key={tab.id}
                          onClick={() => setActiveTab(tab.id as any)}
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

              {/* Grid Content */}
              <div className="flex-1 overflow-y-auto custom-scrollbar p-8 bg-[#1a1a1a]">
                {filteredItems.length === 0 ? (
                  <div className="text-gray-500 text-sm text-center py-32 flex flex-col items-center justify-center">
                    <Wrench size={32} className="mb-4 text-gray-600 opacity-50" />
                    没有找到匹配的项
                  </div>
                ) : (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-5 pb-20">
                    {filteredItems.map(item => {
                      const isSelected = currentSelection.includes(item.id);
                      return (
                        <div
                          key={item.id}
                          onClick={() => handleSelect(item.id)}
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
              </div>
            </div>

            {/* Footer */}
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
                    onSave(currentSelection);
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
