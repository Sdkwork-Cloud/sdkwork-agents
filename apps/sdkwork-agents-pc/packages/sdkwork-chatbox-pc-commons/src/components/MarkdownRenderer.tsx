import { lazy, Suspense, type FC } from 'react';

import { cn } from './classNames';

export { cn };

export interface MarkdownRendererProps {
  content: string;
  onOpenArtifact?: (language: string, code: string, mode?: 'preview' | 'code') => void;
}

const MarkdownRendererImpl = lazy(() =>
  import('./MarkdownRendererImpl').then((module) => ({
    default: module.MarkdownRenderer,
  })),
);

export const MarkdownRenderer: FC<MarkdownRendererProps> = (props) => (
  <Suspense fallback={<div className="whitespace-pre-wrap text-gray-800 dark:text-gray-200">{props.content}</div>}>
    <MarkdownRendererImpl {...props} />
  </Suspense>
);
