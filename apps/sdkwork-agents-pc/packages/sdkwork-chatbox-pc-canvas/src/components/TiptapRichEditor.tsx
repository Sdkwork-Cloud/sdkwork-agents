import React, { useState, useRef, useEffect } from 'react';
import { 
  Heading1, 
  Heading2, 
  Heading3, 
  Bold, 
  Italic, 
  List, 
  ListOrdered, 
  CheckSquare, 
  Quote, 
  Code, 
  Sparkles, 
  Plus, 
  CornerDownLeft, 
  ChevronRight, 
  Info, 
  Eye, 
  Edit3, 
  Layers, 
  Type, 
  Copy, 
  Check, 
  BookOpen, 
  BookMarked,
  Sparkle,
  Menu,
  ChevronDown,
  BookOpenCheck
} from 'lucide-react';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';
import { parseMarkdownToBlocks, renderInlineStyles, Block } from '../utils/markdownParser';


import { useTiptapEditor } from '../hooks/useTiptapEditor';

interface TiptapRichEditorProps {
  content: string;
  onChange: (value: string) => void;
  placeholder?: string;
  nodeId: string;
  mode?: 'edit' | 'preview';
  fontStyle?: 'sans' | 'serif' | 'mono';
  showTOC?: boolean;
}

export const TiptapRichEditor: React.FC<TiptapRichEditorProps> = ({
  content,
  onChange,
  placeholder = "在此输入您的创意构想...",
  nodeId,
  mode: externalMode,
  fontStyle: externalFontStyle,
  showTOC: externalShowTOC
}) => {
  const {
    mode,
    fontStyle,
    showTOC,
    copiedCodeIdx,
    slashMenuOpen, setSlashMenuOpen,
    slashQuery,
    slashIndex, setSlashIndex,
    slashPosition,
    textareaRef,
    slashMenuRef,
    filteredCommands,
    insertCommand,
    handleTextareaKeyDown,
    handleTextareaKeyUp,
    toggleChecklistItem,
    handleCopyCode,
    blocks,
    headingBlocks,
  } = useTiptapEditor(content, onChange, nodeId, externalMode, externalFontStyle, externalShowTOC);

  return (
    <div className="flex flex-col flex-1 gap-2 min-h-[140px] overflow-visible relative">
      {/* MAIN CONTENT BODY */}
      <div className="flex-1 flex gap-2 relative min-h-[120px]">
        
        {/* EDITING MODE */}
        <div className={cn("flex-1 flex flex-col relative", mode === 'edit' ? "block" : "hidden")}>
          <textarea
            id={`textarea-${nodeId}`}
            ref={textareaRef}
            value={content}
            onChange={(e) => onChange(e.target.value)}
            onKeyDown={handleTextareaKeyDown}
            onKeyUp={handleTextareaKeyUp}
            placeholder={placeholder + '\n\n💡 提示: 输入 "/" 触发 Tiptap 快捷指令插入排版区块'}
            className={cn(
              "w-full flex-1 min-h-[130px] bg-transparent leading-relaxed text-zinc-200 outline-none border-none resize-none placeholder:text-zinc-600 no-drag text-[12px] p-1.5 focus:ring-0",
              fontStyle === 'sans' && 'font-sans',
              fontStyle === 'serif' && 'font-serif text-[13px] tracking-wide',
              fontStyle === 'mono' && 'font-mono text-[11px]'
            )}
          />

          {/* FLOATING SLASH COMMAND DROPDOWN */}
          {slashMenuOpen && filteredCommands.length > 0 && (
            <div 
              ref={slashMenuRef}
              style={{ top: slashPosition.top, left: slashPosition.left }}
              className="absolute bg-[#121214] border border-white/10 rounded-xl shadow-2xl p-1 z-[150] w-[200px] max-h-[180px] overflow-y-auto no-drag border-cyan-500/20"
            >
              <div className="px-2 py-1 border-b border-white/5 text-[9px] text-zinc-500 font-bold uppercase tracking-wider flex items-center justify-between mb-1">
                <span>插入 Tiptap 排版组件</span>
                <span className="flex items-center gap-0.5"><CornerDownLeft size={8} /> Enter</span>
              </div>
              {filteredCommands.map((cmd, idx) => (
                <button
                  key={cmd.id}
                  onClick={() => insertCommand(cmd)}
                  onMouseEnter={() => setSlashIndex(idx)}
                  className={cn(
                    "w-full px-2 py-1.5 rounded-lg text-left text-[11px] flex items-center gap-2.5 transition-colors cursor-pointer",
                    slashIndex === idx ? "bg-cyan-500 text-black font-semibold" : "text-zinc-300 hover:bg-white/5"
                  )}
                >
                  <span className={cn("p-1 rounded", slashIndex === idx ? "bg-black/10 text-black" : "bg-white/5 text-cyan-400")}>
                    {cmd.icon}
                  </span>
                  <div className="flex-1 min-w-0">
                    <div className="truncate">{cmd.label}</div>
                    <div className={cn("text-[9px] truncate font-normal", slashIndex === idx ? "text-black/70" : "text-zinc-500")}>
                      {cmd.desc}
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* WYSIWYG PREVIEW MODE */}
        <div className={cn(
          "flex-1 overflow-y-auto custom-scrollbar text-[12.5px] leading-relaxed text-zinc-200 p-1.5 min-h-[130px] select-text",
          mode === 'preview' ? "block" : "hidden",
          fontStyle === 'sans' && 'font-sans',
          fontStyle === 'serif' && 'font-serif text-[13.5px] tracking-wide',
          fontStyle === 'mono' && 'font-mono text-[11.5px]'
        )}>
          {blocks.length === 0 || (blocks.length === 1 && blocks[0].type === 'empty') ? (
            <div className="text-zinc-600 italic py-4 text-center text-[11px]">Empty text node...</div>
          ) : (
            <div className="flex flex-col gap-2.5">
              {blocks.map((block, idx) => {
                switch (block.type) {
                  case 'h1':
                    return (
                      <h1 key={idx} className="text-base font-extrabold text-white border-l-2 border-cyan-400 pl-2 mt-2 flex items-center gap-1 tracking-tight">
                        {renderInlineStyles(block.content)}
                      </h1>
                    );
                  case 'h2':
                    return (
                      <h2 key={idx} className="text-sm font-bold text-zinc-100 pl-1 mt-1.5 text-cyan-300/90 tracking-tight">
                        {renderInlineStyles(block.content)}
                      </h2>
                    );
                  case 'h3':
                    return (
                      <h3 key={idx} className="text-[12.5px] font-bold text-zinc-300 mt-1 pl-1">
                        {renderInlineStyles(block.content)}
                      </h3>
                    );
                  case 'checklist':
                    return (
                      <div key={idx} className="flex items-start gap-2.5 py-0.5 group/todo no-drag">
                        <button
                          onClick={() => toggleChecklistItem(block.lineIndex)}
                          className={cn(
                            "w-4 h-4 rounded mt-0.5 flex items-center justify-center border transition-all cursor-pointer shrink-0",
                            block.isChecked 
                              ? "bg-cyan-500/20 border-cyan-400 text-cyan-400 shadow-[0_0_6px_rgba(34,211,238,0.25)]" 
                              : "border-zinc-700 hover:border-cyan-400/50 bg-black/20"
                          )}
                        >
                          {block.isChecked && <Check size={11} strokeWidth={3} />}
                        </button>
                        <span className={cn(
                          "transition-all duration-200",
                          block.isChecked ? "line-through text-zinc-500 italic" : "text-zinc-200"
                        )}>
                          {renderInlineStyles(block.content)}
                        </span>
                      </div>
                    );
                  case 'bullet':
                    return (
                      <div key={idx} className="flex items-start gap-2 pl-2">
                        <span className="text-cyan-400 text-xs mt-0.5 shrink-0">•</span>
                        <span>{renderInlineStyles(block.content)}</span>
                      </div>
                    );
                  case 'number':
                    return (
                      <div key={idx} className="flex items-start gap-2 pl-2">
                        <span className="text-zinc-500 font-mono text-[11px] mt-0.5 shrink-0">{block.number}.</span>
                        <span>{renderInlineStyles(block.content)}</span>
                      </div>
                    );
                  case 'quote':
                    return (
                      <blockquote key={idx} className="border-l-3 border-zinc-600 bg-white/[0.02] py-1.5 px-3 rounded-r-lg italic text-zinc-400 my-1 font-medium">
                        {renderInlineStyles(block.content)}
                      </blockquote>
                    );
                  case 'callout':
                    return (
                      <div 
                        key={idx} 
                        className={cn(
                          "p-3 rounded-xl border flex gap-2.5 my-1.5 text-[11.5px] relative overflow-hidden shadow-inner",
                          block.calloutType === 'tip' && 'bg-emerald-950/20 border-emerald-500/20 text-emerald-300',
                          block.calloutType === 'warning' && 'bg-amber-950/20 border-amber-500/20 text-amber-300',
                          block.calloutType === 'danger' && 'bg-rose-950/20 border-rose-500/20 text-rose-300',
                          (!block.calloutType || block.calloutType === 'info') && 'bg-cyan-950/20 border-cyan-500/20 text-cyan-200'
                        )}
                      >
                        <div className="shrink-0 mt-0.5">
                          {block.calloutType === 'tip' && <Sparkles size={14} className="text-emerald-400" />}
                          {block.calloutType === 'warning' && <Info size={14} className="text-amber-400" />}
                          {block.calloutType === 'danger' && <Info size={14} className="text-rose-400" />}
                          {(!block.calloutType || block.calloutType === 'info') && <Info size={14} className="text-cyan-400" />}
                        </div>
                        <div className="flex-1 whitespace-pre-line leading-relaxed">
                          {renderInlineStyles(block.content)}
                        </div>
                      </div>
                    );
                  case 'code-block':
                    return (
                      <div key={idx} className="rounded-xl border border-white/5 bg-black/40 overflow-hidden font-mono text-[11px] my-2 relative group/code no-drag">
                        <div className="flex items-center justify-between px-3 py-1.5 bg-zinc-900/60 border-b border-white/5 text-[9px] text-zinc-500">
                          <span className="uppercase tracking-wider font-bold">{block.language || 'code'}</span>
                          <button
                            onClick={() => handleCopyCode(block.content, idx)}
                            className="text-zinc-500 hover:text-white transition-colors cursor-pointer flex items-center gap-1 p-0.5"
                          >
                            {copiedCodeIdx === idx ? <Check size={10} className="text-emerald-400" /> : <Copy size={10} />}
                            <span>{copiedCodeIdx === idx ? '已复制' : '复制'}</span>
                          </button>
                        </div>
                        <pre className="p-3 overflow-x-auto text-zinc-300 custom-scrollbar whitespace-pre leading-relaxed">
                          <code>{block.content}</code>
                        </pre>
                      </div>
                    );
                  case 'divider':
                    return <hr key={idx} className="border-white/5 my-2.5" />;
                  case 'empty':
                    return <div key={idx} className="h-1.5" />;
                  default:
                    return <p key={idx}>{renderInlineStyles(block.content)}</p>;
                }
              })}
            </div>
          )}
        </div>

        {/* OUTLINE TABLE OF CONTENTS SIDEBAR PANEL */}
        {showTOC && headingBlocks.length > 0 && (
          <div className="absolute right-0 top-0 bottom-0 w-[110px] bg-[#121214] border-l border-white/5 p-2 flex flex-col gap-1.5 z-40 animate-in slide-in-from-right-2 duration-200 select-none no-drag shadow-2xl rounded-r-xl">
            <div className="text-[9px] text-zinc-500 font-bold uppercase tracking-wider mb-1 flex items-center gap-1">
              <BookOpenCheck size={10} className="text-cyan-400" /> 文档大纲
            </div>
            <div className="flex-1 overflow-y-auto custom-scrollbar flex flex-col gap-1">
              {headingBlocks.map((h, hIdx) => (
                <button
                  key={hIdx}
                  onClick={() => {
                    // Quick edit scroll to line helper if needed
                    if (textareaRef.current && mode === 'edit') {
                      const lines = content.split('\n');
                      let charOffset = 0;
                      for (let i = 0; i < h.lineIndex; i++) {
                        charOffset += lines[i].length + 1;
                      }
                      textareaRef.current.focus();
                      textareaRef.current.setSelectionRange(charOffset, charOffset + lines[h.lineIndex].length);
                    }
                  }}
                  className={cn(
                    "text-left text-[9.5px] truncate hover:text-cyan-400 transition-colors cursor-pointer block py-0.5",
                    h.type === 'h1' && 'pl-0 text-zinc-300 font-bold',
                    h.type === 'h2' && 'pl-2 text-zinc-400 font-medium',
                    h.type === 'h3' && 'pl-3 text-zinc-500'
                  )}
                  title={h.content}
                >
                  {h.content}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>

      </div>
  );
};

