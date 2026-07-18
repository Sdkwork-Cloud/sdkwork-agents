import React, { useState, useRef, useEffect } from 'react';
import { 
  MousePointer2, 
  Hand, 
  Type, 
  Image, 
  Video, 
  Maximize2, 
  ZoomIn, 
  ZoomOut, 
  Trash2, 
  HelpCircle,
  LayoutGrid,
  Undo2,
  Redo2,
  Share2,
  FileJson,
  FileImage,
  FileText,
  Upload,
  StickyNote,
  Grid,
  GitFork,
  ArrowLeftRight
} from 'lucide-react';
import { CanvasTool } from '../types';
import { cn } from '@sdkwork/agents-pc-commons';

interface ToolbarProps {
  activeTool: CanvasTool;
  setActiveTool: (tool: CanvasTool) => void;
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onResetView: () => void;
  onAutoLayout: (mode: 'hierarchy' | 'grid') => void;
  onClearCanvas: () => void;
  onAddNode: (type: 'text' | 'image-gen' | 'video-gen' | 'sticky') => void;
  setShowHelp: (show: boolean) => void;
  showHelp: boolean;
  onUndo: () => void;
  onRedo: () => void;
  canUndo: boolean;
  canRedo: boolean;
  onExportJSON: () => void;
  onExportPNG: () => void;
  onExportPDF: () => void;
  onImportJSON: (file: File) => void;
  showGrid?: boolean;
  onToggleGrid?: () => void;
  isReadOnly?: boolean;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  activeTool,
  setActiveTool,
  zoom,
  onZoomIn,
  onZoomOut,
  onResetView,
  onAutoLayout,
  onClearCanvas,
  onAddNode,
  setShowHelp,
  showHelp,
  onUndo,
  onRedo,
  canUndo,
  canRedo,
  onExportJSON,
  onExportPNG,
  onExportPDF,
  onImportJSON,
  showGrid = true,
  onToggleGrid,
  isReadOnly = false
}) => {
  const [position, setPosition] = useState<'left' | 'right'>(() => {
    const saved = localStorage.getItem('canvas-toolbar-position');
    return (saved === 'left' || saved === 'right') ? saved : 'right';
  });
  
  const [isExportOpen, setIsExportOpen] = useState(false);
  const [isLayoutOpen, setIsLayoutOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const layoutDropdownRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsExportOpen(false);
      }
      if (layoutDropdownRef.current && !layoutDropdownRef.current.contains(e.target as Node)) {
        setIsLayoutOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      onImportJSON(e.target.files[0]);
      setIsExportOpen(false);
    }
  };

  const togglePosition = () => {
    const nextPos = position === 'left' ? 'right' : 'left';
    setPosition(nextPos);
    localStorage.setItem('canvas-toolbar-position', nextPos);
  };

  // Helper for dynamic tooltip classes based on position
  const getTooltipClass = () => {
    return cn(
      "absolute top-1/2 -translate-y-1/2 bg-[#121214] border border-white/10 text-[10px] text-zinc-300 px-2 py-1.5 rounded-lg shadow-xl opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none z-55",
      position === 'left' ? "left-full ml-3" : "right-full mr-3"
    );
  };

  // Helper for dynamic dropdown layouts
  const getDropdownClass = () => {
    return cn(
      "absolute w-52 bg-[#161618]/95 border border-white/10 backdrop-blur-xl rounded-2xl shadow-[0_12px_32px_rgba(0,0,0,0.6)] p-1.5 flex flex-col gap-1 z-55 animate-in fade-in duration-150",
      position === 'left' 
        ? "left-full ml-3 top-[-10px] origin-left slide-in-from-left-2" 
        : "right-full mr-3 top-[-10px] origin-right slide-in-from-right-2"
    );
  };

  return (
    <div 
      className={cn(
        "absolute top-1/2 -translate-y-1/2 flex flex-col items-center gap-3 bg-[#1e1e20]/90 border border-white/10 backdrop-blur-md p-2 rounded-2xl shadow-2xl z-50 select-none transition-all duration-300",
        position === 'left' ? "left-6" : "right-6"
      )}
    >
      {/* 1. Position Control */}
      <button
        onClick={togglePosition}
        type="button"
        className="w-9 h-9 rounded-xl border border-white/5 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10 flex items-center justify-center cursor-pointer transition-all relative group"
      >
        <ArrowLeftRight size={15} />
        <div className={getTooltipClass()}>
          切换工具栏到{position === 'left' ? "右侧" : "左侧"}
        </div>
      </button>

      <div className="h-[1px] w-6 bg-white/10"></div>

      {/* 2. Interaction Mode */}
      <div className="flex flex-col items-center gap-1 bg-[#151516] p-1 rounded-xl border border-white/5 w-full">
        <button
          onClick={() => setActiveTool('select')}
          type="button"
          className={cn(
            "w-9 h-9 rounded-lg flex items-center justify-center transition-all cursor-pointer relative group",
            activeTool === 'select' ? "bg-cyan-500 text-black shadow-lg" : "text-zinc-400 hover:text-white hover:bg-white/5"
          )}
        >
          <MousePointer2 size={16} fill={activeTool === 'select' ? "currentColor" : "none"} />
          <div className={getTooltipClass()}>
            选择工具 (V)
          </div>
        </button>
        <button
          onClick={() => setActiveTool('hand')}
          type="button"
          className={cn(
            "w-9 h-9 rounded-lg flex items-center justify-center transition-all cursor-pointer relative group",
            activeTool === 'hand' ? "bg-cyan-500 text-black shadow-lg" : "text-zinc-400 hover:text-white hover:bg-white/5"
          )}
        >
          <Hand size={16} fill={activeTool === 'hand' ? "currentColor" : "none"} />
          <div className={getTooltipClass()}>
            抓手工具 (H / 空格键)
          </div>
        </button>
      </div>

      <div className="h-[1px] w-6 bg-white/10"></div>

      {/* 3. Creation Tools */}
      <div className="flex flex-col items-center gap-1.5">
        <button
          onClick={() => onAddNode('text')}
          type="button"
          className="w-9 h-9 rounded-xl border border-white/5 text-zinc-400 hover:text-white bg-white/5 hover:bg-white/10 flex items-center justify-center cursor-pointer transition-all relative group"
        >
          <Type size={16} className="text-cyan-400" />
          <div className={getTooltipClass()}>
            添加文本创意
          </div>
        </button>

        <button
          onClick={() => onAddNode('image-gen')}
          type="button"
          className="w-9 h-9 rounded-xl border border-white/5 text-zinc-400 hover:text-white bg-white/5 hover:bg-white/10 flex items-center justify-center cursor-pointer transition-all relative group"
        >
          <Image size={16} className="text-yellow-400" />
          <div className={getTooltipClass()}>
            添加文生图
          </div>
        </button>

        <button
          onClick={() => onAddNode('video-gen')}
          type="button"
          className="w-9 h-9 rounded-xl border border-white/5 text-zinc-400 hover:text-white bg-white/5 hover:bg-white/10 flex items-center justify-center cursor-pointer transition-all relative group"
        >
          <Video size={16} className="text-indigo-400" />
          <div className={getTooltipClass()}>
            添加图/文生视频
          </div>
        </button>

        <button
          onClick={() => onAddNode('sticky')}
          type="button"
          className="w-9 h-9 rounded-xl border border-white/5 text-zinc-400 hover:text-white bg-white/5 hover:bg-white/10 flex items-center justify-center cursor-pointer transition-all relative group"
        >
          <StickyNote size={16} className="text-amber-400" />
          <div className={getTooltipClass()}>
            添加便签/注释
          </div>
        </button>
      </div>

      <div className="h-[1px] w-6 bg-white/10"></div>

      {/* 4. History Tools */}
      <div className="flex flex-col items-center gap-1 bg-[#151516] p-1 rounded-xl border border-white/5 w-full">
        <button
          onClick={onUndo}
          disabled={!canUndo}
          type="button"
          className="w-8 h-8 rounded-lg flex items-center justify-center text-zinc-400 hover:text-white hover:bg-white/5 disabled:opacity-30 disabled:cursor-not-allowed transition-colors cursor-pointer relative group"
        >
          <Undo2 size={14} />
          <div className={getTooltipClass()}>
            撤销 (Ctrl+Z)
          </div>
        </button>
        <button
          onClick={onRedo}
          disabled={!canRedo}
          type="button"
          className="w-8 h-8 rounded-lg flex items-center justify-center text-zinc-400 hover:text-white hover:bg-white/5 disabled:opacity-30 disabled:cursor-not-allowed transition-colors cursor-pointer relative group"
        >
          <Redo2 size={14} />
          <div className={getTooltipClass()}>
            重做 (Ctrl+Y)
          </div>
        </button>
      </div>

      <div className="h-[1px] w-6 bg-white/10"></div>

      {/* 5. Utility Actions */}
      <div className="flex flex-col items-center gap-1.5">
        {/* Auto Layout Dropdown */}
        <div className="relative" ref={layoutDropdownRef}>
          <button
            onClick={() => setIsLayoutOpen(!isLayoutOpen)}
            type="button"
            className={cn(
              "w-9 h-9 rounded-xl border flex items-center justify-center cursor-pointer transition-all relative group",
              isLayoutOpen
                ? "bg-cyan-500/10 text-cyan-400 border-cyan-500/30 shadow-[0_0_12px_rgba(6,182,212,0.15)]"
                : "border-white/5 bg-white/5 text-zinc-400 hover:text-white hover:border-white/10"
            )}
          >
            <LayoutGrid size={15} />
            {!isLayoutOpen && (
              <div className={getTooltipClass()}>
                自动排版 / 整理
              </div>
            )}
          </button>

          {isLayoutOpen && (
            <div className={getDropdownClass()}>
              <div className="px-3 py-2 text-[10px] font-bold text-zinc-500 uppercase tracking-wider select-none border-b border-white/5 mb-1">
                选择排版布局方式
              </div>
              
              {/* Hierarchical Layout */}
              <button
                onClick={() => {
                  onAutoLayout('hierarchy');
                  setIsLayoutOpen(false);
                }}
                type="button"
                className="w-full px-3 py-2 rounded-xl flex items-center gap-2.5 text-xs text-zinc-300 hover:text-white hover:bg-white/5 transition-all cursor-pointer text-left font-medium"
              >
                <GitFork size={14} className="text-cyan-400 font-normal shrink-0 rotate-90" />
                <div className="flex flex-col min-w-0">
                  <span className="truncate">树状分层排版</span>
                  <span className="text-[9px] text-zinc-500 font-normal truncate">按连接流向左右对齐</span>
                </div>
              </button>

              {/* Grid Layout */}
              <button
                onClick={() => {
                  onAutoLayout('grid');
                  setIsLayoutOpen(false);
                }}
                type="button"
                className="w-full px-3 py-2 rounded-xl flex items-center gap-2.5 text-xs text-zinc-300 hover:text-white hover:bg-white/5 transition-all cursor-pointer text-left font-medium"
              >
                <LayoutGrid size={14} className="text-amber-400 font-normal shrink-0" />
                <div className="flex flex-col min-w-0">
                  <span className="truncate">网格矩阵对齐</span>
                  <span className="text-[9px] text-zinc-500 font-normal truncate">所有卡片等距网格排列</span>
                </div>
              </button>
            </div>
          )}
        </div>

        <button
          onClick={onToggleGrid}
          type="button"
          className={cn(
            "w-9 h-9 rounded-xl border flex items-center justify-center cursor-pointer transition-all relative group",
            showGrid 
              ? "border-cyan-500/10 bg-cyan-500/5 text-cyan-400 hover:text-cyan-300"
              : "border-white/5 bg-white/5 text-zinc-400 hover:text-white"
          )}
        >
          <Grid size={15} />
          <div className={getTooltipClass()}>
            {showGrid ? "隐藏背景网格" : "显示背景网格"}
          </div>
        </button>

        {/* Export / Import Dropdown */}
        <div className="relative" ref={dropdownRef}>
          <button
            onClick={() => setIsExportOpen(!isExportOpen)}
            type="button"
            className={cn(
              "w-9 h-9 rounded-xl border flex items-center justify-center cursor-pointer transition-all relative group",
              isExportOpen
                ? "bg-cyan-500/10 text-cyan-400 border-cyan-500/30 shadow-[0_0_12px_rgba(6,182,212,0.15)]"
                : "border-white/5 bg-white/5 text-zinc-400 hover:text-white hover:border-white/10"
            )}
          >
            <Share2 size={15} />
            {!isExportOpen && (
              <div className={getTooltipClass()}>
                导入 / 导出
              </div>
            )}
          </button>

          {isExportOpen && (
            <div className={getDropdownClass()}>
              <div className="px-3 py-2 text-[10px] font-bold text-zinc-500 uppercase tracking-wider select-none border-b border-white/5 mb-1">
                流程保存与分享
              </div>
              
              {/* Export as JSON */}
              <button
                onClick={() => {
                  onExportJSON();
                  setIsExportOpen(false);
                }}
                type="button"
                className="w-full px-3 py-2 rounded-xl flex items-center gap-2.5 text-xs text-zinc-300 hover:text-white hover:bg-white/5 transition-all cursor-pointer text-left font-medium"
              >
                <FileJson size={14} className="text-cyan-400 font-normal shrink-0" />
                <div className="flex flex-col min-w-0">
                  <span className="truncate">导出为 JSON 备份</span>
                  <span className="text-[9px] text-zinc-500 font-normal truncate">可重新导入并编辑</span>
                </div>
              </button>

              {/* Import JSON */}
              <button
                onClick={() => {
                  fileInputRef.current?.click();
                }}
                type="button"
                className="w-full px-3 py-2 rounded-xl flex items-center gap-2.5 text-xs text-zinc-300 hover:text-white hover:bg-white/5 transition-all cursor-pointer text-left font-medium"
              >
                <Upload size={14} className="text-emerald-400 font-normal shrink-0" />
                <div className="flex flex-col min-w-0">
                  <span className="truncate">导入 JSON 备份</span>
                  <span className="text-[9px] text-zinc-500 font-normal truncate">加载已存流程文件</span>
                </div>
              </button>

              {/* Export as PNG */}
              <button
                onClick={() => {
                  onExportPNG();
                  setIsExportOpen(false);
                }}
                type="button"
                className="w-full px-3 py-2 rounded-xl flex items-center gap-2.5 text-xs text-zinc-300 hover:text-white hover:bg-white/5 transition-all cursor-pointer text-left font-medium"
              >
                <FileImage size={14} className="text-fuchsia-400 font-normal shrink-0" />
                <div className="flex flex-col min-w-0">
                  <span className="truncate">导出为 PNG 图片</span>
                  <span className="text-[9px] text-zinc-500 font-normal truncate">高清全景流程图保存</span>
                </div>
              </button>

              {/* Export as PDF */}
              <button
                onClick={() => {
                  onExportPDF();
                  setIsExportOpen(false);
                }}
                type="button"
                className="w-full px-3 py-2 rounded-xl flex items-center gap-2.5 text-xs text-zinc-300 hover:text-white hover:bg-white/5 transition-all cursor-pointer text-left font-medium"
              >
                <FileText size={14} className="text-rose-400 font-normal shrink-0" />
                <div className="flex flex-col min-w-0">
                  <span className="truncate">导出为多页 PDF</span>
                  <span className="text-[9px] text-zinc-500 font-normal truncate">矢量级精美归档文档</span>
                </div>
              </button>

              <input
                type="file"
                ref={fileInputRef}
                onChange={handleFileChange}
                accept=".json"
                className="hidden"
              />
            </div>
          )}
        </div>

        <button
          onClick={() => setShowHelp(!showHelp)}
          type="button"
          className={cn(
            "w-9 h-9 rounded-xl border flex items-center justify-center cursor-pointer transition-all relative group",
            showHelp 
              ? "bg-cyan-500/10 text-cyan-400 border-cyan-500/20" 
              : "border-white/5 bg-white/5 text-zinc-400 hover:text-white"
          )}
        >
          <HelpCircle size={15} fill={showHelp ? "currentColor" : "none"} fillOpacity={showHelp ? 0.25 : 0} />
          <div className={getTooltipClass()}>
            使用指南
          </div>
        </button>

        <button
          onClick={onClearCanvas}
          type="button"
          className="w-9 h-9 rounded-xl border border-rose-500/10 bg-rose-500/5 text-rose-400 hover:text-white hover:bg-rose-500/20 flex items-center justify-center cursor-pointer transition-all relative group"
        >
          <Trash2 size={15} />
          <div className={getTooltipClass()}>
            清空画布 (不可恢复)
          </div>
        </button>
      </div>
    </div>
  );
};
