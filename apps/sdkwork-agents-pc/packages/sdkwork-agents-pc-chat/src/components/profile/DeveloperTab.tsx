import React from 'react';
import { Copy } from 'lucide-react';

interface DeveloperTabProps {
  t: (key: string) => string;
  handleCopyCode: () => void;
}

export const DeveloperTab: React.FC<DeveloperTabProps> = ({ t, handleCopyCode }) => {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-gray-900 dark:text-white tracking-tight">{t('integrationTitle')}</h3>
        <p className="text-xs text-gray-500 mt-1">{t('developerSubtitle')}</p>
      </div>

      <div className="space-y-4">
        <div className="flex justify-between items-center">
          <span className="text-[11px] uppercase tracking-widest text-[#1890ff] font-bold font-mono">TypeScript / ES Module</span>
          <button 
            onClick={handleCopyCode}
            className="flex items-center gap-1.5 text-xs text-gray-500 dark:text-zinc-400 hover:text-[#1890ff] font-bold transition-colors cursor-pointer"
          >
            <Copy size={13} />
            {t('copySnippet')}
          </button>
        </div>

        <div className="rounded-2xl border border-gray-200 dark:border-zinc-800 bg-[#282c34] text-gray-200 p-5 font-mono text-[11px] leading-relaxed relative overflow-hidden shadow-inner max-h-[350px] overflow-y-auto">
          <span className="absolute right-3 top-3 text-[9px] uppercase tracking-wider bg-white/10 px-2 py-0.5 rounded text-white/50">TypeScript SDK</span>
          <pre className="whitespace-pre-wrap select-all">
{`import { ChatService } from '@sdkwork/agents-pc-chat';

// 1. Configure standard streams
await ChatService.streamChat({
  model: 'gemini-2.5-flash',
  vendor: 'Google',
  messages: [
    { role: 'user', text: 'Hello, World!' }
  ],
  onMessageUpdate: (text) => {
    console.log("Chunk received:", text);
  },
  onComplete: () => {
    console.log("Stream delivery finished.");
  }
});`}
          </pre>
        </div>

        <div className="p-4 rounded-xl border border-gray-100 dark:border-zinc-800 bg-gray-50/50 dark:bg-zinc-900/20 text-xs font-semibold text-gray-600 dark:text-zinc-300">
          <p className="mb-2 uppercase text-[10px] tracking-wider text-gray-400">{t('quickDevRef')}</p>
          <ul className="list-disc leading-loose pl-4 font-mono text-[11px] text-gray-500">
            <li>ChatService.streamChat(options: StreamOptions)</li>
            <li>ChatMessage: &#123; id: string; role: 'user' | 'model'; text: string; images?: string[] &#125;</li>
            <li>ChatSession: &#123; id: string; title: string; messages: ChatMessage[]; updatedAt: number &#125;</li>
          </ul>
        </div>
      </div>
    </div>
  );
};
