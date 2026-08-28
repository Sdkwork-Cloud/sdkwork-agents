export interface MarkdownRendererProps {
  content: string;
  onOpenArtifact?: (language: string, code: string, mode?: 'preview' | 'code') => void;
  /** When true, closes open code fences and shows a streaming cursor. */
  streaming?: boolean;
}
