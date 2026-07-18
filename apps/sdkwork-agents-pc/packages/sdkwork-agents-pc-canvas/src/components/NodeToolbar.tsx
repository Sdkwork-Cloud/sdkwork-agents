import React, { useState } from 'react';
import { 
  Eye, 
  Edit3, 
  Bold, 
  Italic, 
  Code, 
  Quote, 
  Trash2, 
  Type, 
  Menu, 
  Check, 
  Sparkles, 
  Image as ImageIcon, 
  Video as VideoIcon,
  Play,
  Maximize2
} from 'lucide-react';
import { CanvasNode } from '../types';
import { cn } from '@sdkwork/agents-pc-commons';

const stickyColors: Record<string, { bg: string, ring: string, label: string }> = {
  yellow: { bg: 'bg-[#fef9c3]', ring: 'ring-yellow-400/50', label: '黄色' },
  pink: { bg: 'bg-[#fce7f3]', ring: 'ring-pink-400/50', label: '粉色' },
  cyan: { bg: 'bg-[#ecfeff]', ring: 'ring-cyan-400/50', label: '蓝色' },
  emerald: { bg: 'bg-[#ecfdf5]', ring: 'ring-emerald-400/50', label: '绿色' },
  orange: { bg: 'bg-[#fff7ed]', ring: 'ring-orange-400/50', label: '橙色' },
  purple: { bg: 'bg-[#faf5ff]', ring: 'ring-purple-400/50', label: '紫色' }
};

interface NodeToolbarProps {
  node: CanvasNode;
  onUpdate: (id: string, updates: Partial<CanvasNode>) => void;
  onDelete: (id: string) => void;
  triggerGeneration?: () => void;
}

