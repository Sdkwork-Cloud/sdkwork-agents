import React, { useState, useEffect } from 'react';
import { Folder, Share, MoreHorizontal, Mic, SlidersHorizontal, Plus, ChevronDown } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { ProjectService, ProjectDetails } from '@sdkwork/agents-pc-chat';

interface ProjectHomeViewProps {
  projectName: string;
}

export const ProjectHomeView: React.FC<ProjectHomeViewProps> = ({ projectName }) => {
  const [projectDetails, setProjectDetails] = useState<ProjectDetails | null>(null);

  useEffect(() => {
    ProjectService.getProjectDetails(projectName).then(setProjectDetails);
  }, [projectName]);

  const formatDate = (ts: number) => {
    const d = new Date(ts);
    return `${d.getMonth() + 1}月${d.getDate()}日`;
  };

  return (
    <div className="flex-1 flex justify-center w-full bg-[#f5f5f5] dark:bg-[#0d0d0d] overflow-y-auto pt-16">
      <div className="w-full max-w-3xl px-6 flex flex-col items-center">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <div className="flex items-center gap-3">
            <Folder size={24} className="text-zinc-200" />
            <h1 className="text-2xl font-semibold text-white">{projectName}</h1>
          </div>
          
          <div className="flex items-center gap-2">
            <button className="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-white/5 hover:bg-white/10 text-zinc-300 text-[13px] transition-colors border border-white/5">
              <Share size={14} />
              分享
            </button>
            <button className="p-1.5 rounded-full bg-white/5 hover:bg-white/10 text-zinc-300 transition-colors border border-white/5">
              <MoreHorizontal size={16} />
            </button>
          </div>
        </div>
        
        {/* Search / Input Box */}
        <div className="w-full max-w-2xl bg-[#1C1C1E] border border-white/10 rounded-2xl flex items-center px-4 py-3 shadow-sm mb-6">
          <Plus size={20} className="text-zinc-400 mr-3" />
          <input 
            type="text" 
            placeholder={`${projectName}中的新聊天`}
            className="flex-1 bg-transparent border-none outline-none text-white text-[15px] placeholder:text-zinc-500"
          />
          <div className="flex items-center gap-3 ml-2">
            <button className="flex items-center gap-1 text-[13px] text-zinc-400 hover:text-zinc-200 transition-colors">
              Instant
              <ChevronDown size={14} />
            </button>
            <button className="text-zinc-400 hover:text-zinc-200 transition-colors">
              <Mic size={18} />
            </button>
            <div className="w-8 h-8 rounded-full bg-white flex items-center justify-center cursor-pointer">
               <SlidersHorizontal size={16} className="text-black" />
            </div>
          </div>
        </div>
        
        {/* Tabs */}
        <div className="w-full max-w-2xl flex items-center gap-6 border-b border-white/5 pb-3 mb-4">
          <button className="text-[14px] font-medium text-white px-3 py-1 bg-white/10 rounded-full">
            聊天
          </button>
          <button className="text-[14px] font-medium text-zinc-500 hover:text-zinc-300 transition-colors">
            来源
          </button>
        </div>
        
        {/* Chat List */}
        <div className="w-full max-w-2xl flex flex-col gap-2">
          {!projectDetails ? (
             <div className="text-center text-zinc-500 py-4">加载中...</div>
          ) : (
            projectDetails.chats.map(chat => (
              <button key={chat.id} className="w-full text-left flex justify-between items-start py-4 group hover:bg-white/5 rounded-xl px-2 -mx-2 transition-colors">
                <div>
                  <div className="text-[15px] font-medium text-white mb-1">{chat.title}</div>
                  <div className="text-[13px] text-zinc-500 line-clamp-1">{chat.messages[0]?.text || '无内容'}</div>
                </div>
                <span className="text-[13px] text-zinc-500 pt-0.5 group-hover:text-zinc-400 transition-colors">{formatDate(chat.updatedAt)}</span>
              </button>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
