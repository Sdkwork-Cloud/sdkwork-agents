import React, { useEffect, useRef } from 'react';
import { X, Trash2, Info, CheckCircle2, AlertTriangle, AlertCircle, Clock } from 'lucide-react';
import { useAgentState } from '@/src/contexts/AgentStateContext';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';

interface AgentHistoryLogProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AgentHistoryLog: React.FC<AgentHistoryLogProps> = ({ isOpen, onClose }) => {
  const { events, clearEvents } = useAgentState();
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [events, isOpen]);

  if (!isOpen) return null;

  const getEventIcon = (type: string) => {
    switch (type) {
      case 'success': return <CheckCircle2 size={14} className="text-emerald-500" />;
      case 'warning': return <AlertTriangle size={14} className="text-yellow-500" />;
      case 'error': return <AlertCircle size={14} className="text-rose-500" />;
      case 'info':
      default:
        return <Info size={14} className="text-blue-500" />;
    }
  };

  const formatTime = (ts: number) => {
    return new Date(ts).toLocaleTimeString(undefined, { 
      hour12: false, 
      hour: '2-digit', 
      minute: '2-digit',
      second: '2-digit'
    });
  };

  return (
    <div className="fixed top-16 right-4 z-40 w-80 max-h-[400px] flex flex-col bg-white/90 dark:bg-[#1C1C1E]/90 backdrop-blur-md border border-gray-200 dark:border-white/10 rounded-xl shadow-lg overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200">
      <div className="flex justify-between items-center px-4 py-3 border-b border-gray-200 dark:border-white/10 bg-gray-50/50 dark:bg-black/20">
        <div className="flex items-center gap-2">
          <Clock size={16} className="text-gray-500 dark:text-gray-400" />
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Agent History</h3>
        </div>
        <div className="flex items-center gap-1">
          <button 
            onClick={clearEvents}
            className="p-1.5 rounded-md hover:bg-gray-200 dark:hover:bg-white/10 text-gray-500 transition-colors"
            title="Clear Log"
          >
            <Trash2 size={14} />
          </button>
          <button 
            onClick={onClose}
            className="p-1.5 rounded-md hover:bg-gray-200 dark:hover:bg-white/10 text-gray-500 transition-colors"
            title="Close"
          >
            <X size={14} />
          </button>
        </div>
      </div>
      
      <div 
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-2 space-y-1"
      >
        {events.length === 0 ? (
          <div className="py-8 text-center text-xs text-gray-500 dark:text-gray-400">
            No events recorded yet.
          </div>
        ) : (
          events.map((event) => (
            <div 
              key={event.id}
              className="flex items-start gap-2.5 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-white/5 transition-colors group"
            >
              <div className="pt-0.5 shrink-0">
                {getEventIcon(event.type)}
              </div>
              <div className="flex-1 min-w-0">
                <p className={cn(
                  "text-xs leading-relaxed break-words",
                  event.type === 'error' ? "text-rose-600 dark:text-rose-400" :
                  event.type === 'warning' ? "text-yellow-700 dark:text-yellow-400" :
                  "text-gray-700 dark:text-gray-300"
                )}>
                  {event.message}
                </p>
                <div className="text-[10px] text-gray-400 dark:text-gray-500 mt-0.5">
                  {formatTime(event.timestamp)}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