export const NodeToolbar: React.FC<NodeToolbarProps> = ({
  node,
  onUpdate,
  onDelete,
  triggerGeneration
}) => {
  const [showFontDropdown, setShowFontDropdown] = useState(false);

  const applyFormat = (prefix: string, suffix: string = '') => {
    const textarea = document.getElementById(`textarea-${node.id}`) as HTMLTextAreaElement;
    if (!textarea) return;
    const val = textarea.value;
    const selectionStart = textarea.selectionStart;
    const selectionEnd = textarea.selectionEnd;
    const selectedText = val.substring(selectionStart, selectionEnd);
    const replacement = prefix + selectedText + suffix;
    
    onUpdate(node.id, { content: val.slice(0, selectionStart) + replacement + val.slice(selectionEnd) });
    
    setTimeout(() => {
      textarea.focus();
      const newCursor = selectionStart + prefix.length + selectedText.length;
      textarea.setSelectionRange(newCursor, newCursor);
    }, 50);
  };

  const currentMode = node.editorMode || 'preview';
  const currentFont = node.fontStyle || 'sans';
  const currentShowTOC = !!node.showTOC;

  // Render different tools based on node.type
  return (
    <div 
      onMouseDown={(e) => e.stopPropagation()}
      className={cn(
        "absolute bottom-full left-0 mb-3.5 flex items-center gap-1.5 p-1 rounded-xl shadow-[0_10px_30px_rgba(0,0,0,0.6)] z-50 no-drag select-none whitespace-nowrap animate-in fade-in slide-in-from-bottom-2 duration-150 border",
        node.type === 'sticky'
          ? "bg-white/95 backdrop-blur-md border-zinc-200 text-zinc-800"
          : "bg-zinc-950/95 backdrop-blur-xl border-white/10 text-zinc-100"
      )}
    >
      {/* ========================================================
          A. TEXT NODE TOOLBAR
          ======================================================== */}
      {node.type === 'text' && (
        <>
          {/* Mode Toggle */}
          <div className="flex items-center p-0.5 rounded-lg border bg-black/40 border-white/5">
            <button
              onClick={() => onUpdate(node.id, { editorMode: 'preview' })}
              className={cn(
                "px-2 py-1 rounded-md text-[10px] font-bold transition-all flex items-center gap-1 cursor-pointer",
                currentMode === 'preview' 
                  ? "bg-cyan-500 text-black font-extrabold shadow-sm" 
                  : "text-zinc-400 hover:text-white"
              )}
            >
              <Eye size={10} />
              <span>预览</span>
            </button>
            <button
              onClick={() => onUpdate(node.id, { editorMode: 'edit' })}
              className={cn(
                "px-2 py-1 rounded-md text-[10px] font-bold transition-all flex items-center gap-1 cursor-pointer",
                currentMode === 'edit' 
                  ? "bg-cyan-500 text-black font-extrabold shadow-sm" 
                  : "text-zinc-400 hover:text-white"
              )}
            >
              <Edit3 size={10} />
              <span>编辑</span>
            </button>
          </div>

          <div className="w-[1px] h-4 bg-white/10" />

          {/* Text formatting options (Enabled when in edit mode) */}
          {currentMode === 'edit' && (
            <div className="flex items-center gap-0.5">
              <button
                onClick={() => applyFormat('**', '**')}
                className="p-1.5 rounded-md transition-colors cursor-pointer hover:bg-white/10 text-zinc-400 hover:text-white"
                title="加粗"
              >
                <Bold size={11} />
              </button>
              <button
                onClick={() => applyFormat('*', '*')}
                className="p-1.5 rounded-md transition-colors cursor-pointer hover:bg-white/10 text-zinc-400 hover:text-white"
                title="斜体"
              >
                <Italic size={11} />
              </button>
              <button
                onClick={() => applyFormat('`', '`')}
                className="p-1.5 rounded-md transition-colors cursor-pointer hover:bg-white/10 text-zinc-400 hover:text-white"
                title="行内代码"
              >
                <Code size={11} />
              </button>
              <button
                onClick={() => applyFormat('> ')}
                className="p-1.5 rounded-md transition-colors cursor-pointer hover:bg-white/10 text-zinc-400 hover:text-white"
                title="引用段落"
              >
                <Quote size={11} />
              </button>
            </div>
          )}

          {currentMode === 'edit' && <div className="w-[1px] h-4 bg-white/10" />}

          {/* Font Selector Dropdown */}
          <div className="relative">
            <button
              onClick={() => setShowFontDropdown(!showFontDropdown)}
              className="px-2 py-1 rounded-lg text-[10px] font-bold flex items-center gap-1 transition-all border border-white/5 bg-black/30 text-zinc-300 hover:text-white cursor-pointer"
            >
              <Type size={10} className="text-cyan-400" />
              <span className="capitalize">{currentFont}</span>
            </button>

            {showFontDropdown && (
              <div className="absolute top-full left-0 mt-1 rounded-lg border shadow-xl p-1 z-50 flex flex-col gap-0.5 min-w-[80px] bg-zinc-950 border-white/10">
                {['sans', 'serif', 'mono'].map((f) => (
                  <button
                    key={f}
                    onClick={() => {
                      onUpdate(node.id, { fontStyle: f as any });
                      setShowFontDropdown(false);
                    }}
                    className={cn(
                      "px-2 py-1 text-left text-[10px] font-bold rounded cursor-pointer flex items-center justify-between",
                      currentFont === f 
                        ? "bg-cyan-500/10 text-cyan-400 font-extrabold"
                        : "text-zinc-400 hover:bg-white/5 hover:text-white"
                    )}
                  >
                    <span className="capitalize">{f}</span>
                    {currentFont === f && <Check size={8} />}
                  </button>
                ))}
              </div>
            )}
          </div>

          <div className="w-[1px] h-4 bg-white/10" />

          {/* Outline TOC toggle */}
          <button
            onClick={() => onUpdate(node.id, { showTOC: !currentShowTOC })}
            className={cn(
              "px-2 py-1 rounded-lg border text-[10px] font-bold flex items-center gap-1 transition-all cursor-pointer",
              currentShowTOC 
                ? "bg-cyan-500/10 text-cyan-400 border-cyan-500/20" 
                : "border-white/5 bg-black/30 text-zinc-400 hover:text-white"
            )}
          >
            <Menu size={10} />
            <span>大纲目录</span>
          </button>
        </>
      )}

      {/* ========================================================
          B. IMAGE NODE TOOLBAR
          ======================================================== */}
      {node.type === 'image-gen' && (
        <>
          <div className="flex items-center gap-1.5 px-1.5">
            <ImageIcon size={11} className="text-cyan-400 shrink-0" />
            <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">图片属性</span>
          </div>

          <div className="w-[1px] h-4 bg-white/10" />

          {/* Aspect Ratio Switcher */}
          <div className="flex items-center gap-1">
            {['1:1', '4:3', '16:9', '9:16'].map((ratio) => (
              <button
                key={ratio}
                onClick={() => {
                  onUpdate(node.id, { ratio });
                  // Adjust height to respect aspect ratio
                  const width = node.width || 260;
                  const numericRatio = ratio === '1:1' ? 1 : ratio === '4:3' ? 4/3 : ratio === '16:9' ? 16/9 : 9/16;
                  const targetHeight = Math.round(width / numericRatio) + 37;
                  onUpdate(node.id, { ratio, height: targetHeight });
                }}
                className={cn(
                  "px-2 py-1 text-[9.5px] font-bold rounded-lg transition-all border cursor-pointer",
                  node.ratio === ratio 
                    ? "bg-cyan-500 text-black border-cyan-400 shadow-sm" 
                    : "border-white/5 bg-black/30 text-zinc-400 hover:text-white"
                )}
              >
                {ratio}
              </button>
            ))}
          </div>

          <div className="w-[1px] h-4 bg-white/10" />

          {/* Regenerate Trigger */}
          {triggerGeneration && (
            <button
              onClick={() => triggerGeneration()}
              disabled={node.status === 'generating'}
              className="px-2.5 py-1 text-[10px] font-bold rounded-lg bg-cyan-500 hover:bg-cyan-400 text-black hover:scale-102 active:scale-98 transition-all flex items-center gap-1 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Sparkles size={10} />
              <span>{node.status === 'completed' ? '重新渲染' : '生成图像'}</span>
            </button>
          )}
        </>
      )}

      {/* ========================================================
          C. VIDEO NODE TOOLBAR
          ======================================================== */}
      {node.type === 'video-gen' && (
        <>
          <div className="flex items-center gap-1.5 px-1.5">
            <VideoIcon size={11} className="text-indigo-400 shrink-0" />
            <span className="text-[10px] font-bold text-zinc-400 uppercase tracking-wider">视频渲染</span>
          </div>

          <div className="w-[1px] h-4 bg-white/10" />

          {/* Aspect Ratio Switcher */}
          <div className="flex items-center gap-1">
            {['1:1', '16:9', '9:16'].map((ratio) => (
              <button
                key={ratio}
                onClick={() => {
                  // Adjust height to respect aspect ratio
                  const width = node.width || 260;
                  const numericRatio = ratio === '1:1' ? 1 : ratio === '16:9' ? 16/9 : 9/16;
                  const targetHeight = Math.round(width / numericRatio) + 37;
                  onUpdate(node.id, { ratio, height: targetHeight });
                }}
                className={cn(
                  "px-2 py-1 text-[9.5px] font-bold rounded-lg transition-all border cursor-pointer",
                  node.ratio === ratio 
                    ? "bg-indigo-500 text-white border-indigo-400 shadow-sm" 
                    : "border-white/5 bg-black/30 text-zinc-400 hover:text-white"
                )}
              >
                {ratio}
              </button>
            ))}
          </div>

          <div className="w-[1px] h-4 bg-white/10" />

          {/* Duration Selector */}
          <div className="flex items-center gap-1 bg-black/30 border border-white/5 rounded-lg px-1.5 py-0.5">
            <span className="text-[9px] text-zinc-500 font-bold mr-1">时长</span>
            {['5s', '10s'].map((d) => {
              const numVal = parseInt(d);
              const isActive = node.duration === numVal;
              return (
                <button
                  key={d}
                  onClick={() => onUpdate(node.id, { duration: numVal })}
                  className={cn(
                    "px-1.5 py-0.5 text-[9px] font-bold rounded cursor-pointer",
                    isActive ? "bg-indigo-500/20 text-indigo-400 font-extrabold" : "text-zinc-500 hover:text-zinc-300"
                  )}
                >
                  {d}
                </button>
              );
            })}
          </div>

          <div className="w-[1px] h-4 bg-white/10" />

          {/* Regenerate Video Trigger */}
          {triggerGeneration && (
            <button
              onClick={() => triggerGeneration()}
              disabled={node.status === 'generating'}
              className="px-2.5 py-1 text-[10px] font-bold rounded-lg bg-indigo-500 hover:bg-indigo-400 text-white hover:scale-102 active:scale-98 transition-all flex items-center gap-1 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Play size={10} fill="currentColor" />
              <span>{node.status === 'completed' ? '重新渲染' : '镜头渲染'}</span>
            </button>
          )}
        </>
      )}

      {/* ========================================================
          D. STICKY NOTE TOOLBAR
          ======================================================== */}
      {node.type === 'sticky' && (
        <>
          <div className="flex items-center gap-1 px-1.5">
            {['yellow', 'pink', 'cyan', 'emerald', 'orange', 'purple'].map((col) => (
              <button
                key={col}
                onClick={() => onUpdate(node.id, { color: col })}
                className={cn(
                  "w-4 h-4 rounded-full border border-black/10 transition-transform cursor-pointer hover:scale-110 relative flex items-center justify-center",
                  col === 'yellow' ? "bg-yellow-300" :
                  col === 'pink' ? "bg-pink-300" :
                  col === 'cyan' ? "bg-cyan-300" :
                  col === 'emerald' ? "bg-emerald-300" :
                  col === 'orange' ? "bg-orange-300" : "bg-purple-300",
                  node.color === col ? "ring-2 ring-black/40 scale-105" : ""
                )}
                title={`设为${col === 'yellow' ? '黄色' : col === 'pink' ? '粉色' : col === 'cyan' ? '蓝色' : col === 'emerald' ? '绿色' : col === 'orange' ? '橙色' : '紫色'}`}
              >
                {node.color === col && <Check size={8} className="text-black/70 stroke-[3]" />}
              </button>
            ))}
          </div>
        </>
      )}

      {/* ========================================================
          GLOBAL OPERATIONS (DELETE)
          ======================================================== */}
      <div className={cn("w-[1px] h-4", node.type === 'sticky' ? "bg-zinc-200" : "bg-white/10")} />

      <button
        onClick={() => onDelete(node.id)}
        className={cn(
          "p-1.5 rounded-lg transition-colors cursor-pointer group/trash flex items-center justify-center",
          node.type === 'sticky' 
            ? "hover:bg-rose-50 text-zinc-500 hover:text-rose-600" 
            : "hover:bg-rose-500/15 text-zinc-400 hover:text-rose-400 border border-transparent hover:border-rose-500/20"
        )}
        title="删除节点"
      >
        <Trash2 size={11} />
      </button>
    </div>
  );
};
