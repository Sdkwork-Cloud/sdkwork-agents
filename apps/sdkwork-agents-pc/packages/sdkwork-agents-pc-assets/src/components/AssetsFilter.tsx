import React from 'react';
import { ChevronDown } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

interface AssetsFilterProps {
  activeFilter: 'image' | 'video' | 'audio' | 'document';
  setActiveFilter: (filter: 'image' | 'video' | 'audio' | 'document') => void;
}

export const AssetsFilter: React.FC<AssetsFilterProps> = ({ activeFilter, setActiveFilter }) => {
  return (
    <div className="flex items-center px-6 pb-4">
      <div className="flex items-center space-x-4 text-xs font-medium text-zinc-500">
        <button 
          onClick={() => setActiveFilter('image')}
          className={cn("transition-colors", activeFilter === 'image' ? "text-white" : "hover:text-zinc-300")}
        >
          图片
        </button>
        <button 
          onClick={() => setActiveFilter('video')}
          className={cn("transition-colors", activeFilter === 'video' ? "text-white" : "hover:text-zinc-300")}
        >
          视频
        </button>
        <button 
          onClick={() => setActiveFilter('audio')}
          className={cn("transition-colors", activeFilter === 'audio' ? "text-white" : "hover:text-zinc-300")}
        >
          音频
        </button>
        <button 
          onClick={() => setActiveFilter('document')}
          className={cn("transition-colors", activeFilter === 'document' ? "text-white" : "hover:text-zinc-300")}
        >
          文档
        </button>
        <div className="w-[1px] h-3 bg-zinc-700 mx-1"></div>
        <button className="flex items-center space-x-1 hover:text-zinc-300 transition-colors">
          <span>筛选</span>
          <ChevronDown size={14} />
        </button>
      </div>
    </div>
  );
};
