import React, { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Search, Check, Cpu } from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import { createPortal } from 'react-dom';

import type { ModelCatalogItem } from '../services/RuntimeCatalogService';
import { engineKeyToVendorLabel, modelCatalogVendorIcon } from '../services/RuntimeCatalogService';

export interface SelectModelPopoverProps {
  isOpen: boolean;
  onClose: () => void;
  triggerElement: HTMLElement | null;
  selectedModelId: string;
  models: ModelCatalogItem[];
  loading?: boolean;
  onSave: (modelId: string) => void;
}

export const SelectModelPopover: React.FC<SelectModelPopoverProps> = ({
  isOpen,
  onClose,
  triggerElement,
  selectedModelId,
  models,
  loading = false,
  onSave,
}) => {
  const vendors = Array.from(
    models.reduce((map, model) => {
      if (!map.has(model.engineKey)) {
        map.set(model.engineKey, {
          id: model.engineKey,
          name: engineKeyToVendorLabel(model.engineKey),
          icon: modelCatalogVendorIcon(model.engineKey),
        });
      }
      return map;
    }, new Map<string, { id: string; name: string; icon: React.ReactNode }>()).values(),
  );

  const [activeVendorId, setActiveVendorId] = useState<string>('');
  const [searchQuery, setSearchQuery] = useState('');
  const [position, setPosition] = useState({ top: 0, left: 0 });

  useEffect(() => {
    if (isOpen && triggerElement) {
      setSearchQuery('');
      const selected = models.find(
        (model) => model.id === selectedModelId || model.label === selectedModelId,
      );
      if (selected) {
        setActiveVendorId(selected.engineKey);
      } else if (vendors[0]) {
        setActiveVendorId(vendors[0].id);
      }

      const updatePosition = () => {
        const rect = triggerElement.getBoundingClientRect();
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
    return undefined;
  }, [isOpen, triggerElement, selectedModelId, models, vendors]);

  const filteredModels = models.filter((model) => {
    const matchesVendor = !activeVendorId || model.engineKey === activeVendorId;
    const matchesSearch =
      !searchQuery.trim() ||
      model.label.toLowerCase().includes(searchQuery.toLowerCase()) ||
      model.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      model.id.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesVendor && matchesSearch;
  });

  if (!isOpen) return null;

  return createPortal(
    <>
      <div className="fixed inset-0 z-[100]" onClick={onClose} />
      <motion.div
        initial={{ opacity: 0, y: -10, scale: 0.95 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -10, scale: 0.95 }}
        transition={{ duration: 0.15, ease: 'easeOut' }}
        style={{ top: position.top, left: position.left }}
        className="fixed z-[101] w-[560px] bg-[#1c1c1e] rounded-xl shadow-2xl border border-white/10 flex overflow-hidden h-[420px]"
      >
        <div className="w-[150px] bg-[#151515] border-r border-white/5 flex flex-col p-2 shrink-0 overflow-y-auto custom-scrollbar gap-1">
          {vendors.length === 0 ? (
            <div className="px-3 py-2 text-[12px] text-gray-500">无可用引擎</div>
          ) : (
            vendors.map((vendor) => (
              <button
                key={vendor.id}
                onClick={() => setActiveVendorId(vendor.id)}
                className={cn(
                  'flex items-center gap-2 px-3 py-2.5 rounded-lg text-[13px] font-medium transition-all w-full text-left',
                  activeVendorId === vendor.id
                    ? 'bg-blue-600/15 border border-blue-500/40 text-blue-400'
                    : 'text-gray-400 hover:bg-white/10 hover:text-gray-200 border border-transparent',
                )}
              >
                <div
                  className={cn(
                    'text-current opacity-80 shrink-0',
                    activeVendorId === vendor.id ? 'opacity-100' : '',
                  )}
                >
                  {vendor.icon}
                </div>
                <span className="truncate">{vendor.name}</span>
              </button>
            ))
          )}
        </div>

        <div className="flex-1 flex flex-col min-w-0 bg-[#1c1c1e]">
          <div className="p-3 border-b border-white/5 shrink-0">
            <div className="relative">
              <input
                type="text"
                placeholder="搜索运行时模型..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full bg-[#151515] border border-white/10 rounded-lg pl-8 pr-3 py-1.5 text-[13px] text-gray-200 outline-none focus:border-blue-500 transition-colors shadow-inner"
              />
              <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500" size={14} />
            </div>
          </div>

          <div className="flex-1 overflow-y-auto custom-scrollbar p-3">
            {loading ? (
              <div className="text-gray-500 text-[13px] text-center py-10">正在加载模型目录...</div>
            ) : filteredModels.length === 0 ? (
              <div className="text-gray-500 text-[13px] text-center py-10 flex flex-col items-center justify-center">
                <Cpu size={24} className="mb-2 text-gray-600 opacity-50" />
                未配置可用的 agent-engine 模型
              </div>
            ) : (
              <div className="space-y-2">
                {filteredModels.map((model) => {
                  const isSelected =
                    selectedModelId === model.id || selectedModelId === model.label;
                  return (
                    <div
                      key={`${model.engineKey}:${model.id}`}
                      onClick={() => {
                        onSave(model.id);
                        onClose();
                      }}
                      className={cn(
                        'relative group rounded-xl p-3 cursor-pointer transition-all flex flex-col',
                        isSelected
                          ? 'bg-blue-500/10 border border-blue-500/30 shadow-sm'
                          : 'bg-[#252528]/50 border border-transparent hover:bg-[#2a2a2d] hover:border-white/5',
                      )}
                    >
                      <div className="flex items-center justify-between mb-1">
                        <h3
                          className={cn(
                            'text-[14px] font-semibold',
                            isSelected
                              ? 'text-blue-400'
                              : 'text-gray-100 group-hover:text-white transition-colors',
                          )}
                        >
                          {model.label}
                        </h3>
                        {isSelected && <Check size={16} className="text-blue-500" strokeWidth={3} />}
                      </div>
                      <p className="text-[12px] text-gray-400 leading-relaxed mb-2 line-clamp-2 pr-6">
                        {model.description || model.id}
                      </p>
                      <div className="bg-[#151515] border border-white/5 px-2 py-0.5 rounded text-[10px] font-mono text-gray-500 w-fit">
                        {model.id}
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
    document.body,
  );
};
