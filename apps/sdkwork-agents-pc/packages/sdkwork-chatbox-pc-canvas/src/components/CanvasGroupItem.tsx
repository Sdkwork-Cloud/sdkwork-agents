import React from 'react';
import { CanvasGroup } from '../types';
import { FolderDot, ChevronDown, ChevronRight } from 'lucide-react';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';

interface CanvasGroupItemProps {
  group: CanvasGroup;
  isSelected: boolean;
  onMouseDown: (groupId: string, e: React.MouseEvent) => void;
  onResizeMouseDown: (groupId: string, e: React.MouseEvent) => void;
  onTitleChange: (groupId: string, title: string) => void;
  onColorChange: (groupId: string, color: CanvasGroup['color']) => void;
  onDisband: (groupId: string) => void;
  onToggleCollapse: (groupId: string) => void;
  onContextMenu?: (e: React.MouseEvent) => void;
}

const colorSchemes = {
  cyan: {
    border: 'border-cyan-500/20 group-hover:border-cyan-500/40',
    bg: 'bg-gradient-to-br from-cyan-500/[0.02] to-transparent',
    text: 'text-cyan-400',
    accent: 'bg-cyan-400 shadow-[0_0_8px_rgba(34,211,238,0.5)]',
    activeBorder: 'border-cyan-400 shadow-[0_0_15px_rgba(34,211,238,0.1)]'
  },
  yellow: {
    border: 'border-yellow-500/20 group-hover:border-yellow-500/40',
    bg: 'bg-gradient-to-br from-yellow-500/[0.02] to-transparent',
    text: 'text-yellow-400',
    accent: 'bg-yellow-400 shadow-[0_0_8px_rgba(250,204,21,0.5)]',
    activeBorder: 'border-yellow-400 shadow-[0_0_15px_rgba(250,204,21,0.1)]'
  },
  pink: {
    border: 'border-pink-500/20 group-hover:border-pink-500/40',
    bg: 'bg-gradient-to-br from-pink-500/[0.02] to-transparent',
    text: 'text-pink-400',
    accent: 'bg-pink-400 shadow-[0_0_8px_rgba(244,114,182,0.5)]',
    activeBorder: 'border-pink-400 shadow-[0_0_15px_rgba(244,114,182,0.1)]'
  },
  emerald: {
    border: 'border-emerald-500/20 group-hover:border-emerald-500/40',
    bg: 'bg-gradient-to-br from-emerald-500/[0.02] to-transparent',
    text: 'text-emerald-400',
    accent: 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)]',
    activeBorder: 'border-emerald-400 shadow-[0_0_15px_rgba(52,211,153,0.1)]'
  },
  violet: {
    border: 'border-violet-500/20 group-hover:border-violet-500/40',
    bg: 'bg-gradient-to-br from-violet-500/[0.02] to-transparent',
    text: 'text-violet-400',
    accent: 'bg-violet-400 shadow-[0_0_8px_rgba(167,139,250,0.5)]',
    activeBorder: 'border-violet-400 shadow-[0_0_15px_rgba(167,139,250,0.1)]'
  },
  orange: {
    border: 'border-orange-500/20 group-hover:border-orange-500/40',
    bg: 'bg-gradient-to-br from-orange-500/[0.02] to-transparent',
    text: 'text-orange-400',
    accent: 'bg-orange-400 shadow-[0_0_8px_rgba(251,146,60,0.5)]',
    activeBorder: 'border-orange-400 shadow-[0_0_15px_rgba(251,146,60,0.1)]'
  }
};

