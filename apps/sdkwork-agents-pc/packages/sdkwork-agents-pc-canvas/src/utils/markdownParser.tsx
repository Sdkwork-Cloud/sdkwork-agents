import React from 'react';

export interface Block {
  type: 'h1' | 'h2' | 'h3' | 'paragraph' | 'bullet' | 'number' | 'checklist' | 'quote' | 'code-block' | 'callout' | 'divider' | 'empty';
  content: string;
  lineIndex: number;
  isChecked?: boolean;
  number?: string;
  language?: string;
  calloutType?: string;
}

export function parseMarkdownToBlocks(text: string): Block[] {
  const lines = text.split('\n');
  const blocks: Block[] = [];
  let inCodeBlock = false;
  let codeBlockLines: string[] = [];
  let codeBlockStartIndex = -1;
  let codeLanguage = '';

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    if (trimmed.startsWith('```')) {
      if (inCodeBlock) {
        blocks.push({
          type: 'code-block',
          content: codeBlockLines.join('\n'),
          language: codeLanguage,
          lineIndex: codeBlockStartIndex
        });
        inCodeBlock = false;
        codeBlockLines = [];
        codeBlockStartIndex = -1;
      } else {
        inCodeBlock = true;
        codeBlockStartIndex = i;
        codeLanguage = trimmed.slice(3).trim() || 'javascript';
      }
      continue;
    }

    if (inCodeBlock) {
      codeBlockLines.push(line);
      continue;
    }

    if (trimmed.startsWith('>') && (trimmed.includes('[!info]') || trimmed.includes('[!tip]') || trimmed.includes('[!warning]') || trimmed.includes('[!danger]'))) {
      const match = trimmed.match(/\[!(info|tip|warning|danger)\]/i);
      const calloutType = match ? match[1].toLowerCase() : 'info';
      let calloutContent = trimmed.replace(/>\s*\[!(info|tip|warning|danger)\]/i, '').trim();
      let lastCalloutIdx = i;
      while (i + 1 < lines.length && lines[i + 1].trim().startsWith('>')) {
        i++;
        const nextLine = lines[i].trim();
        calloutContent += '\n' + nextLine.slice(1).trim();
      }
      blocks.push({
        type: 'callout',
        calloutType,
        content: calloutContent,
        lineIndex: lastCalloutIdx
      });
      continue;
    }

    if (trimmed === '---' || trimmed === '***' || trimmed === '___') {
      blocks.push({
        type: 'divider',
        content: '',
        lineIndex: i
      });
      continue;
    }

    if (trimmed.startsWith('# ')) {
      blocks.push({ type: 'h1', content: trimmed.slice(2), lineIndex: i });
    } else if (trimmed.startsWith('## ')) {
      blocks.push({ type: 'h2', content: trimmed.slice(3), lineIndex: i });
    } else if (trimmed.startsWith('### ')) {
      blocks.push({ type: 'h3', content: trimmed.slice(4), lineIndex: i });
    } else if (trimmed.startsWith('- [ ] ') || trimmed.startsWith('* [ ] ')) {
      blocks.push({ type: 'checklist', content: trimmed.slice(6), isChecked: false, lineIndex: i });
    } else if (trimmed.startsWith('- [x] ') || trimmed.startsWith('* [x] ') || trimmed.startsWith('- [X] ') || trimmed.startsWith('* [X] ')) {
      blocks.push({ type: 'checklist', content: trimmed.slice(6), isChecked: true, lineIndex: i });
    } else if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
      blocks.push({ type: 'bullet', content: trimmed.slice(2), lineIndex: i });
    } else if (/^\d+\.\s/.test(trimmed)) {
      const match = trimmed.match(/^(\d+)\.\s(.*)/);
      const num = match ? match[1] : '1';
      const text = match ? match[2] : trimmed;
      blocks.push({ type: 'number', content: text, number: num, lineIndex: i });
    } else if (trimmed.startsWith('> ')) {
      blocks.push({ type: 'quote', content: trimmed.slice(2), lineIndex: i });
    } else {
      blocks.push({
        type: trimmed === '' ? 'empty' : 'paragraph',
        content: line,
        lineIndex: i
      });
    }
  }

  if (inCodeBlock) {
    blocks.push({
      type: 'code-block',
      content: codeBlockLines.join('\n'),
      language: codeLanguage,
      lineIndex: codeBlockStartIndex
    });
  }

  return blocks;
}

export function renderInlineStyles(text: string): React.ReactNode {
  if (!text) return '';
  
  const parts: React.ReactNode[] = [];
  let currentIndex = 0;
  const regex = /(`[^`]+`|\*\*[^*]+\*\*|__[^*]+__|_[^_]+_|==[^=]+==|~~[^~]+~~)/g;
  let match;
  
  while ((match = regex.exec(text)) !== null) {
    const matchStr = match[0];
    const matchIndex = match.index;
    
    if (matchIndex > currentIndex) {
      parts.push(text.slice(currentIndex, matchIndex));
    }
    
    if (matchStr.startsWith('`') && matchStr.endsWith('`')) {
      parts.push(
        <code key={matchIndex} className="px-1.5 py-0.5 bg-white/10 text-cyan-300 rounded font-mono text-[11px] border border-white/5 mx-0.5">
          {matchStr.slice(1, -1)}
        </code>
      );
    } else if ((matchStr.startsWith('**') && matchStr.endsWith('**')) || (matchStr.startsWith('__') && matchStr.endsWith('__'))) {
      parts.push(
        <strong key={matchIndex} className="font-extrabold text-white">
          {matchStr.slice(2, -2)}
        </strong>
      );
    } else if (matchStr.startsWith('*') && matchStr.endsWith('*')) {
      parts.push(
        <em key={matchIndex} className="italic text-zinc-300">
          {matchStr.slice(1, -1)}
        </em>
      );
    } else if (matchStr.startsWith('_') && matchStr.endsWith('_')) {
      parts.push(
        <em key={matchIndex} className="italic text-zinc-300">
          {matchStr.slice(1, -1)}
        </em>
      );
    } else if (matchStr.startsWith('==') && matchStr.endsWith('==')) {
      parts.push(
        <mark key={matchIndex} className="bg-yellow-500/20 text-yellow-200 px-1 py-0.5 rounded border border-yellow-500/10 mx-0.5 font-medium">
          {matchStr.slice(2, -2)}
        </mark>
      );
    } else if (matchStr.startsWith('~~') && matchStr.endsWith('~~')) {
      parts.push(
        <span key={matchIndex} className="line-through text-zinc-500">
          {matchStr.slice(2, -2)}
        </span>
      );
    }
    
    currentIndex = regex.lastIndex;
  }
  
  if (currentIndex < text.length) {
    parts.push(text.slice(currentIndex));
  }
  
  return parts.length > 0 ? parts : text;
}
