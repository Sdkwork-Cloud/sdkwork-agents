import React, { useState, useEffect, useRef } from 'react';
import { X, MessageCircle } from 'lucide-react';
import { ChatSession } from '@sdkwork/agents-pc-chat';
import { cn } from '@sdkwork/agents-pc-commons';

interface SearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  sessions: ChatSession[];
  onSelectSession: (id: string) => void;
}

export const SearchModal: React.FC<SearchModalProps> = ({ isOpen, onClose, sessions, onSelectSession }) => {
  const [query, setQuery] = useState('');
  const [activeTab, setActiveTab] = useState('全部');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp);
    const now = new Date();
    if (date.toDateString() === now.toDateString()) {
      return 'Today';
    }
    return `${date.getFullYear()}年${date.getMonth() + 1}月${date.getDate()}日`;
  };

  const filteredSessions = query.trim() === '' 
    ? sessions 
    : sessions.filter(s => {
        const titleMatch = s.title.toLowerCase().includes(query.toLowerCase());
        const contentMatch = s.messages.some(m => m.text.toLowerCase().includes(query.toLowerCase()));
        return titleMatch || contentMatch;
      });

  const TABS = ['全部', '聊天', '图片', '文档', '项目'];

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/60 backdrop-blur-sm animate-in fade-in duration-200" onClick={onClose}>
      <div 
        className="w-[680px] max-h-[75vh] bg-[#232323] rounded-2xl overflow-hidden shadow-2xl relative animate-in zoom-in-95 duration-200 flex flex-col border border-[#333]"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header / Input */}
        <div className="flex items-center px-5 py-4 border-b border-transparent shrink-0">
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索..."
            className="flex-1 bg-transparent text-white text-[17px] font-medium outline-none placeholder:text-zinc-400"
          />
          <div className="flex items-center ml-2 shrink-0 h-6">
            {query && (
              <>
                <button 
                  onClick={() => setQuery('')} 
                  className="text-[14px] text-zinc-400 hover:text-white transition-colors"
                >
                  清除
                </button>
                <div className="w-[1px] h-4 bg-zinc-700 mx-3" />
              </>
            )}
            <button onClick={onClose} className="text-zinc-400 hover:text-white transition-colors">
              <X size={20} />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-3 pb-3 custom-scrollbar flex flex-col">
          {query.trim() === '' ? (
            <div className="space-y-1">
              <div className="text-[13px] text-zinc-400 px-3 py-3 font-medium">最近聊天</div>
              {sessions.slice(0, 10).map(session => (
                <button
                  key={session.id}
                  className="w-full flex items-center gap-4 px-3 py-3 text-white hover:bg-white/5 rounded-xl transition-colors text-left group"
                  onClick={() => {
                    onSelectSession(session.id);
                    onClose();
                  }}
                >
                  <MessageCircle size={18} className="text-zinc-400 shrink-0" />
                  <span className="text-[15px] truncate flex-1">{session.title}</span>
                </button>
              ))}
            </div>
          ) : (
            <div className="space-y-3">
              {/* Tabs */}
              <div className="flex items-center gap-1 px-1 mb-2">
                {TABS.map(tab => (
                  <button
                    key={tab}
                    onClick={() => setActiveTab(tab)}
                    className={cn(
                      "px-4 py-1.5 rounded-full text-[14px] font-medium transition-colors",
                      activeTab === tab 
                        ? "bg-white/10 text-white" 
                        : "text-zinc-400 hover:text-zinc-300 hover:bg-white/5"
                    )}
                  >
                    {tab}
                  </button>
                ))}
              </div>

              <div className="space-y-1">
                {filteredSessions.length > 0 ? (
                  filteredSessions.map((session, index) => {
                    const matchingMessage = session.messages.find(m => m.text.toLowerCase().includes(query.toLowerCase()));
                    let snippet = matchingMessage ? matchingMessage.text : session.messages.find(m => m.role === 'user')?.text || '...';
                    
                    if (snippet.length > 80) {
                      snippet = snippet.substring(0, 80) + '...';
                    }

                    return (
                      <button
                        key={session.id}
                        className={cn(
                          "w-full flex items-start gap-4 p-4 text-white rounded-xl transition-colors text-left group",
                          index === 0 ? "bg-[#2d2d2d]" : "hover:bg-[#2a2a2a]"
                        )}
                        onClick={() => {
                          onSelectSession(session.id);
                          onClose();
                        }}
                      >
                        <MessageCircle size={20} className="text-zinc-400 shrink-0 mt-0.5" />
                        <div className="flex-1 min-w-0 pr-4">
                          <div className="text-[16px] font-medium truncate mb-1">
                            {session.title}
                          </div>
                          <div className="text-[14px] text-zinc-400 truncate">
                            {snippet}
                          </div>
                        </div>
                        <div className="text-[13px] text-zinc-400 shrink-0 mt-1 whitespace-nowrap">
                          {formatDate(session.updatedAt)}
                        </div>
                      </button>
                    );
                  })
                ) : (
                  <div className="py-12 text-center text-zinc-500 text-[14px]">
                    没有找到匹配的内容
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
