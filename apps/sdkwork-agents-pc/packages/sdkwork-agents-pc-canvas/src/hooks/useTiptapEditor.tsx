import React, { useState, useRef, useEffect } from 'react';
import { parseMarkdownToBlocks } from '../utils/markdownParser';
import { 
  Heading1, 
  Heading2, 
  Heading3, 
  List, 
  ListOrdered, 
  CheckSquare, 
  Quote, 
  Code, 
  Info, 
  Plus 
} from 'lucide-react';

interface Command {
  id: string;
  label: string;
  desc: string;
  icon: React.ReactNode;
  template: string;
}

export function useTiptapEditor(
  content: string, 
  onChange: (value: string) => void, 
  nodeId: string,
  externalMode?: 'edit' | 'preview',
  externalFontStyle?: 'sans' | 'serif' | 'mono',
  externalShowTOC?: boolean
) {
  const [localMode, setLocalMode] = useState<'edit' | 'preview'>('preview');
  const mode = externalMode !== undefined ? externalMode : localMode;
  const setMode = setLocalMode;

  const [localFontStyle, setLocalFontStyle] = useState<'sans' | 'serif' | 'mono'>(() => {
    return (localStorage.getItem(`font_${nodeId}`) as any) || 'sans';
  });
  const fontStyle = externalFontStyle !== undefined ? externalFontStyle : localFontStyle;
  const setFontStyle = (val: 'sans' | 'serif' | 'mono') => {
    setLocalFontStyle(val);
    localStorage.setItem(`font_${nodeId}`, val);
  };

  const [localShowTOC, setLocalShowTOC] = useState(false);
  const showTOC = externalShowTOC !== undefined ? externalShowTOC : localShowTOC;
  const setShowTOC = setLocalShowTOC;

  const [copiedCodeIdx, setCopiedCodeIdx] = useState<number | null>(null);

  // Slash commands state
  const [slashMenuOpen, setSlashMenuOpen] = useState(false);
  const [slashQuery, setSlashQuery] = useState('');
  const [slashIndex, setSlashIndex] = useState(0);
  const [slashPosition, setSlashPosition] = useState({ top: 0, left: 0 });

  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const slashMenuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    localStorage.setItem(`font_${nodeId}`, fontStyle);
  }, [fontStyle, nodeId]);

  const commands: Command[] = [
    { id: 'h1', label: '一级标题', desc: 'Heading 1', icon: <Heading1 size={14} />, template: '# ' },
    { id: 'h2', label: '二级标题', desc: 'Heading 2', icon: <Heading2 size={14} />, template: '## ' },
    { id: 'h3', label: '三级标题', desc: 'Heading 3', icon: <Heading3 size={14} />, template: '### ' },
    { id: 'todo', label: '任务列表', desc: 'Task / Checklist', icon: <CheckSquare size={14} />, template: '- [ ] ' },
    { id: 'bullet', label: '无序列表', desc: 'Bullet Point', icon: <List size={14} />, template: '- ' },
    { id: 'number', label: '有序列表', desc: 'Numbered List', icon: <ListOrdered size={14} />, template: '1. ' },
    { id: 'quote', label: '引用段落', desc: 'Blockquote', icon: <Quote size={14} />, template: '> ' },
    { id: 'callout', label: '高亮卡片', desc: 'Callout Info Box', icon: <Info size={14} />, template: '> [!info] 提示内容\n> ' },
    { id: 'code', label: '代码块', desc: 'Code Block', icon: <Code size={14} />, template: '```javascript\n\n```' },
    { id: 'divider', label: '分割线', desc: 'Horizontal Rule', icon: <Plus size={14} className="rotate-45" />, template: '\n---\n' }
  ];

  const filteredCommands = commands.filter(cmd => 
    cmd.label.toLowerCase().includes(slashQuery.toLowerCase()) || 
    cmd.desc.toLowerCase().includes(slashQuery.toLowerCase()) ||
    cmd.id.toLowerCase().includes(slashQuery.toLowerCase())
  );

  const insertCommand = (cmd: Command) => {
    if (!textareaRef.current) return;
    const textarea = textareaRef.current;
    const val = textarea.value;
    const cursor = textarea.selectionStart;
    const lastSlash = val.lastIndexOf('/', cursor - 1);

    if (lastSlash !== -1) {
      const before = val.slice(0, lastSlash);
      const after = val.slice(cursor);
      
      let finalInsert = cmd.template;
      let newCursorPos = lastSlash + finalInsert.length;

      // Adjust cursor position for enclosing elements like code block
      if (cmd.id === 'code') {
        newCursorPos = lastSlash + 13; // Position between triple backticks
      }

      const nextVal = before + finalInsert + after;
      onChange(nextVal);
      setSlashMenuOpen(false);

      // Re-focus and update cursor
      setTimeout(() => {
        textarea.focus();
        textarea.setSelectionRange(newCursorPos, newCursorPos);
      }, 50);
    }
  };

  const handleTextareaKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (slashMenuOpen && filteredCommands.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSlashIndex(prev => (prev + 1) % filteredCommands.length);
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSlashIndex(prev => (prev - 1 + filteredCommands.length) % filteredCommands.length);
      } else if (e.key === 'Enter') {
        e.preventDefault();
        insertCommand(filteredCommands[slashIndex]);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        setSlashMenuOpen(false);
      }
    } else {
      // Smart formatting shortcuts (e.g., Tab to indent, pairs matching)
      if (e.key === 'Tab') {
        e.preventDefault();
        const textarea = e.currentTarget;
        const val = textarea.value;
        const start = textarea.selectionStart;
        const end = textarea.selectionEnd;
        const nextVal = val.substring(0, start) + '  ' + val.substring(end);
        onChange(nextVal);
        setTimeout(() => {
          textarea.setSelectionRange(start + 2, start + 2);
        }, 10);
      }
    }
  };

  const handleTextareaKeyUp = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    const textarea = e.currentTarget;
    const val = textarea.value;
    const cursor = textarea.selectionStart;
    
    // Check for "/" triggers
    const lastSlash = val.lastIndexOf('/', cursor - 1);
    const lastNewline = val.lastIndexOf('\n', cursor - 1);
    
    if (lastSlash !== -1 && lastSlash >= lastNewline) {
      const query = val.slice(lastSlash + 1, cursor);
      if (!query.includes(' ') && query.length < 10) {
        setSlashMenuOpen(true);
        setSlashQuery(query);
        setSlashIndex(0);
        
        // Position dropdown inside container gracefully
        const lineCount = val.slice(0, lastSlash).split('\n').length;
        setSlashPosition({
          top: Math.min(190, 38 + lineCount * 17),
          left: 12
        });
        return;
      }
    }
    setSlashMenuOpen(false);
  };

  // Click outside listener for slash menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (slashMenuRef.current && !slashMenuRef.current.contains(event.target as Node)) {
        setSlashMenuOpen(false);
      }
    };

    if (slashMenuOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [slashMenuOpen]);

  // Insert standard markdown block helper
  const applyToolbarFormat = (prefix: string, suffix: string = '') => {
    if (!textareaRef.current) return;
    const textarea = textareaRef.current;
    const val = textarea.value;
    const selectionStart = textarea.selectionStart;
    const selectionEnd = textarea.selectionEnd;
    const selectedText = val.substring(selectionStart, selectionEnd);
    const replacement = prefix + selectedText + suffix;
    
    onChange(val.slice(0, selectionStart) + replacement + val.slice(selectionEnd));
    
    setTimeout(() => {
      textarea.focus();
      const newCursor = selectionStart + prefix.length + selectedText.length;
      textarea.setSelectionRange(newCursor, newCursor);
    }, 50);
  };

  // Toggle todo checklist item in document content
  const toggleChecklistItem = (lineIdx: number) => {
    const lines = content.split('\n');
    const targetLine = lines[lineIdx];
    
    if (targetLine.includes('- [ ]')) {
      lines[lineIdx] = targetLine.replace('- [ ]', '- [x]');
    } else if (targetLine.includes('- [x]')) {
      lines[lineIdx] = targetLine.replace('- [x]', '- [ ]');
    } else if (targetLine.includes('* [ ]')) {
      lines[lineIdx] = targetLine.replace('* [ ]', '* [x]');
    } else if (targetLine.includes('* [x]')) {
      lines[lineIdx] = targetLine.replace('* [x]', '* [ ]');
    }
    
    onChange(lines.join('\n'));
  };

  // Copy code block helper
  const handleCopyCode = (code: string, idx: number) => {
    navigator.clipboard.writeText(code);
    setCopiedCodeIdx(idx);
    setTimeout(() => setCopiedCodeIdx(null), 2000);
  };

  const blocks = parseMarkdownToBlocks(content || '');
  const headingBlocks = blocks.filter(b => b.type === 'h1' || b.type === 'h2' || b.type === 'h3');

  // Stats calculation
  const charCount = (content || '').length;
  const wordCount = (content || '').trim() ? (content || '').trim().split(/\s+/).length : 0;
  const readTime = Math.max(1, Math.ceil(wordCount / 200));

  return {
    mode, setMode,
    fontStyle, setFontStyle,
    showTOC, setShowTOC,
    copiedCodeIdx, setCopiedCodeIdx,
    slashMenuOpen, setSlashMenuOpen,
    slashQuery, setSlashQuery,
    slashIndex, setSlashIndex,
    slashPosition, setSlashPosition,
    textareaRef,
    slashMenuRef,
    commands,
    filteredCommands,
    insertCommand,
    handleTextareaKeyDown,
    handleTextareaKeyUp,
    applyToolbarFormat,
    toggleChecklistItem,
    handleCopyCode,
    blocks,
    headingBlocks,
    charCount,
    wordCount,
    readTime
  };
}
