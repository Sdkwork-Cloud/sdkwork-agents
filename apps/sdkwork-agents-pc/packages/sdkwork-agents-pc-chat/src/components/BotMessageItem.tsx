import React from 'react';
import { Copy, Check, ThumbsUp, ThumbsDown } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
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
  const showTypingIndicator = isStreaming && !message.text;

  return (
    <div className="chat-markdown-assistant group relative flex w-full min-w-0 items-start">
      <div className="relative flex w-full min-w-0 flex-col gap-1.5 text-[15px] leading-[1.75]">
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
