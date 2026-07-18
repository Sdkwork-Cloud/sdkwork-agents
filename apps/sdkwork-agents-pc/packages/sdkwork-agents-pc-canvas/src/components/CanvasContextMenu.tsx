import React from 'react';
import { Type, Image, Video, Trash2, Copy, ClipboardPaste, BoxSelect, Download, CopyPlus, StickyNote, FolderMinus } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

export interface CanvasContextMenuProps {
  x: number;
  y: number;
  target: 'canvas' | 'node' | 'group' | 'connection';
  onClose: () => void;
  onAction: (action: string) => void;
  hasClipboardContent?: boolean;
  hasMultipleSelection?: boolean;
  isInGroup?: boolean;
}

export const CanvasContextMenu: React.FC<CanvasContextMenuProps> = ({
  x, y, target, onClose, onAction, hasClipboardContent = false, hasMultipleSelection = false, isInGroup = false
}) => {
  // Prevent click from propagating to canvas
  const handleAction = (e: React.MouseEvent, action: string) => {
    e.stopPropagation();
    onAction(action);
    onClose();
  };

  return (
    <>
      <div className="fixed inset-0 z-40" onContextMenu={(e) => { e.preventDefault(); onClose(); }} onClick={onClose} />
      <div 
        className="fixed z-50 bg-[#1e1e20] border border-white/10 shadow-2xl rounded-xl py-1.5 min-w-[170px] animate-in fade-in duration-100"
        style={{ left: x, top: y }}
        onContextMenu={(e) => e.preventDefault()}
      >
        {target === 'canvas' && (
          <>
            <button onClick={(e) => handleAction(e, 'add-text')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <Type size={14} className="text-cyan-400" />
              新建文本节点
            </button>
            <button onClick={(e) => handleAction(e, 'add-image')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <Image size={14} className="text-yellow-400" />
              新建文生图
            </button>
            <button onClick={(e) => handleAction(e, 'add-video')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <Video size={14} className="text-indigo-400" />
              新建图生视频
            </button>
            <button onClick={(e) => handleAction(e, 'add-sticky')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <StickyNote size={14} className="text-amber-400" />
              新建便签 / 注释
            </button>
            <div className="h-[1px] bg-white/5 my-1" />
            <button onClick={(e) => handleAction(e, 'paste')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <ClipboardPaste size={14} className="text-zinc-500" />
              粘贴 (Ctrl+V)
            </button>
            <div className="h-[1px] bg-white/5 my-1" />
            <button onClick={(e) => handleAction(e, 'select-all')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <BoxSelect size={14} className="text-zinc-500" />
              全选
            </button>
            <div className="h-[1px] bg-white/5 my-1" />
            <button onClick={(e) => handleAction(e, 'zoom-to-fit')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <BoxSelect size={14} className="text-zinc-500" />
              缩放至适应
            </button>
            <button onClick={(e) => handleAction(e, 'export')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <Download size={14} className="text-emerald-400" />
              导出画布
            </button>
            <button onClick={(e) => handleAction(e, 'clear-workspace')} className="w-full text-left px-3 py-1.5 hover:bg-zinc-900 text-rose-400 hover:text-rose-300 text-xs flex items-center gap-2 transition-colors">
              <Trash2 size={14} className="text-rose-400" />
              清空画布
            </button>
          </>
        )}

        {target === 'node' && (
          <>
            {hasMultipleSelection && (
              <>
                <button onClick={(e) => handleAction(e, 'create-group')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
                  <BoxSelect size={14} className="text-cyan-400" />
                  组合分组
                </button>
                <div className="h-[1px] bg-white/5 my-1" />
              </>
            )}
            {isInGroup && !hasMultipleSelection && (
              <>
                <button onClick={(e) => handleAction(e, 'remove-from-group')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
                  <FolderMinus size={14} className="text-orange-400" />
                  移出当前分组
                </button>
                <div className="h-[1px] bg-white/5 my-1" />
              </>
            )}
            <button onClick={(e) => handleAction(e, 'copy')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <Copy size={14} className="text-zinc-400" />
              复制 (Ctrl+C)
            </button>
            <button onClick={(e) => handleAction(e, 'duplicate')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <CopyPlus size={14} className="text-cyan-400" />
              创建副本 / 克隆
            </button>
            <button 
              disabled={!hasClipboardContent}
              onClick={(e) => hasClipboardContent && handleAction(e, 'paste')} 
              className={cn(
                "w-full text-left px-3 py-1.5 text-xs flex items-center gap-2 transition-colors",
                hasClipboardContent 
                  ? "hover:bg-white/5 text-zinc-300 hover:text-white cursor-pointer" 
                  : "text-zinc-600 cursor-not-allowed opacity-50"
              )}
            >
              <ClipboardPaste size={14} className={hasClipboardContent ? "text-yellow-400" : "text-zinc-600"} />
              粘贴 (Ctrl+V)
            </button>
            <div className="h-[1px] bg-white/5 my-1" />
            <button onClick={(e) => handleAction(e, 'delete')} className="w-full text-left px-3 py-1.5 hover:bg-zinc-900 text-rose-400 hover:text-rose-300 text-xs flex items-center gap-2 transition-colors">
              <Trash2 size={14} className="text-rose-400" />
              删除 (Delete)
            </button>
          </>
        )}

        {target === 'group' && (
          <>
            <button onClick={(e) => handleAction(e, 'disband-group')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <BoxSelect size={14} className="text-zinc-500" />
              解散分组
            </button>
            <div className="h-[1px] bg-white/5 my-1" />
            <button onClick={(e) => handleAction(e, 'copy')} className="w-full text-left px-3 py-1.5 hover:bg-white/5 text-zinc-300 hover:text-white text-xs flex items-center gap-2 transition-colors">
              <Copy size={14} className="text-zinc-500" />
              复制 (Ctrl+C)
            </button>
            <div className="h-[1px] bg-white/5 my-1" />
            <button onClick={(e) => handleAction(e, 'delete')} className="w-full text-left px-3 py-1.5 hover:bg-zinc-900 text-rose-400 hover:text-rose-300 text-xs flex items-center gap-2 transition-colors">
              <Trash2 size={14} />
              删除 (Delete)
            </button>
          </>
        )}

        {target === 'connection' && (
          <>
            <button onClick={(e) => handleAction(e, 'delete-conn')} className="w-full text-left px-3 py-1.5 hover:bg-zinc-900 text-rose-400 hover:text-rose-300 text-xs flex items-center gap-2 transition-colors">
              <Trash2 size={14} />
              断开连接
            </button>
          </>
        )}
      </div>
    </>
  );
};
