import React, { useState, useRef, useEffect } from 'react';
import { 
  Grid, 
  Camera, 
  Library, 
  HelpCircle as InfoIcon,
  Lock,
  Unlock,
  Settings2
} from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

interface CanvasTopActionBarProps {
  snapToGrid: boolean;
  onToggleSnapToGrid: () => void;
  showSnapshots: boolean;
  onToggleSnapshots: (val: boolean) => void;
  showTemplates: boolean;
  onToggleTemplates: (val: boolean) => void;
  showHelp: boolean;
  onShowHelp: () => void;
  isReadOnly?: boolean;
  onToggleReadOnly?: () => void;
}

export const CanvasTopActionBar: React.FC<CanvasTopActionBarProps> = ({
  snapToGrid,
  onToggleSnapToGrid,
  showSnapshots,
  onToggleSnapshots,
  showTemplates,
  onToggleTemplates,
  showHelp,
  onShowHelp,
  isReadOnly = false,
  onToggleReadOnly
}) => {
  return (
    <div className="absolute top-6 right-6 z-30 no-export flex flex-col items-end group">
      {/* Trigger Icon */}
      <div className="w-10 h-10 bg-[#1e1e20]/90 border border-white/10 backdrop-blur-md rounded-xl flex items-center justify-center text-zinc-400 hover:text-white hover:border-white/20 shadow-xl cursor-pointer transition-all">
        <Settings2 size={18} />
      </div>

      {/* Popover Panel (visible on group hover) */}
      <div className="flex flex-col items-end gap-2 mt-2 opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all duration-200 origin-top-right transform scale-95 group-hover:scale-100 absolute top-10 right-0 pt-2">
        {onToggleReadOnly && (
          <button
            onClick={onToggleReadOnly}
            className={cn(
              "h-9 px-3 bg-[#1e1e20]/90 border backdrop-blur-md rounded-xl flex items-center gap-1.5 text-xs font-bold shadow-xl cursor-pointer transition-all select-none animate-pulse-subtle whitespace-nowrap",
              isReadOnly 
                ? "border-amber-500/40 text-amber-400 shadow-[0_0_15px_rgba(245,158,11,0.15)] bg-amber-500/5 hover:bg-amber-500/10" 
                : "border-white/10 text-zinc-400 hover:border-white/20 hover:text-white hover:bg-[#252528]"
            )}
            title={isReadOnly ? "解锁编辑模式" : "开启演示评审模式 (只读锁定)"}
          >
            {isReadOnly ? (
              <Lock size={14} className="text-amber-400 animate-bounce-subtle" />
            ) : (
              <Unlock size={14} className="text-zinc-400" />
            )}
            <span>{isReadOnly ? '演示模式: 锁定' : '演示模式: 自由'}</span>
          </button>
        )}

        <button
          onClick={onToggleSnapToGrid}
          className={cn(
            "h-9 px-3 bg-[#1e1e20]/90 border backdrop-blur-md rounded-xl flex items-center gap-1.5 text-xs font-bold shadow-xl cursor-pointer transition-all select-none whitespace-nowrap",
            snapToGrid 
              ? "border-cyan-500/40 text-cyan-400 shadow-[0_0_15px_rgba(6,182,212,0.15)] hover:bg-[#20282d]" 
              : "border-white/10 text-zinc-400 hover:border-white/20 hover:text-white hover:bg-[#252528]"
          )}
          title={snapToGrid ? "网格吸附已开启" : "网格吸附已关闭"}
        >
          <Grid size={14} className={snapToGrid ? "text-cyan-400" : "text-zinc-400"} />
          <span>网格吸附: {snapToGrid ? '开启' : '关闭'}</span>
        </button>

        <button
          onClick={() => {
            const nextVal = !showSnapshots;
            onToggleSnapshots(nextVal);
            if (nextVal) onToggleTemplates(false);
          }}
          className={cn(
            "h-9 px-3 bg-[#1e1e20]/90 border backdrop-blur-md rounded-xl flex items-center gap-1.5 text-xs font-bold shadow-xl cursor-pointer transition-all select-none whitespace-nowrap",
            showSnapshots
              ? "border-amber-500/40 text-amber-400 shadow-[0_0_15px_rgba(245,158,11,0.15)] hover:bg-[#2e2620]"
              : "border-white/10 text-zinc-400 hover:border-white/20 hover:text-white hover:bg-[#252528]"
          )}
          title={showSnapshots ? "隐藏快照面板" : "显示快照面板"}
        >
          <Camera size={14} className={showSnapshots ? "text-amber-400" : "text-zinc-400"} />
          <span>快照书签: {showSnapshots ? '开启' : '关闭'}</span>
        </button>

        <button
          onClick={() => {
            const nextVal = !showTemplates;
            onToggleTemplates(nextVal);
            if (nextVal) onToggleSnapshots(false);
          }}
          className={cn(
            "h-9 px-3 bg-[#1e1e20]/90 border backdrop-blur-md rounded-xl flex items-center gap-1.5 text-xs font-bold shadow-xl cursor-pointer transition-all select-none whitespace-nowrap",
            showTemplates
              ? "border-cyan-500/40 text-cyan-400 shadow-[0_0_15px_rgba(6,182,212,0.15)] hover:bg-[#20282d]"
              : "border-white/10 text-zinc-400 hover:border-white/20 hover:text-white hover:bg-[#252528]"
          )}
          title={showTemplates ? "隐藏模板库" : "显示模板库"}
        >
          <Library size={14} className={showTemplates ? "text-cyan-400" : "text-zinc-400"} />
          <span>模版配置库: {showTemplates ? '开启' : '关闭'}</span>
        </button>

        {!showHelp && (
          <button
            onClick={onShowHelp}
            className="h-9 px-3 bg-[#1e1e20]/90 border border-white/10 hover:border-white/20 hover:text-white text-zinc-400 backdrop-blur-md rounded-xl flex items-center gap-1.5 text-xs font-bold shadow-xl cursor-pointer transition-all whitespace-nowrap"
          >
            <InfoIcon size={14} className="text-cyan-400" />
            <span>使用指南</span>
          </button>
        )}
      </div>
    </div>
  );
};
