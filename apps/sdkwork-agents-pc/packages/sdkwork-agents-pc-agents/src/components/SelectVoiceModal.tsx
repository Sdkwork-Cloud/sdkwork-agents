import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { X, Mic, Radio, Speaker, Headphones, User, Music, Search, Check, Play } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { voiceService, VoiceConfig } from '../services/VoiceService';
import { toast } from './Toast';

export interface SelectVoiceModalProps {
  isOpen: boolean;
  onClose: () => void;
  selectedVoices: string[];
  onSave: (voiceIds: string[]) => void;
  isMulti?: boolean;
}

export const SelectVoiceModal: React.FC<SelectVoiceModalProps> = ({
  isOpen,
  onClose,
  selectedVoices,
  onSave,
  isMulti = false,
}) => {
  const [activeCategory, setActiveCategory] = useState<string>('all');
  const [voices, setVoices] = useState<VoiceConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentSelection, setCurrentSelection] = useState<string[]>([]);

  useEffect(() => {
    if (isOpen) {
      setCurrentSelection(selectedVoices);
      setSearchQuery('');
      setActiveCategory('all');
      loadVoices();
    }
  }, [isOpen, selectedVoices]);

  const loadVoices = async () => {
    setLoading(true);
    try {
      const [market, my] = await Promise.all([
        voiceService.getMarketVoices(),
        voiceService.getMyVoices(),
      ]);
      setVoices([...market, ...my]);
    } catch (error) {
      console.error('Failed to load voices:', error);
    } finally {
      setLoading(false);
    }
  };

  const getIcon = (iconName?: string) => {
    switch (iconName) {
      case 'Mic': return <Mic size={24} />;
      case 'Radio': return <Radio size={24} />;
      case 'Speaker': return <Speaker size={24} />;
      case 'Headphones': return <Headphones size={24} />;
      case 'User': return <User size={24} />;
      default: return <Music size={24} />;
    }
  };

  const categories = [
    { id: 'all', name: '全部' },
    { id: 'reading', name: '阅读' },
    { id: 'news', name: '播报' },
    { id: 'anime', name: '动漫' },
    { id: 'business', name: '客服' },
    { id: 'custom', name: '克隆' }
  ];

  const handleSelect = (id: string) => {
    if (isMulti) {
      setCurrentSelection(prev => 
        prev.includes(id) ? prev.filter(v => v !== id) : [...prev, id]
      );
    } else {
      setCurrentSelection([id]);
    }
  };

  const filteredVoices = voices.filter(v => {
    const matchesSearch = !searchQuery.trim() || 
      v.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
      (v.description || '').toLowerCase().includes(searchQuery.toLowerCase());
    const matchesCategory = activeCategory === 'all' || v.categoryId === activeCategory;
    return matchesSearch && matchesCategory;
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
                <h2 className="text-xl font-bold text-gray-100 mb-1">选择发音人</h2>
                <p className="text-xs text-gray-400">为你的智能体挑选合适的声音模型，支持多维度分类与专业克隆音色。</p>
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
                      placeholder="搜索声音..." 
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                      className="w-full bg-[#202020] border border-white/10 rounded-xl pl-9 pr-3 py-2.5 text-sm text-gray-200 outline-none focus:border-purple-500 transition-colors shadow-inner"
                    />
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" size={16} />
                  </div>
                </div>
                <div className="px-4 space-y-1.5">
                  {categories.map(cat => (
                    <button
                      key={cat.id}
                      onClick={() => setActiveCategory(cat.id)}
                      className={cn(
                        "w-full flex items-center px-4 py-3 rounded-xl text-[14px] font-medium transition-all text-left",
                        activeCategory === cat.id 
                          ? "bg-purple-600 border border-purple-500 box-border text-white shadow-lg shadow-purple-500/20" 
                          : "text-gray-400 hover:bg-white/5 hover:text-gray-200 border border-transparent"
                      )}
                    >
                      {cat.name}
                    </button>
                  ))}
                </div>
              </div>

              {/* Grid Content */}
              <div className="flex-1 overflow-y-auto custom-scrollbar p-8 bg-[#1a1a1a]">
                {loading ? (
                  <div className="text-gray-500 text-sm text-center py-20 flex flex-col items-center justify-center gap-3">
                    <div className="w-6 h-6 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
                    正在同步声音模型市场...
                  </div>
                ) : filteredVoices.length === 0 ? (
                  <div className="text-gray-500 text-sm text-center py-32 flex flex-col items-center justify-center">
                    <Music size={32} className="mb-4 text-gray-600 opacity-50" />
                    未找到符合条件的声音
                  </div>
                ) : (
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-5 pb-20">
                    {filteredVoices.map(voice => {
                      const isSelected = currentSelection.includes(voice.id);
                      return (
                        <div
                          key={voice.id}
                          onClick={() => handleSelect(voice.id)}
                          className={cn(
                            "relative group bg-[#252528] rounded-xl border p-4 cursor-pointer transition-all hover:-translate-y-1",
                            isSelected 
                              ? "border-purple-500 shadow-md shadow-purple-500/10" 
                              : "border-white/5 hover:border-white/20 hover:bg-[#2a2a2d]"
                          )}
                        >
                          {isSelected && (
                            <div className="absolute -top-2 -right-2 w-6 h-6 bg-purple-500 rounded-full flex items-center justify-center text-white shadow-lg z-10 border-2 border-[#252528]">
                              <Check size={12} strokeWidth={3} />
                            </div>
                          )}
                          
                          <div className="flex items-start gap-4">
                            <div className={cn("w-12 h-12 rounded-xl flex flex-shrink-0 items-center justify-center text-white shadow-inner", voice.color || 'bg-purple-500')}>
                              {getIcon(voice.iconName)}
                            </div>
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center justify-between mb-1">
                                <h3 className={cn("font-medium truncate", isSelected ? "text-purple-400" : "text-gray-100 group-hover:text-white transition-colors")}>
                                  {voice.name}
                                </h3>
                              </div>
                              <p className="text-[12px] text-gray-400 line-clamp-2 leading-relaxed">
                                {voice.description}
                              </p>
                            </div>
                          </div>
                          
                          <div className="mt-4 pt-3 border-t border-white/5 flex items-center justify-between">
                            <div className="flex items-center gap-1.5 text-xs text-gray-500">
                              <User size={12} />
                              <span>{voice.author || '我'}</span>
                            </div>
                            <button 
                              onClick={(e) => {
                                e.stopPropagation();
                                if (voice.audioPreview) {
                                  void new Audio(voice.audioPreview).play().catch(() => {
                                    toast('无法播放预览音频', 'error');
                                  });
                                  return;
                                }
                                toast('该音色暂无预览音频', 'info');
                              }}
                              className="w-7 h-7 rounded-full bg-[#181818] border border-white/5 flex items-center justify-center text-gray-400 hover:text-white hover:bg-purple-500 hover:border-purple-500 transition-all opacity-0 group-hover:opacity-100"
                            >
                              <Play size={10} fill="currentColor" className="ml-0.5" />
                            </button>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            </div>

            {/* Footer */}
            <div className="p-4 border-t border-white/5 bg-[#202020] flex items-center justify-between shrink-0">
              <div className="text-sm text-gray-400">
                已选择 <span className="text-purple-400 font-semibold">{currentSelection.length}</span> 个声音 
                {isMulti && <span className="ml-2 text-xs text-gray-500">(支持多选)</span>}
              </div>
              <div className="flex items-center gap-3">
                <button
                  onClick={onClose}
                  className="px-5 py-2 rounded-lg text-sm font-medium text-gray-300 hover:bg-white/10 transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={() => {
                    onSave(currentSelection);
                    onClose();
                  }}
                  className="px-6 py-2 rounded-lg text-sm font-medium bg-purple-600 hover:bg-purple-500 text-white shadow-lg shadow-purple-500/20 transition-all"
                >
                  确认关联
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
};
