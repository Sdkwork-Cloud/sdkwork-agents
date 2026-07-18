import React from 'react';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';

interface PPTPreviewProps {
  srcDoc: string;
  layout: 'split' | 'edit' | 'preview';
}

export const PPTPreview: React.FC<PPTPreviewProps> = ({ srcDoc, layout }) => {
  if (layout !== 'split' && layout !== 'preview') return null;

  return (
    <div className={cn("h-full bg-white dark:bg-[#0d0d0d] relative", layout === 'split' ? "w-1/2" : "w-full")}>
      <iframe
        srcDoc={srcDoc}
        className="w-full h-full border-none"
        sandbox="allow-scripts allow-forms allow-same-origin"
        title="PPT Preview"
      />
    </div>
  );
};
