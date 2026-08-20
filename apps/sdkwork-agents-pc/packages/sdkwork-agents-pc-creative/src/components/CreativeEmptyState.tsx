import React from 'react';
import { Sparkles } from 'lucide-react';
import { CreativeInputBox } from '@sdkwork/agents-pc-commons';

interface CreativeEmptyStateProps {
  activeSessionId: string;
  handleSend: (val: string, mode: string) => void;
  initialMode?: string;
}

export const CreativeEmptyState: React.FC<CreativeEmptyStateProps> = ({
  activeSessionId,
  handleSend,
  initialMode = 'agent',
}) => {
  return (
    <div className="flex-1 flex flex-col items-center justify-center relative px-6 max-w-[1056px] mx-auto w-full">
      <div className="text-2xl font-medium mb-10 text-zinc-800 flex items-center gap-2 dark:text-zinc-100">
        <Sparkles className="text-cyan-400 fill-cyan-400/20" size={24} />
        你好，想创作什么？
      </div>
      <CreativeInputBox 
        key={activeSessionId} 
        initialMode={initialMode} 
        onSubmit={(val, mode) => handleSend(val, mode)} 
        className="w-full" 
      />
    </div>
  );
};
