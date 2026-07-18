import React, { useState, useEffect } from 'react';
import { PPTHeader } from './components/PPTHeader';
import { PPTEditor } from './components/PPTEditor';
import { PPTPreview } from './components/PPTPreview';
import { getPresentationHtml } from './utils/pptUtils';
import { PPTService } from '@/packages/sdkwork-chatbox-pc-core/src/services/PPTService';

export const PPTView = () => {
  const [content, setContent] = useState('');
  const [layout, setLayout] = useState<'split' | 'edit' | 'preview'>('split');
  const [theme, setTheme] = useState('black');

  const themes = ['black', 'white', 'league', 'beige', 'sky', 'night', 'serif', 'simple', 'solarized', 'blood', 'moon'];

  const [srcDoc, setSrcDoc] = useState('');

  useEffect(() => {
    PPTService.getDefaultMarkdown().then(md => {
      setContent(md);
      setSrcDoc(getPresentationHtml(md, theme));
    });
  }, []);

  // Debounce the iframe update
  useEffect(() => {
    if (!content) return;
    const handler = setTimeout(() => {
      setSrcDoc(getPresentationHtml(content, theme));
    }, 500);
    return () => clearTimeout(handler);
  }, [content, theme]);

  const handleExport = () => {
    const blob = new Blob([srcDoc], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const opened = window.open(url, '_blank');
    if (!opened) {
      URL.revokeObjectURL(url);
      return;
    }
    window.setTimeout(() => URL.revokeObjectURL(url), 60_000);
  };

  return (
    <div className="flex flex-col h-full bg-white dark:bg-[#0d0d0d] border-l border-gray-200 dark:border-gray-800">
      <PPTHeader 
        layout={layout}
        setLayout={setLayout}
        theme={theme}
        setTheme={setTheme}
        themes={themes}
        onExport={handleExport}
      />
      
      {/* Main split area */}
      <div className="flex-1 flex min-h-0 overflow-hidden">
        <PPTEditor content={content} setContent={setContent} layout={layout} />
        <PPTPreview srcDoc={srcDoc} layout={layout} />
      </div>
    </div>
  );
};
