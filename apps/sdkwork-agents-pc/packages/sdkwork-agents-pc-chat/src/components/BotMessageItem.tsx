import React from 'react';
import {
  Check,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Copy,
  Loader2,
  ThumbsDown,
  ThumbsUp,
  Wrench,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ChatMessage, ChatToolCall } from '@sdkwork/agents-pc-chat';
import { MarkdownRenderer, cn } from '@sdkwork/agents-pc-commons';
import { TypingIndicator } from './TypingIndicator';

interface BotMessageItemProps {
  message: ChatMessage;
  copiedId: string | null;
  feedback: Record<string, 'up' | 'down'>;
  handleCopy: (text: string, id: string) => void;
  handleFeedback: (id: string, type: 'up' | 'down') => void;
  onOpenArtifact: (lang: string, code: string, mode?: 'preview' | 'code') => void;
  isStreaming?: boolean;
}

/** Collapsible "thinking"/"reasoning" block streamed from the runtime. */
const ThinkingBlock: React.FC<{ content: string; streaming: boolean }> = ({
  content,
  streaming,
}) => {
  const { t: tCommon } = useTranslation('common');
  const [open, setOpen] = React.useState(false);
  if (!content) return null;
  return (
    <div className="mb-2 overflow-hidden rounded-lg border border-gray-200/80 bg-gray-50/80 dark:border-gray-700/80 dark:bg-[#232323]">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-center gap-1.5 px-2.5 py-1.5 text-left text-xs font-medium text-gray-500 transition-colors hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
        aria-expanded={open}
      >
        {open ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
        {streaming && <Loader2 size={12} className="animate-spin" />}
        <span>{tCommon('thinking', { defaultValue: '深度思考' })}</span>
      </button>
      {open && (
        <div className="border-t border-gray-200/80 px-2.5 py-2 text-[13px] leading-[1.7] text-gray-600 dark:border-gray-700/80 dark:text-gray-300">
          <MarkdownRenderer content={content} streaming={streaming} />
        </div>
      )}
    </div>
  );
};

/** Collapsible tool/skill/MCP invocation card. */
const ToolCallCard: React.FC<{ tool: ChatToolCall }> = ({ tool }) => {
  const { t: tCommon } = useTranslation('common');
  const [open, setOpen] = React.useState(false);
  const running = tool.status === 'running';
  return (
    <div className="mb-1.5 w-full overflow-hidden rounded-md border border-gray-200/80 bg-gray-50/60 dark:border-gray-700/80 dark:bg-[#1f1f1f]">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex w-full items-center gap-1.5 px-2 py-1.5 text-left text-xs text-gray-600 transition-colors hover:bg-gray-100/70 dark:text-gray-300 dark:hover:bg-[#2a2a2a]"
        aria-expanded={open}
      >
        {running ? (
          <Loader2 size={13} className="animate-spin text-[#1890ff]" />
        ) : tool.status === 'completed' ? (
          <CheckCircle2 size={13} className="text-emerald-500 dark:text-emerald-400" />
        ) : (
          <Wrench size={13} className="text-gray-400" />
        )}
        <span className="truncate font-medium">
          {tool.name || tCommon('toolCall', { defaultValue: '工具调用' })}
        </span>
        {open ? (
          <ChevronDown size={13} className="ml-auto shrink-0" />
        ) : (
          <ChevronRight size={13} className="ml-auto shrink-0" />
        )}
      </button>
      {open && tool.arguments && (
        <pre className="max-h-56 overflow-auto border-t border-gray-200/80 px-2.5 py-2 text-xs whitespace-pre-wrap break-all text-gray-600 dark:border-gray-700/80 dark:text-gray-300">
          {tool.arguments}
        </pre>
      )}
    </div>
  );
};

export const BotMessageItem: React.FC<BotMessageItemProps> = ({
  message,
  copiedId,
  feedback,
  handleCopy,
  handleFeedback,
  onOpenArtifact,
  isStreaming = false,
}) => {
  const { t: tCommon } = useTranslation('common');
  const showTypingIndicator = isStreaming && !message.text && !message.reasoning;

  return (
    <div className="chat-markdown-assistant group relative flex w-full min-w-0 items-start">
      <div className="relative flex w-full min-w-0 flex-col gap-1.5 text-[15px] leading-[1.75]">
        {message.reasoning && <ThinkingBlock content={message.reasoning} streaming={isStreaming} />}

        {message.toolCalls && message.toolCalls.length > 0 && (
          <div className="mb-1 flex max-w-full w-full flex-col gap-0 text-xs">
            {message.toolCalls.map((tool) => (
              <ToolCallCard key={tool.id} tool={tool} />
            ))}
          </div>
        )}

        {showTypingIndicator ? (
          <TypingIndicator />
        ) : (
          <MarkdownRenderer
            content={message.text}
            onOpenArtifact={onOpenArtifact}
            streaming={isStreaming}
          />
        )}

        {!isStreaming && message.text && (
          <div className="mt-0.5 flex items-center gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
            <button
              type="button"
              onClick={() => handleCopy(message.text, message.id)}
              className="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-[#e5e5e5] hover:text-gray-900 dark:hover:bg-[#2f2f2f] dark:hover:text-gray-200"
              title={tCommon('copy')}
            >
              {copiedId === message.id ? (
                <Check size={14} className="text-emerald-500 dark:text-emerald-400" />
              ) : (
                <Copy size={14} />
              )}
            </button>
            <button
              type="button"
              onClick={() => handleFeedback(message.id, 'up')}
              className={cn(
                'rounded-md p-1.5 transition-colors hover:bg-[#e5e5e5] dark:hover:bg-[#2f2f2f]',
                feedback[message.id] === 'up'
                  ? 'text-[#1890ff]'
                  : 'text-gray-400 hover:text-gray-900 dark:hover:text-gray-200',
              )}
              title={tCommon('goodResponse')}
            >
              <ThumbsUp size={14} />
            </button>
            <button
              type="button"
              onClick={() => handleFeedback(message.id, 'down')}
              className={cn(
                'rounded-md p-1.5 transition-colors hover:bg-[#e5e5e5] dark:hover:bg-[#2f2f2f]',
                feedback[message.id] === 'down'
                  ? 'text-red-500 dark:text-red-400'
                  : 'text-gray-400 hover:text-gray-900 dark:hover:text-gray-200',
              )}
              title={tCommon('badResponse')}
            >
              <ThumbsDown size={14} />
            </button>
          </div>
        )}
      </div>
    </div>
  );
};
