import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Search, Check, Cpu, Zap, Brain, Sparkles, Globe } from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import { createPortal } from 'react-dom';

export interface SelectModelPopoverProps {
  isOpen: boolean;
  onClose: () => void;
  triggerElement: HTMLElement | null;
  selectedModelId: string;
  onSave: (modelId: string) => void;
}

export const SelectModelPopover: React.FC<SelectModelPopoverProps> = ({
  isOpen,
  onClose,
  triggerElement,
  selectedModelId,
  onSave,
}) => {
  const [activeVendorId, setActiveVendorId] = useState<string>('openai');
  const [searchQuery, setSearchQuery] = useState('');
  const [position, setPosition] = useState({ top: 0, left: 0 });

  const vendors = [
    { id: 'openai', name: 'OpenAI', icon: <Brain size={14} /> },
    { id: 'anthropic', name: 'Anthropic', icon: <Cpu size={14} /> },
    { id: 'google', name: 'Google', icon: <Globe size={14} /> },
    { id: 'deepseek', name: 'DeepSeek', icon: <Zap size={14} /> },
    { id: 'custom', name: '自建', icon: <Sparkles size={14} /> },
  ];

  const models = [
    { id: 'gpt-4o', vendorId: 'openai', name: 'GPT-4o', description: 'OpenAI 最新旗舰模型，高级推理，原生多模态支持。', contextWindow: '128K' },
    { id: 'gpt-4-turbo', vendorId: 'openai', name: 'GPT-4 Turbo', description: '功能强大的模型，具有视觉能力。', contextWindow: '128K' },
    { id: 'gpt-3.5-turbo', vendorId: 'openai', name: 'GPT-3.5 Turbo', description: '速度快，性价比高。', contextWindow: '16K' },
    { id: 'claude-3-opus', vendorId: 'anthropic', name: 'Claude 3 Opus', description: 'Anthropic 性能最强的模型，擅长复杂任务。', contextWindow: '200K' },
    { id: 'claude-3.5-sonnet', vendorId: 'anthropic', name: 'Claude 3.5 Sonnet', description: '极快的速度与卓越的智能水平。', contextWindow: '200K' },
    { id: 'claude-3-haiku', vendorId: 'anthropic', name: 'Claude 3 Haiku', description: '响应最快，最紧凑的模型。', contextWindow: '200K' },
    { id: 'gemini-1.5-pro', vendorId: 'google', name: 'Gemini 1.5 Pro', description: 'Google 最强大的模型，支持超大上下文。', contextWindow: '2M' },
    { id: 'gemini-1.5-flash', vendorId: 'google', name: 'Gemini 1.5 Flash', description: '轻量级、速度极快的模型。', contextWindow: '1M' },
    { id: 'deepseek-chat', vendorId: 'deepseek', name: 'DeepSeek-V2', description: '经济高效，擅长编程与通识。', contextWindow: '128K' },
    { id: 'deepseek-coder', vendorId: 'deepseek', name: 'DeepSeek-Coder', description: '专注于代码生成与补全。', contextWindow: '128K' },
    { id: 'custom-llama-3', vendorId: 'custom', name: 'Llama 3 70B', description: '企业内部私有化部署。', contextWindow: '8K' },
  ];

  useEffect(() => {
    if (isOpen && triggerElement) {
      setSearchQuery('');
      const model = models.find(m => m.name === selectedModelId || m.id === selectedModelId);
      if (model) {
        setActiveVendorId(model.vendorId);
      }
      
      const updatePosition = () => {
        const rect = triggerElement.getBoundingClientRect();
        // Positioning below the trigger
        setPosition({
          top: rect.bottom + 8,
          left: rect.left,
        });
      };
      
      updatePosition();
      window.addEventListener('resize', updatePosition);
      window.addEventListener('scroll', updatePosition, true);
      
      return () => {
        window.removeEventListener('resize', updatePosition);
        window.removeEventListener('scroll', updatePosition, true);
      };
    }
  }, [isOpen, triggerElement, selectedModelId]);

  const filteredModels = models.filter(m => {
    const matchesVendor = m.vendorId === activeVendorId;
    const matchesSearch = !searchQuery.trim() || 
      m.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
      (m.description || '').toLowerCase().includes(searchQuery.toLowerCase());
    return matchesVendor && matchesSearch;
  });

  if (!isOpen) return null;

  return createPortal(
    <>
      <div 
        className="fixed inset-0 z-[100]" 
        onClick={onClose}
      />
    <motion.div
      initial={{ opacity: 0, y: -10, scale: 0.95 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: -10, scale: 0.95 }}
      transition={{ duration: 0.15, ease: "easeOut" }}
      style={{ top: position.top, left: position.left }}
      className="fixed z-[101] w-[560px] bg-[#1c1c1e] rounded-xl shadow-2xl border border-white/10 flex overflow-hidden h-[420px]"
    >
      {/* Left Sidebar: Vendor Tabs */}
      <div className="w-[150px] bg-[#151515] border-r border-white/5 flex flex-col p-2 shrink-0 overflow-y-auto custom-scrollbar gap-1">
        {vendors.map(vendor => (
          <button
            key={vendor.id}
            onClick={() => setActiveVendorId(vendor.id)}
            className={cn(
              "flex items-center gap-2 px-3 py-2.5 rounded-lg text-[13px] font-medium transition-all w-full text-left",
              activeVendorId === vendor.id 
                ? "bg-blue-600/15 border border-blue-500/40 text-blue-400" 
                : "text-gray-400 hover:bg-white/10 hover:text-gray-200 border border-transparent"
            )}
          >
            <div className={cn("text-current opacity-80 shrink-0", activeVendorId === vendor.id ? 'opacity-100' : '')}>
              {vendor.icon}
            </div>
            <span className="truncate">{vendor.name}</span>
          </button>
        ))}
      </div>

      {/* Right Pane: Search & Models List */}
      <div className="flex-1 flex flex-col min-w-0 bg-[#1c1c1e]">
        {/* Search */}
        <div className="p-3 border-b border-white/5 shrink-0">
          <div className="relative">
            <input 
              type="text" 
              placeholder="搜索当前供应商模型..." 
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-[#151515] border border-white/10 rounded-lg pl-8 pr-3 py-1.5 text-[13px] text-gray-200 outline-none focus:border-blue-500 transition-colors shadow-inner"
            />
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500" size={14} />
          </div>
        </div>

        {/* Models List */}
        <div className="flex-1 overflow-y-auto custom-scrollbar p-3">
          {filteredModels.length === 0 ? (
            <div className="text-gray-500 text-[13px] text-center py-10 flex flex-col items-center justify-center">
              <Cpu size={24} className="mb-2 text-gray-600 opacity-50" />
              未找到匹配的模型
            </div>
          ) : (
            <div className="space-y-2">
              {filteredModels.map(model => {
                const isSelected = selectedModelId === model.name || selectedModelId === model.id;
                return (
                  <div
                    key={model.id}
                    onClick={() => {
                      onSave(model.name);
                      onClose();
                    }}
                    className={cn(
                      "relative group rounded-xl p-3 cursor-pointer transition-all flex flex-col",
                      isSelected 
                        ? "bg-blue-500/10 border border-blue-500/30 shadow-sm" 
                        : "bg-[#252528]/50 border border-transparent hover:bg-[#2a2a2d] hover:border-white/5"
                    )}
                  >
                    <div className="flex items-center justify-between mb-1">
                      <h3 className={cn("text-[14px] font-semibold", isSelected ? "text-blue-400" : "text-gray-100 group-hover:text-white transition-colors")}>
                        {model.name}
                      </h3>
                      {isSelected && (
                        <Check size={16} className="text-blue-500" strokeWidth={3} />
                      )}
                    </div>
                    <p className="text-[12px] text-gray-400 leading-relaxed mb-3 line-clamp-2 pr-6">
                      {model.description}
                    </p>
                    <div className="flex items-center gap-2 mt-auto">
                      <div className="bg-[#151515] border border-white/5 px-2 py-0.5 rounded text-[10px] font-mono text-gray-500">
                        CTX: {model.contextWindow}
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </motion.div>
    </>,
    document.body
  );
};
