import React from 'react';
import { X, Bot } from 'lucide-react';
import { useTranslation } from 'react-i18next';

interface ProjectSettingsModalProps {
  onClose: () => void;
  projectName: string;
}

export const ProjectSettingsModal: React.FC<ProjectSettingsModalProps> = ({ onClose, projectName }) => {
  const { t } = useTranslation('common');

  return (
    <div className="fixed inset-0 bg-black/60 z-[9999] flex items-center justify-center animate-in fade-in duration-200">
      <div 
        className="bg-[#1C1C1E] w-full max-w-[520px] rounded-2xl shadow-2xl flex flex-col border border-white/10 animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-5 border-b border-white/5">
          <h2 className="text-lg font-medium text-white">项目设置</h2>
          <button 
            onClick={onClose}
            className="text-zinc-400 hover:text-white p-1 rounded-md hover:bg-white/10 transition-colors"
          >
            <X size={20} />
          </button>
        </div>
        
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          <div className="space-y-2">
            <label className="text-sm text-zinc-300 font-medium">项目名称</label>
            <div className="flex items-center gap-3 bg-[#111111] border border-white/10 rounded-xl px-4 py-3">
              <Bot size={18} className="text-zinc-400" />
              <input 
                type="text"
                defaultValue={projectName}
                className="bg-transparent border-none outline-none text-white text-[15px] flex-1"
              />
            </div>
          </div>
          
          <div className="space-y-2">
            <label className="text-sm text-zinc-300 font-medium block">指令</label>
            <span className="text-xs text-zinc-500 block mb-2">设置此项目的背景信息并自定义 ChatGPT 的回复方式。</span>
            <textarea 
              className="w-full bg-[#111111] border border-white/10 rounded-xl px-4 py-3 text-white text-[14px] min-h-[100px] outline-none focus:border-indigo-500/50 transition-colors resize-y placeholder:text-zinc-600"
              placeholder="例如“用西班牙语回答。参考最新的 JavaScript 文档。回答要简短且突出重点。”"
            />
          </div>
          
          <div className="space-y-2">
            <label className="text-sm text-zinc-300 font-medium block">记忆</label>
            <div className="w-full bg-[#111111] border border-white/10 rounded-xl px-4 py-3 text-white text-[14px]">
              默认
            </div>
            <span className="text-xs text-zinc-500 block mt-2">该项目可以访问外部聊天的记忆，反之亦然。此设置无法更改。</span>
          </div>
          
          <div className="space-y-2">
            <label className="text-sm text-zinc-300 font-medium block">库访问权限</label>
            <div className="relative">
              <select className="w-full bg-[#111111] border border-white/10 rounded-xl px-4 py-3 text-white text-[14px] outline-none appearance-none cursor-pointer">
                <option value="enabled">已启用</option>
                <option value="disabled">已禁用</option>
              </select>
              <div className="absolute right-4 top-1/2 -translate-y-1/2 pointer-events-none">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-zinc-400"><polyline points="6 9 12 15 18 9"></polyline></svg>
              </div>
            </div>
            <span className="text-xs text-zinc-500 block mt-2">此项目在保持私有状态时可访问你的文件库。共享此项目将禁用库访问权限。</span>
          </div>
          
          <div className="pt-2">
            <button className="px-4 py-2 rounded-xl border border-red-500/30 text-red-400 hover:bg-red-500/10 transition-colors text-[14px] font-medium">
              删除项目
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
