import React from 'react';
import { ChevronDown, SquarePen } from 'lucide-react';

interface CreativeToolbarProps {
  title: string;
}

export const CreativeToolbar: React.FC<CreativeToolbarProps> = ({ title }) => {
  return (
    <div className="h-14 border-b border-white/5 flex items-center px-6 shrink-0 justify-between">
      <div className="text-[14px] font-medium text-zinc-300 truncate max-w-[300px]">
        {title}
      </div>
      
      {/* Filter controls matching screenshot */}
      <div className="flex items-center gap-4 text-[12px] text-zinc-400 font-medium select-none">
        <div className="flex items-center gap-1 hover:text-zinc-200 cursor-pointer py-1 px-2 rounded hover:bg-white/5 transition-colors">
          <span>时间</span>
          <ChevronDown size={13} className="text-zinc-500" />
        </div>
        <div className="flex items-center gap-1 hover:text-zinc-200 cursor-pointer py-1 px-2 rounded hover:bg-white/5 transition-colors">
          <span>生成模式</span>
          <ChevronDown size={13} className="text-zinc-500" />
        </div>
        <div className="flex items-center gap-1 hover:text-zinc-200 cursor-pointer py-1 px-2 rounded hover:bg-white/5 transition-colors">
          <span>操作类型</span>
          <ChevronDown size={13} className="text-zinc-500" />
        </div>
        <div className="w-[1px] h-3.5 bg-white/10" />
        <div className="flex items-center gap-1.5 hover:text-cyan-300 text-cyan-400 cursor-pointer py-1 px-2 rounded hover:bg-white/5 transition-colors">
          <SquarePen size={12} />
          <span>自资库</span>
        </div>
      </div>
    </div>
  );
};