export const CanvasGroupItem: React.FC<CanvasGroupItemProps> = ({
  group,
  isSelected,
  onMouseDown,
  onResizeMouseDown,
  onTitleChange,
  onColorChange,
  onDisband,
  onToggleCollapse,
  onContextMenu
}) => {
  const scheme = colorSchemes[group.color || 'cyan'];
  
  const groupX = typeof group.x === 'number' && !isNaN(group.x) ? group.x : 0;
  const groupY = typeof group.y === 'number' && !isNaN(group.y) ? group.y : 0;
  const groupWidth = typeof group.width === 'number' && !isNaN(group.width) ? group.width : 240;
  const groupHeight = typeof group.height === 'number' && !isNaN(group.height) ? group.height : 180;

  return (
    <div
      onContextMenu={onContextMenu}
      style={{
        left: groupX,
        top: groupY,
        width: groupWidth,
        height: group.isCollapsed ? 40 : groupHeight,
        zIndex: 10
      }}
      onMouseDown={(e) => onMouseDown(group.id, e)}
      className={cn(
        "absolute rounded-[28px] border transition-all duration-150 group/group pointer-events-auto",
        group.isCollapsed ? "border-solid shadow-lg bg-black/50" : "border-dashed",
        isSelected ? scheme.activeBorder : scheme.border,
        scheme.bg
      )}
    >
      {/* Group Label / Inline editor */}
      <div 
        className={cn(
          "absolute top-0 left-0 right-0 h-10 px-4 flex items-center justify-between bg-black/40 cursor-grab active:cursor-grabbing transition-all",
          group.isCollapsed ? "rounded-[28px] border-b-0" : "rounded-t-[28px] border-b border-white/5"
        )}
      >
        <div className="flex items-center gap-2">
          {/* Toggle Expand/Collapse Button */}
          <button
            onClick={(e) => { e.stopPropagation(); onToggleCollapse(group.id); }}
            className="p-1 -ml-1 rounded-md hover:bg-white/10 text-zinc-400 hover:text-zinc-100 transition-colors cursor-pointer"
            title={group.isCollapsed ? "展开分组" : "折叠分组"}
          >
            {group.isCollapsed ? (
              <ChevronRight size={14} />
            ) : (
              <ChevronDown size={14} />
            )}
          </button>
          
          <div className={cn("w-2 h-2 rounded-full", scheme.accent)} />
          <input
            type="text"
            value={group.title}
            onChange={(e) => onTitleChange(group.id, e.target.value)}
            className="bg-transparent text-[12px] font-extrabold text-zinc-100 outline-none border-none p-0 focus:bg-white/5 rounded px-1.5 py-0.5 w-48 font-sans"
            onClick={(e) => e.stopPropagation()}
          />
          <span className="text-[10px] text-zinc-500 font-mono font-medium">
            ({group.nodeIds.length} cards)
          </span>
        </div>

        <div className="flex items-center gap-2 opacity-30 group-hover/group:opacity-100 transition-opacity">
          {/* Color Palette selectors */}
          <div className="flex items-center gap-1 bg-[#121214] p-1 rounded-lg border border-white/5">
            {(['cyan', 'yellow', 'pink', 'emerald', 'violet', 'orange'] as const).map(color => (
              <button
                key={color}
                onClick={() => onColorChange(group.id, color)}
                className={cn(
                  "w-2.5 h-2.5 rounded-full hover:scale-125 transition-transform cursor-pointer color-picker-dot",
                  color === 'cyan' && 'bg-cyan-400',
                  color === 'yellow' && 'bg-yellow-400',
                  color === 'pink' && 'bg-pink-400',
                  color === 'emerald' && 'bg-emerald-400',
                  color === 'violet' && 'bg-violet-400',
                  color === 'orange' && 'bg-orange-400',
                  group.color === color && 'ring-1 ring-white scale-110'
                )}
              />
            ))}
          </div>

          <div className="w-[1px] h-3 bg-white/10 mx-1" />

          <button
            onClick={(e) => { e.stopPropagation(); onDisband(group.id); }}
            className="text-[10px] text-zinc-400 hover:text-red-400 bg-white/5 hover:bg-red-500/10 px-2.5 py-0.5 rounded-md transition-colors font-bold"
            title="解散分组"
          >
            解散
          </button>
        </div>
      </div>

      {/* Empty group alert helper */}
      {!group.isCollapsed && group.nodeIds.length === 0 && (
        <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none text-zinc-600">
          <FolderDot size={24} className="opacity-40 mb-1.5" />
          <span className="text-[11px]">拖拽卡片至此即可自动加入</span>
        </div>
      )}
      
      {/* Resizer */}
      {!group.isCollapsed && (
        <div 
          className="absolute -right-2 -bottom-2 w-5 h-5 bg-[#1e1e20] border border-white/10 rounded-full cursor-nwse-resize opacity-0 group-hover/group:opacity-100 transition-opacity shadow-lg flex items-center justify-center z-50 hover:scale-110 hover:border-white/30"
          onMouseDown={(e) => onResizeMouseDown(group.id, e)}
        >
          <div className="w-1.5 h-1.5 rounded-full bg-zinc-400" />
        </div>
      )}
    </div>
  );
};

