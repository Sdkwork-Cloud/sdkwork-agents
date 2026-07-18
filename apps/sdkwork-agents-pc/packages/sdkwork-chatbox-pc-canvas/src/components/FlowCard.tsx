import React, { useState, useRef, useEffect } from 'react';
import { 
  Sparkles, 
  Trash2, 
  Maximize2, 
  Minimize2, 
  Play, 
  Pause, 
  Image, 
  Video, 
  FileText, 
  Heading1, 
  Heading2, 
  Bold, 
  List, 
  ListOrdered,
  RefreshCw,
  Clock,
  ArrowRight,
  Sparkle,
  Layers,
  ChevronDown,
  Box,
  Check,
  Crop,
  PenTool,
  LayoutTemplate,
  Scan,
  RectangleHorizontal,
  StickyNote
} from 'lucide-react';
import { CanvasNode } from '../types';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';
import { CanvasService } from '@/packages/sdkwork-chatbox-pc-core/src/services/CanvasService';

import { useFlowCard } from '../hooks/useFlowCard';
import { ImageGenNodeBody } from './ImageGenNodeBody';
import { VideoGenNodeBody } from './VideoGenNodeBody';
import { NodeToolbar } from './NodeToolbar';

const TiptapRichEditor = React.lazy(() =>
  import('./TiptapRichEditor').then((module) => ({ default: module.TiptapRichEditor })),
);

const stickyColors: Record<string, { bg: string, text: string, border: string, ring: string, headerText: string, iconColor: string, selectionGlow: string }> = {
  yellow: {
    bg: 'bg-[#fef9c3]/95 backdrop-blur-md',
    text: 'text-zinc-800',
    border: 'border-yellow-300/60 hover:border-yellow-400/80',
    ring: 'ring-yellow-400/50',
    headerText: 'text-yellow-900/80',
    iconColor: 'text-yellow-700',
    selectionGlow: 'border-yellow-400/30'
  },
  pink: {
    bg: 'bg-[#fce7f3]/95 backdrop-blur-md',
    text: 'text-zinc-800',
    border: 'border-pink-300/60 hover:border-pink-400/80',
    ring: 'ring-pink-400/50',
    headerText: 'text-pink-900/80',
    iconColor: 'text-pink-700',
    selectionGlow: 'border-pink-400/30'
  },
  cyan: {
    bg: 'bg-[#ecfeff]/95 backdrop-blur-md',
    text: 'text-zinc-800',
    border: 'border-cyan-300/60 hover:border-cyan-400/80',
    ring: 'ring-cyan-400/50',
    headerText: 'text-cyan-900/80',
    iconColor: 'text-cyan-700',
    selectionGlow: 'border-cyan-400/30'
  },
  emerald: {
    bg: 'bg-[#ecfdf5]/95 backdrop-blur-md',
    text: 'text-zinc-800',
    border: 'border-emerald-300/60 hover:border-emerald-400/80',
    ring: 'ring-emerald-400/50',
    headerText: 'text-emerald-900/80',
    iconColor: 'text-emerald-700',
    selectionGlow: 'border-emerald-400/30'
  },
  orange: {
    bg: 'bg-[#fff7ed]/95 backdrop-blur-md',
    text: 'text-zinc-800',
    border: 'border-orange-300/60 hover:border-orange-400/80',
    ring: 'ring-orange-400/50',
    headerText: 'text-orange-900/80',
    iconColor: 'text-orange-700',
    selectionGlow: 'border-orange-400/30'
  },
  purple: {
    bg: 'bg-[#faf5ff]/95 backdrop-blur-md',
    text: 'text-zinc-800',
    border: 'border-purple-300/60 hover:border-purple-400/80',
    ring: 'ring-purple-400/50',
    headerText: 'text-purple-900/80',
    iconColor: 'text-purple-700',
    selectionGlow: 'border-purple-400/30'
  }
};

interface FlowCardProps {
  node: CanvasNode;
  onUpdate: (id: string, updates: Partial<CanvasNode>) => void;
  onDelete: (id: string) => void;
  onDragStart: (id: string, e: React.MouseEvent) => void;
  onPortMouseDown: (id: string, type: 'input' | 'output', e: React.MouseEvent) => void;
  isSelected: boolean;
  onSelect: (id: string, e: React.MouseEvent) => void;
  connectedInputNode?: CanvasNode;
  connectedOutputNodes?: CanvasNode[];
  isSnappingTarget?: boolean;
  isDragging?: boolean;
  onContextMenu?: (e: React.MouseEvent) => void;
  onResizeStart?: (e: React.MouseEvent, direction: 'w' | 'h' | 'both') => void;
}

