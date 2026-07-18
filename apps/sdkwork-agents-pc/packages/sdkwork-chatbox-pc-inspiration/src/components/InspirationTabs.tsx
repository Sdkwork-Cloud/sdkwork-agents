import React from 'react';
import { Search, Plus } from 'lucide-react';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';

interface InspirationTabsProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  searchQuery: string;
  setSearchQuery: (query: string) => void;
}

export const InspirationTabs: React.FC<InspirationTabsProps> = ({
  activeTab,
  setActiveTab,
  searchQuery,
  setSearchQuery
}) => {
  return (
    <div className="flex items-center justify-between mb-8 pb-3 border-b border-white/5">
      <div className="flex items-center gap-6 overflow-x-auto [&::-webkit-scrollbar]:hidden [-ms-overflow-style:none] [scrollbar-width:none]">
        <div className="flex gap-1 bg-[#1e1e1e] rounded-xl p-1">
          {['发现', '技能', '短片', '活动'].map(tab => (
            <button 
              key={tab}
              onClick={() => {
                setActiveTab(tab);
                setSearchQuery('');
              }}
              className={cn(
                "px-5 py-1.5 rounded-lg text-[13px] font-semibold transition-all whitespace-nowrap", 
                activeTab === tab 
                  ? "bg-white/10 text-white shadow" 
                  : "text-zinc-400 hover:text-zinc-200"
              )}
            >
              {tab}
            </button>
          ))}
        </div>
        <div className="relative">
          <Search size={14} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-zinc-500" />
          <input 
            type="text" 
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={activeTab === '技能' ? '搜索实用创意技能' : activeTab === '活动' ? '搜索热门大赛' : '童话世界'}
            className="bg-[#1e1e1e] border border-white/5 rounded-full pl-9 pr-4 py-1.5 text-[13px] text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-white/20 w-[220px] transition-all"
          />
        </div>
      </div>

      {/* "+ 发布短片" Button shown ONLY under Short Video (短片) Tab */}
      {activeTab === '短片' && (
        <button className="flex items-center gap-1.5 bg-[#1e1e1e] hover:bg-[#252525] border border-white/5 rounded-xl px-4 py-2 text-[13px] font-semibold transition-colors shrink-0">
          <Plus size={15} />
          发布短片
        </button>
      )}
    </div>
  );
};
