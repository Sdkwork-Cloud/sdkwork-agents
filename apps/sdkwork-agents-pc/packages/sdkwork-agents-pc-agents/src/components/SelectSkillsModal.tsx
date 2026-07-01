import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { X, Search, Check, Layers, Network, Activity, GitBranch, KeySquare, Blocks, Briefcase } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

export interface SkillItem {
  id: string;
  name: string;
  description: string;
  provider: string;
  icon: React.ReactNode;
  category: 'workflow' | 'preset';
}

export interface SelectSkillsModalProps {
  isOpen: boolean;
  onClose: () => void;
  selectedSkillIds: string[];
  onSave: (skillIds: string[]) => void;
  skills?: SkillItem[];
}

export const AVAILABLE_SKILLS: SkillItem[] = [
  { id: 'planning', name: '步骤规划 (ReAct)', description: '对复杂任务进行自动拆解并分步执行，提高推理准确性。', provider: '高级心智', category: 'workflow', icon: <Blocks size={20} className="text-cyan-500" /> },
  { id: 'reflection', name: '自我反思 (Reflection)', description: '在得出最终答案前，生成多个草案并选取最优解。', provider: '高级心智', category: 'workflow', icon: <Activity size={20} className="text-pink-500" /> },
  { id: 'multi-route', name: '多模型路由 (Routing)', description: '根据问题难度自动切换 GPT-4o / Claude 3.5 以降低成本。', provider: '成本优化', category: 'workflow', icon: <GitBranch size={20} className="text-violet-500" /> },
  { id: 'multi-agent', name: '多智能体协作 (Swarm)', description: '主模型可创建子 Agent (如分析师、程序员) 并行解决问题。', provider: '高级心智', category: 'workflow', icon: <Network size={20} className="text-blue-500" /> },
  { id: 'cot', name: '思维链 (CoT)', description: '强制要求模型输出完整的内部思考过程后再作答。', provider: '基础心智', category: 'workflow', icon: <KeySquare size={20} className="text-amber-500" /> },
  
  { id: 'domain-expert', name: '行业专家预设', description: '自动加载法律、医疗、金融等行业的默认系统级术语限制。', provider: '特定场景', category: 'preset', icon: <Briefcase size={20} className="text-rose-500" /> }
];

export const SelectSkillsModal: React.FC<SelectSkillsModalProps> = ({
  isOpen,
  onClose,
  selectedSkillIds,
  onSave,
  skills,
}) => {
  const [activeTab, setActiveTab] = useState<'workflow' | 'preset'>('workflow');
  const [searchQuery, setSearchQuery] = useState('');
  const [currentSelection, setCurrentSelection] = useState<string[]>([]);

  useEffect(() => {
    if (isOpen) {
      setCurrentSelection(selectedSkillIds);
      setSearchQuery('');
    }
  }, [isOpen, selectedSkillIds]);

  const handleSelect = (id: string) => {
    setCurrentSelection(prev => 
      prev.includes(id) ? prev.filter(v => v !== id) : [...prev, id]
    );
  };

  const catalog = skills && skills.length > 0 ? skills : AVAILABLE_SKILLS;

  const filteredItems = catalog.filter(item => {
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
                <h2 className="text-xl font-bold text-gray-100 mb-1">选择 Agent Skills</h2>
                <p className="text-xs text-gray-400">赋予智能体更高级的推理、反思和多维度协作能力。</p>
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
                      placeholder="搜索心智..." 
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="w-full bg-[#202020] border border-white/10 rounded-xl pl-9 pr-3 py-2.5 text-sm text-gray-200 outline-none focus:border-cyan-500/50 transition-colors shadow-inner"
                    />
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={16} />
                  </div>
                </div>
                
                <div className="flex-1 overflow-y-auto custom-scrollbar px-4 space-y-6">
                  <div>
                    <h3 className="text-[11px] font-semibold text-gray-500 uppercase tracking-wider mb-2 px-4">技能类型</h3>
                    <div className="space-y-1.5">
                      {[
                        { id: 'workflow', name: '心智与工作流' },
                        { id: 'preset', name: '角色预设' },
                      ].map(tab => (
                        <button
                          key={tab.id}
                          onClick={() => setActiveTab(tab.id as any)}
                          className={cn(
                            "w-full flex items-center justify-between px-4 py-2.5 rounded-xl text-[14px] font-medium transition-all text-left group",
                            activeTab === tab.id 
                              ? "bg-cyan-500/10 text-cyan-400" 
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
                    <Layers size={32} className="mb-4 text-gray-600 opacity-50" />
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
                              ? "border-cyan-500 shadow-md shadow-cyan-500/10 bg-cyan-500/5" 
                              : "border-white/5 hover:border-white/20 hover:bg-[#2a2a2d]"
                          )}
                        >
                          {isSelected && (
                            <div className="absolute top-4 right-4 text-cyan-500">
                              <Check size={18} strokeWidth={3} />
                            </div>
                          )}
                          
                          <div className="flex items-center gap-3 mb-3">
                            <div className="w-10 h-10 rounded-lg bg-[#1e1e1e] border border-white/5 flex items-center justify-center shadow-inner shrink-0 group-hover:scale-105 transition-transform">
                              {item.icon}
                            </div>
                            <div className="flex-1 min-w-0 pr-4">
                              <h3 className={cn("text-[15px] font-semibold truncate", isSelected ? "text-cyan-400" : "text-gray-100 group-hover:text-white transition-colors")}>
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
                已启用 <span className="text-cyan-400 font-semibold">{currentSelection.length}</span> 项技能
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
                  className="px-6 py-2.5 rounded-xl text-sm font-medium bg-cyan-600 hover:bg-cyan-500 text-white shadow-lg shadow-cyan-500/20 transition-all"
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