export const FlowCard: React.FC<FlowCardProps> = ({
  node,
  onUpdate,
  onDelete,
  onDragStart,
  onPortMouseDown,
  isSelected,
  onSelect,
  connectedInputNode,
  connectedOutputNodes,
  isSnappingTarget = false,
  isDragging = false,
  onContextMenu,
  onResizeStart
}) => {
  const {
    isPlaying,
    setIsPlaying,
    videoRef,
    isHovered,
    setIsHovered,
    activeDropdown,
    setActiveDropdown,
    cardDropdownRef,
    handleMouseDown,
    handleVideoPlayToggle,
    triggerGeneration,
    cardRef
  } = useFlowCard(node, onUpdate, onSelect, onDragStart, connectedInputNode);

  const isSticky = node.type === 'sticky';
  const stickyColorConfig = isSticky ? stickyColors[node.color || 'yellow'] : null;

  const nodeWidth = typeof node.width === 'number' && !isNaN(node.width) ? node.width : 260;
  const nodeHeight = typeof node.height === 'number' && !isNaN(node.height) ? node.height : 250;
  const nodeX = typeof node.x === 'number' && !isNaN(node.x) ? node.x : 0;
  const nodeY = typeof node.y === 'number' && !isNaN(node.y) ? node.y : 0;

  return (
    <div
      ref={cardRef}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onMouseDown={handleMouseDown}
      onContextMenu={onContextMenu}
      style={{
        left: nodeX,
        top: nodeY,
        width: nodeWidth,
        height: node.isCollapsed ? undefined : nodeHeight,
        zIndex: isDragging ? 60 : isSelected ? 40 : 20,
      }}
      className={cn(
        "absolute rounded-2xl transition-all duration-150 ease-out flex flex-col group/card select-none overflow-visible pointer-events-auto",
        isSticky 
          ? cn(stickyColorConfig?.bg, "border", stickyColorConfig?.border, "shadow-[0_12px_28px_rgba(0,0,0,0.22)]")
          : "bg-[#111113]/95 backdrop-blur-xl border border-white/5 shadow-[0_15px_35px_rgba(0,0,0,0.55)] hover:border-white/10",
        isDragging
          ? isSticky
            ? cn("scale-[1.03] rotate-[0.8deg] shadow-2xl ring-2", stickyColorConfig?.ring)
            : "border-cyan-400 scale-[1.03] rotate-[0.8deg] shadow-2xl shadow-cyan-500/20 ring-2 ring-cyan-400/40"
          : isSnappingTarget
            ? "border-emerald-400 scale-[1.02] shadow-[0_0_30px_rgba(52,211,153,0.35)] ring-2 ring-emerald-400/50"
            : isSelected
               ? isSticky
                 ? cn("ring-2 scale-[1.01] shadow-xl", stickyColorConfig?.ring)
                 : "border-cyan-400 shadow-[0_0_25px_rgba(34,211,238,0.35),_0_0_50px_rgba(34,211,238,0.15),_0_15px_35px_rgba(0,0,0,0.65)] ring-2 ring-cyan-400/50 scale-[1.01]"
               : ""
      )}
    >
      {/* FLOATING NODE TOOLBAR */}
      {isSelected && !isDragging && (
        <NodeToolbar
          node={node}
          onUpdate={onUpdate}
          onDelete={onDelete}
          triggerGeneration={triggerGeneration}
        />
      )}

      {/* SELECTION GLOW OVERLAY */}
      {isSelected && (
        <div className={cn(
          "absolute inset-0 rounded-2xl border pointer-events-none z-50 animate-pulse duration-[3000ms]",
          isSticky ? stickyColorConfig?.selectionGlow : "border-cyan-400/30"
        )} />
      )}

      {/* 1. INPUT PORT (Left Side) */}
      {!isSticky && (
        <div 
          onMouseDown={(e) => { e.stopPropagation(); onPortMouseDown(node.id, 'input', e); }}
          className="absolute -left-2 top-1/2 -translate-y-1/2 w-4 h-8 flex items-center justify-center cursor-crosshair z-50 group/port pointer-events-auto"
          title="输入端口"
        >
          <div className={cn(
            "port-dot w-2 h-4 rounded-full transition-all duration-150",
            isSnappingTarget 
              ? "bg-emerald-400 scale-y-150 ring-4 ring-emerald-400/40 animate-pulse shadow-[0_0_12px_rgba(52,211,153,0.8)]" 
              : "bg-zinc-600 group-hover/port:bg-emerald-400 group-hover/port:scale-y-125"
          )} />
        </div>
      )}

      {/* 2. OUTPUT PORT (Right Side) */}
      {!isSticky && (
        <div 
          onMouseDown={(e) => { e.stopPropagation(); onPortMouseDown(node.id, 'output', e); }}
          className="absolute -right-2 top-1/2 -translate-y-1/2 w-4 h-8 flex items-center justify-center cursor-crosshair z-50 group/port pointer-events-auto"
          title="输出端口"
        >
          <div className="port-dot w-2 h-4 rounded-full bg-zinc-600 group-hover/port:bg-cyan-400 group-hover/port:scale-y-125 transition-all" />
        </div>
      )}

      {/* NODE HEADER */}
      {isSticky ? (
        <div 
          className={cn(
            "flex items-center justify-between px-3 py-1.5 cursor-grab active:cursor-grabbing border-b border-black/5 transition-colors z-10",
            node.isCollapsed ? "rounded-b-2xl border-b-0" : ""
          )}
          onDoubleClick={(e) => { e.stopPropagation(); onUpdate(node.id, { isCollapsed: !node.isCollapsed }) }}
        >
          <div className="flex items-center gap-1.5 flex-1 mr-2 min-w-0">
            <StickyNote size={12} className={cn("shrink-0", stickyColorConfig?.iconColor)} />
            <input
              type="text"
              value={node.title || ''}
              onChange={(e) => onUpdate(node.id, { title: e.target.value })}
              className={cn(
                "text-[11px] font-bold bg-transparent border-0 focus:ring-0 focus:outline-none p-0 w-full select-text cursor-text no-drag",
                stickyColorConfig?.headerText
              )}
              placeholder="便签标题"
            />
          </div>
          <div className="flex items-center gap-1.5 no-drag shrink-0">
            {/* Color Palette (dot toggles) */}
            <div className="flex items-center gap-1">
              {['yellow', 'pink', 'cyan', 'emerald', 'orange', 'purple'].map((col) => (
                <button
                  key={col}
                  onClick={(e) => { e.stopPropagation(); onUpdate(node.id, { color: col }); }}
                  className={cn(
                    "w-2.5 h-2.5 rounded-full border border-black/10 transition-transform cursor-pointer hover:scale-125",
                    col === 'yellow' ? "bg-yellow-300" :
                    col === 'pink' ? "bg-pink-300" :
                    col === 'cyan' ? "bg-cyan-300" :
                    col === 'emerald' ? "bg-emerald-300" :
                    col === 'orange' ? "bg-orange-300" : "bg-purple-300",
                    node.color === col ? "ring-1 ring-black/50 scale-110" : ""
                  )}
                  title={`设为${col === 'yellow' ? '黄色' : col === 'pink' ? '粉色' : col === 'cyan' ? '蓝色' : col === 'emerald' ? '绿色' : col === 'orange' ? '橙色' : '紫色'}`}
                />
              ))}
            </div>
            
            <button 
              onClick={(e) => { e.stopPropagation(); onUpdate(node.id, { isCollapsed: !node.isCollapsed }) }}
              className="p-1 hover:bg-black/5 rounded-md text-zinc-600 hover:text-zinc-900 transition-colors"
              title={node.isCollapsed ? "展开" : "折叠"}
            >
              {node.isCollapsed ? <Maximize2 size={11} /> : <Minimize2 size={11} />}
            </button>
          </div>
        </div>
      ) : (
        <div 
          className={cn(
            "flex items-center justify-between px-3 py-2 cursor-grab active:cursor-grabbing border-b border-white/5 group-hover/card:border-white/10 transition-colors z-10",
            node.type === 'text' ? "" : "bg-black/40 backdrop-blur-md rounded-t-2xl",
            node.isCollapsed ? "rounded-b-2xl border-b-0" : ""
          )}
          onDoubleClick={(e) => { e.stopPropagation(); onUpdate(node.id, { isCollapsed: !node.isCollapsed }) }}
        >
          <div className="flex items-center gap-2">
            {node.type === 'text' ? <FileText size={12} className={cn("transition-colors duration-150", isSelected ? "text-cyan-400" : "text-zinc-500")} /> : 
             node.type === 'image-gen' ? <Image size={12} className={cn("transition-colors duration-150", isSelected ? "text-cyan-400" : "text-zinc-500")} /> :
             <Video size={12} className={cn("transition-colors duration-150", isSelected ? "text-cyan-400" : "text-zinc-500")} />}
            <span className={cn("text-xs font-medium transition-colors duration-150", isSelected ? "text-cyan-300" : "text-zinc-300")}>{node.title || 'Node'}</span>
          </div>
          <button 
            onClick={(e) => { e.stopPropagation(); onUpdate(node.id, { isCollapsed: !node.isCollapsed }) }}
            className="p-1 hover:bg-white/10 rounded-md text-zinc-500 hover:text-zinc-300 transition-colors no-drag"
            title={node.isCollapsed ? "展开节点" : "折叠节点"}
          >
            {node.isCollapsed ? <Maximize2 size={12} /> : <Minimize2 size={12} />}
          </button>
        </div>
      )}

      {/* CARD BODY CONTENT */}
      {!node.isCollapsed && (
        <div className={cn(
          "flex-1 flex flex-col cursor-grab active:cursor-grabbing",
          node.type === 'text' ? "p-4" : node.type === 'sticky' ? "p-0" : "p-0 rounded-b-2xl overflow-hidden relative"
        )}>
          {/* A. TEXT / DRAFTING CARD */}
          {node.type === 'text' && (
            <div className="flex flex-col flex-1 min-h-[140px] cursor-text">
              <React.Suspense fallback={<div className="min-h-[140px] animate-pulse rounded-lg bg-white/5" />}>
                <TiptapRichEditor
                  content={node.content || ''}
                  onChange={(value) => onUpdate(node.id, { content: value })}
                  nodeId={node.id}
                  mode={node.editorMode || 'preview'}
                  fontStyle={node.fontStyle || 'sans'}
                  showTOC={!!node.showTOC}
                />
              </React.Suspense>
            </div>
          )}

          {/* B. IMAGE GENERATOR CARD */}
          {node.type === 'image-gen' && (
            <ImageGenNodeBody
              node={node}
              connectedInputNode={connectedInputNode}
              onUpdate={onUpdate}
              triggerGeneration={triggerGeneration}
            />
          )}

          {/* C. VIDEO GENERATOR CARD */}
          {node.type === 'video-gen' && (
            <VideoGenNodeBody
              node={node}
              connectedInputNode={connectedInputNode}
              onUpdate={onUpdate}
              triggerGeneration={triggerGeneration}
              videoRef={videoRef}
              isPlaying={isPlaying}
              handleVideoPlayToggle={handleVideoPlayToggle}
            />
          )}

          {/* D. STICKY NOTE CARD BODY */}
          {node.type === 'sticky' && (
            <div className="flex-1 flex flex-col p-3 h-full cursor-text">
              <textarea
                value={node.content || ''}
                onChange={(e) => onUpdate(node.id, { content: e.target.value })}
                placeholder="在此输入便签内容 / 流程标注..."
                className="w-full h-full min-h-[120px] bg-transparent resize-none border-0 focus:ring-0 focus:outline-none text-[11px] font-medium leading-relaxed text-zinc-800 placeholder-zinc-500/50 select-text cursor-text no-drag scrollbar-thin"
              />
            </div>
          )}
        </div>
      )}

      {/* RESIZE HANDLE */}
      {isSelected && !node.isCollapsed && onResizeStart && node.type !== 'image-gen' && node.type !== 'video-gen' && (
        <div
          onMouseDown={(e) => {
            e.stopPropagation();
            onResizeStart(e, 'both');
          }}
          className="absolute bottom-1 right-1 w-4 h-4 cursor-se-resize z-50 flex items-end justify-end p-0.5 group/resize active:scale-95 transition-transform"
          title="拖拽调整节点大小"
        >
          <svg width="8" height="8" viewBox="0 0 8 8" className={cn("transition-colors", isSticky ? "text-black/30 group-hover/resize:text-black/70" : "text-zinc-500 group-hover/resize:text-cyan-400" )}>
            <line x1="6" y1="0" x2="0" y2="6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            <line x1="6" y1="3" x2="3" y2="6" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </div>
      )}

      {/* Corner Handles for Image and Video nodes */}
      {(node.type === 'image-gen' || node.type === 'video-gen') && !node.isCollapsed && (isHovered || isSelected) && (
        <>
          {/* Top-Left: Drag Handle */}
          <div
            onMouseDown={(e) => {
              e.stopPropagation();
              onSelect(node.id, e);
              onDragStart(node.id, e);
            }}
            className={cn(
              "absolute -top-1.5 -left-1.5 w-3 h-3 rounded-full border border-zinc-950/50 shadow-md cursor-grab active:cursor-grabbing z-50 hover:scale-125 transition-transform duration-100",
              isSelected ? "bg-cyan-400 ring-2 ring-cyan-400/30" : "bg-zinc-400 hover:bg-cyan-300"
            )}
            title="按住拖拽移动节点"
          />

          {/* Top-Right: Drag Handle */}
          <div
            onMouseDown={(e) => {
              e.stopPropagation();
              onSelect(node.id, e);
              onDragStart(node.id, e);
            }}
            className={cn(
              "absolute -top-1.5 -right-1.5 w-3 h-3 rounded-full border border-zinc-950/50 shadow-md cursor-grab active:cursor-grabbing z-50 hover:scale-125 transition-transform duration-100",
              isSelected ? "bg-cyan-400 ring-2 ring-cyan-400/30" : "bg-zinc-400 hover:bg-cyan-300"
            )}
            title="按住拖拽移动节点"
          />

          {/* Bottom-Left: Drag Handle */}
          <div
            onMouseDown={(e) => {
              e.stopPropagation();
              onSelect(node.id, e);
              onDragStart(node.id, e);
            }}
            className={cn(
              "absolute -bottom-1.5 -left-1.5 w-3 h-3 rounded-full border border-zinc-950/50 shadow-md cursor-grab active:cursor-grabbing z-50 hover:scale-125 transition-transform duration-100",
              isSelected ? "bg-cyan-400 ring-2 ring-cyan-400/30" : "bg-zinc-400 hover:bg-cyan-300"
            )}
            title="按住拖拽移动节点"
          />

          {/* Right Border Resize Handle Zone */}
          {onResizeStart && (
            <div
              onMouseDown={(e) => {
                e.stopPropagation();
                onSelect(node.id, e);
                onResizeStart(e, 'w');
              }}
              className="absolute -right-1 top-2 bottom-2 w-2.5 cursor-ew-resize z-40 group/r-border flex items-center justify-center"
              title="左右拖拽调整宽度并按比例缩放"
            >
              <div className="w-1 h-1/3 rounded-full bg-cyan-400/0 group-hover/r-border:bg-cyan-400/50 group-active/r-border:bg-cyan-400 transition-colors" />
            </div>
          )}

          {/* Bottom Border Resize Handle Zone */}
          {onResizeStart && (
            <div
              onMouseDown={(e) => {
                e.stopPropagation();
                onSelect(node.id, e);
                onResizeStart(e, 'h');
              }}
              className="absolute -bottom-1 left-2 right-2 h-2.5 cursor-ns-resize z-40 group/b-border flex items-center justify-center"
              title="上下拖拽调整高度并按比例缩放"
            >
              <div className="h-1 w-1/3 rounded-full bg-cyan-400/0 group-hover/b-border:bg-cyan-400/50 group-active/b-border:bg-cyan-400 transition-colors" />
            </div>
          )}

          {/* Bottom-Right: Resize Handle */}
          {onResizeStart && (
            <div
              onMouseDown={(e) => {
                e.stopPropagation();
                onSelect(node.id, e);
                onResizeStart(e, 'both');
              }}
              className={cn(
                "absolute -bottom-1.5 -right-1.5 w-3.5 h-3.5 rounded-full border border-zinc-950/50 shadow-md cursor-se-resize z-50 hover:scale-125 transition-transform duration-100 flex items-center justify-center",
                isSelected ? "bg-cyan-400 ring-2 ring-cyan-400/30 animate-pulse" : "bg-zinc-400 hover:bg-cyan-300"
              )}
              title="拖拽调整节点大小"
            >
              <div className="w-1.5 h-1.5 rounded-full bg-zinc-950 opacity-60" />
            </div>
          )}
        </>
      )}
    </div>
  );
};
