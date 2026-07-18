import React from 'react';
import { FolderPlus, Trash2, X } from 'lucide-react';

interface CanvasSelectionToolbarProps {
  selectedCount: number;
  onCreateGroup: () => void;
  onBatchDelete: () => void;
  onClearSelection: () => void;
  onAlignNodes?: (alignment: 'left' | 'center-h' | 'right' | 'top' | 'center-v' | 'bottom' | 'distribute-h' | 'distribute-v') => void;
}

// Figma-style Precise Alignment Icons
const AlignLeftIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M4 2v20" />
    <path d="M8 5h12v4H8z" />
    <path d="M8 15h8v4H8z" />
  </svg>
);

const AlignCenterHIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M12 2v20" />
    <path d="M6 5h12v4H6z" />
    <path d="M8 15h8v4H8z" />
  </svg>
);

const AlignRightIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M20 2v20" />
    <path d="M4 5h12v4H4z" />
    <path d="M8 15h8v4H8z" />
  </svg>
);

const AlignTopIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M2 4h20" />
    <path d="M5 8h4v12H5z" />
    <path d="M15 8h4v8h-4z" />
  </svg>
);

const AlignCenterVIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M2 12h20" />
    <path d="M5 6h4v12H5z" />
    <path d="M15 8h4v8h-4z" />
  </svg>
);

const AlignBottomIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M2 20h20" />
    <path d="M5 4h4v12H5z" />
    <path d="M15 8h4v8h-4z" />
  </svg>
);

const DistributeHIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M4 2v20" />
    <path d="M20 2v20" />
    <rect x="9" y="6" width="6" height="12" rx="1" />
  </svg>
);

const DistributeVIcon = () => (
  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
    <path d="M2 4h20" />
    <path d="M2 20h20" />
    <rect x="6" y="9" width="12" height="6" rx="1" />
  </svg>
);

export const CanvasSelectionToolbar: React.FC<CanvasSelectionToolbarProps> = ({
  selectedCount,
  onCreateGroup,
  onBatchDelete,
  onClearSelection,
  onAlignNodes
}) => {
  if (selectedCount === 0) return null;

  return (
    <div className="absolute top-24 left-1/2 -translate-x-1/2 bg-[#1b1b1d]/95 border border-cyan-500/30 text-zinc-100 backdrop-blur-md px-5 py-3 rounded-2xl shadow-2xl z-50 select-none animate-in fade-in slide-in-from-top-4 duration-200 flex items-center gap-4">
      <div className="flex items-center gap-2">
        <span className="w-2 h-2 rounded-full bg-cyan-400 animate-ping" />
        <span className="text-xs font-extrabold tracking-wide text-zinc-300">
          已选中 <span className="text-cyan-400">{selectedCount}</span> 个工作流节点
        </span>
      </div>

      {selectedCount >= 2 && onAlignNodes && (
        <>
          <div className="w-[1px] h-4 bg-white/10" />
          
          <div className="flex items-center gap-1 bg-black/25 p-1 rounded-xl border border-white/5" title="节点对齐选项">
            <button
              onClick={() => onAlignNodes('left')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="向左对齐"
            >
              <AlignLeftIcon />
            </button>
            <button
              onClick={() => onAlignNodes('center-h')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="水平居中对齐"
            >
              <AlignCenterHIcon />
            </button>
            <button
              onClick={() => onAlignNodes('right')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="向右对齐"
            >
              <AlignRightIcon />
            </button>
            
            <div className="w-[1px] h-3 bg-white/10 mx-1" />

            <button
              onClick={() => onAlignNodes('top')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="向上对齐"
            >
              <AlignTopIcon />
            </button>
            <button
              onClick={() => onAlignNodes('center-v')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="垂直居中对齐"
            >
              <AlignCenterVIcon />
            </button>
            <button
              onClick={() => onAlignNodes('bottom')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="向下对齐"
            >
              <AlignBottomIcon />
            </button>

            <div className="w-[1px] h-3 bg-white/10 mx-1" />

            <button
              onClick={() => onAlignNodes('distribute-h')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="水平均匀分布"
            >
              <DistributeHIcon />
            </button>
            <button
              onClick={() => onAlignNodes('distribute-v')}
              className="w-7 h-7 flex items-center justify-center text-zinc-400 hover:text-cyan-400 hover:bg-white/5 rounded-lg transition-all active:scale-90 cursor-pointer"
              title="垂直均匀分布"
            >
              <DistributeVIcon />
            </button>
          </div>
        </>
      )}

      <div className="w-[1px] h-4 bg-white/10" />

      <div className="flex items-center gap-2">
        <button
          onClick={onCreateGroup}
          className="h-8 px-3.5 bg-cyan-500 hover:bg-cyan-400 text-black text-[11px] font-extrabold rounded-xl flex items-center gap-1.5 shadow-lg active:scale-95 transition-all cursor-pointer"
        >
          <FolderPlus size={13} />
          <span>📦 组合成新分组</span>
        </button>

        <button
          onClick={onBatchDelete}
          className="h-8 px-3.5 bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 border border-rose-500/10 text-[11px] font-extrabold rounded-xl flex items-center gap-1.5 active:scale-95 transition-all cursor-pointer"
        >
          <Trash2 size={13} />
          <span>批量删除</span>
        </button>

        <button
          onClick={onClearSelection}
          className="w-8 h-8 flex items-center justify-center text-zinc-400 hover:text-white hover:bg-white/5 rounded-xl transition-colors cursor-pointer"
          title="取消选择"
        >
          <X size={14} />
        </button>
      </div>
    </div>
  );
};
