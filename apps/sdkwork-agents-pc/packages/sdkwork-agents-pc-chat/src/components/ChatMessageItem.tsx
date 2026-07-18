import React from 'react';
import { ChatMessage } from '@sdkwork/agents-pc-chat';
import { cn } from '@sdkwork/agents-pc-commons';
import { UserMessageItem } from './UserMessageItem';
import { BotMessageItem } from './BotMessageItem';
import { ImagePreviewList } from './ImagePreviewList';

interface ChatMessageItemProps {
  message: ChatMessage;
  copiedId: string | null;
  feedback: Record<string, 'up' | 'down'>;
  handleCopy: (text: string, id: string) => void;
  handleFeedback: (id: string, type: 'up' | 'down') => void;
  onOpenArtifact: (lang: string, code: string, mode?: 'preview' | 'code') => void;
}

export const ChatMessageItem: React.FC<ChatMessageItemProps> = ({
  message,
  copiedId,
  feedback,
  handleCopy,
  handleFeedback,
  onOpenArtifact
}) => {
  return (
    <div
      className={cn(
        "flex w-full mb-6",
        message.role === 'user' ? "justify-end" : "justify-start"
      )}
    >
      <div className={cn(
        "flex flex-col min-w-0 w-full",
        message.role === 'user' ? "items-end" : "items-start"
      )}>
        <ImagePreviewList images={message.images || []} isUser={message.role === 'user'} />
        
        {message.role === 'user' ? (
          <UserMessageItem 
            message={message} 
            copiedId={copiedId} 
            handleCopy={handleCopy} 
          />
        ) : (
          <BotMessageItem 
            message={message} 
            copiedId={copiedId} 
            feedback={feedback}
            handleCopy={handleCopy}
            handleFeedback={handleFeedback}
            onOpenArtifact={onOpenArtifact}
          />
        )}
      </div>
    </div>
  );
};
