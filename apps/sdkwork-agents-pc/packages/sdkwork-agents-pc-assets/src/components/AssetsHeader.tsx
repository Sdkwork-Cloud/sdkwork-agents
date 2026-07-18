import React from 'react';
import { Search, RefreshCw } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

interface AssetsHeaderProps {
  activeTab: 'history' | 'subject' | 'canvas';
  setActiveTab: (tab: 'history' | 'subject' | 'canvas') => void;
}

export const AssetsHeader: React.FC<AssetsHeaderProps> = ({ activeTab, setActiveTab }) => {
  return (
    <div className="flex items-center justify-between px-6 py-4">
      {/* Left Tabs */}
      <div className="flex items-center space-x-6">
        <button 
          onClick={() => setActiveTab('history')}
          className={cn("text-sm font-medium transition-colors relative pb-1", activeTab === 'history' ? "text-white" : "text-zinc-500 hover:text-zinc-300")}
        >
          生成历史
          {activeTab === 'history' && <div className="absolute left-1/2 -translate-x-1/2 -bottom-1 w-4 h-[2px] bg-white rounded-full"></div>}
        </button>
        <button 
          onClick={() => setActiveTab('subject')}
          className={cn("text-sm font-medium transition-colors relative pb-1", activeTab === 'subject' ? "text-white" : "text-zinc-500 hover:text-zinc-300")}
        >
          主体
          {activeTab === 'subject' && <div className="absolute left-1/2 -translate-x-1/2 -bottom-1 w-4 h-[2px] bg-white rounded-full"></div>}
        </button>
        <button 
          onClick={() => setActiveTab('canvas')}
          className={cn("text-sm font-medium transition-colors relative pb-1", activeTab === 'canvas' ? "text-white" : "text-zinc-500 hover:text-zinc-300")}
        >
          画布
          {activeTab === 'canvas' && <div className="absolute left-1/2 -translate-x-1/2 -bottom-1 w-4 h-[2px] bg-white rounded-full"></div>}
        </button>
      </div>

      {/* Right Actions */}
      <div className="flex items-center space-x-3">
        <div className="w-8 h-8 flex items-center justify-center rounded-lg bg-white/5 hover:bg-white/10 transition-colors cursor-pointer">
          <Search size={16} className="text-zinc-400" />
        </div>
        <button className="h-8 px-3 rounded-lg bg-white/5 hover:bg-white/10 transition-colors text-xs font-medium flex items-center">
          批量操作
        </button>
        <button className="h-8 px-3 rounded-lg bg-white/5 hover:bg-white/10 transition-colors text-xs font-medium flex items-center space-x-1.5">
          <span>同步到剪映</span>
          <RefreshCw size={12} className="text-zinc-400" />
        </button>
      </div>
    </div>
  );
};
