import React, { useState } from 'react';
import { Layers, Keyboard, BookOpen } from 'lucide-react';

interface CanvasHelpPanelProps {
  showHelp: boolean;
  setShowHelp: (show: boolean) => void;
}

export const CanvasHelpPanel: React.FC<CanvasHelpPanelProps> = ({ showHelp, setShowHelp }) => {
  const [activeTab, setActiveTab] = useState<'guide' | 'shortcuts'>('guide');

  if (!showHelp) return null;

  return (
    <div className="absolute top-6 left-6 w-80 bg-[#141416]/95 border border-white/10 backdrop-blur-md p-5 rounded-2xl shadow-2xl z-40 select-none animate-in slide-in-from-left duration-200">
      <div className="flex items-center justify-between border-b border-white/5 pb-3 mb-4">
        <div className="flex items-center gap-2">
          <span className="p-1 rounded bg-cyan-500/10 text-cyan-400">
            <Layers size={14} />
          </span>
          <span className="text-[13px] font-extrabold text-zinc-100 tracking-wide">Octo 流程无限画布</span>
        </div>
        <button 
          onClick={() => setShowHelp(false)}
          className="text-xs text-zinc-500 hover:text-zinc-300 bg-white/5 hover:bg-white/10 px-2 py-1 rounded cursor-pointer transition-colors"
        >
          收起
        </button>
      </div>

      {/* Tabs */}
      <div className="flex bg-black/30 p-1 rounded-xl border border-white/5 mb-4">
        <button
          onClick={() => setActiveTab('guide')}
          className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg text-xs font-medium cursor-pointer transition-all ${
            activeTab === 'guide'
              ? 'bg-zinc-800 text-cyan-400 shadow-sm'
              : 'text-zinc-500 hover:text-zinc-300'
          }`}
        >
          <BookOpen size={13} />
          <span>功能指南</span>
        </button>
        <button
          onClick={() => setActiveTab('shortcuts')}
          className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg text-xs font-medium cursor-pointer transition-all ${
            activeTab === 'shortcuts'
              ? 'bg-zinc-800 text-cyan-400 shadow-sm'
              : 'text-zinc-500 hover:text-zinc-300'
          }`}
        >
          <Keyboard size={13} />
          <span>快捷按键</span>
        </button>
      </div>

      {activeTab === 'guide' ? (
        <div className="space-y-4 text-[12px] leading-relaxed text-zinc-400 animate-in fade-in duration-150">
          <p className="text-zinc-500 text-[11px]">
            高精密度无限创意画布。支持文本大纲创意、AI一键文生图、高画质图生视频的完整剧组链式工作流。
          </p>

          <div className="space-y-3 pt-1">
            <div className="flex items-start gap-2.5">
              <div className="w-5 h-5 rounded bg-cyan-500/10 flex items-center justify-center shrink-0 text-cyan-400 font-bold text-[11px]">1</div>
              <div>
                <h4 className="font-bold text-zinc-200">鼠标左键拖拽圈选 (框选)</h4>
                <p className="text-zinc-500 text-[11px] mt-0.5">在画布空白处按住左键并拖拽，即可拉出选框，批量圈选卡片节点。</p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <div className="w-5 h-5 rounded bg-yellow-500/10 flex items-center justify-center shrink-0 text-yellow-400 font-bold text-[11px]">2</div>
              <div>
                <h4 className="font-bold text-zinc-200">一键组合成新分组 (Miro式编组)</h4>
                <p className="text-zinc-500 text-[11px] mt-0.5">多选后，点击顶部浮标 📦，即可生成自适应的分组框。拖拽分组头部可移动组内所有卡片！</p>
              </div>
            </div>

            <div className="flex items-start gap-2.5">
              <div className="w-5 h-5 rounded bg-emerald-500/10 flex items-center justify-center shrink-0 text-emerald-400 font-bold text-[11px]">3</div>
              <div>
                <h4 className="font-bold text-zinc-200">极致流畅的连线 snapping</h4>
                <p className="text-zinc-500 text-[11px] mt-0.5">拖拽端口圆点连接时，会自动进行磁吸对齐，端口产生呼吸涟漪高亮反馈。</p>
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="space-y-4 text-[11px] text-zinc-400 max-h-[360px] overflow-y-auto pr-1 scrollbar-thin animate-in fade-in duration-150">
          <div className="space-y-2.5">
            <div className="text-[10px] uppercase tracking-wider text-zinc-500 font-bold border-b border-white/5 pb-1">基础操作</div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">撤销 (Undo)</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Ctrl + Z</kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">重做 (Redo)</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Ctrl + Y</kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">删除选中元素</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Delete</kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">重置选择 / 取消</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Esc</kbd>
            </div>
          </div>

          <div className="space-y-2.5 pt-1">
            <div className="text-[10px] uppercase tracking-wider text-zinc-500 font-bold border-b border-white/5 pb-1">卡片编辑</div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">多选卡片 (Shift)</span>
              <span className="text-zinc-500 text-[10px]">按住 Shift + 点击</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">复制卡片 (Copy)</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Ctrl + C</kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">粘贴卡片 (Paste)</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Ctrl + V</kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">克隆复制 (Clone)</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Ctrl + D</kbd>
            </div>
          </div>

          <div className="space-y-2.5 pt-1">
            <div className="text-[10px] uppercase tracking-wider text-zinc-500 font-bold border-b border-white/5 pb-1">视图与画布</div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">拖拽平移画布</span>
              <div className="flex items-center gap-1">
                <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">Space</kbd>
                <span className="text-zinc-500">+ 拖拽</span>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">画布缩放</span>
              <span className="text-zinc-500 text-[10px]">鼠标滚轮 (Wheel)</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">删除连接线</span>
              <span className="text-zinc-500 text-[10px]">双击连接线</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">切换为选择模式</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">V</kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-zinc-300">切换为抓手模式</span>
              <kbd className="px-1.5 py-0.5 font-mono font-bold bg-zinc-800 border border-white/10 rounded text-zinc-300 shadow">H</kbd>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
