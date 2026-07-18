import React, { useState } from 'react';
import { Search, ChevronDown, List as ListIcon, LayoutGrid, FileText, Image as ImageIcon, Presentation, Filter } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

interface FileItem {
  id: string;
  name: string;
  type: 'image' | 'ppt' | 'md' | 'other';
  date: string;
  size: string;
}

const mockFiles: FileItem[] = [
  { id: '1', name: '限时优惠 AI 服务价格表.png', type: 'image', date: '昨天', size: '1.72 MB' },
  { id: '2', name: '猪肚鸡完整教程-图文版.pptx', type: 'ppt', date: '7月11日', size: '36.2 KB' },
  { id: '3', name: 'AI时代, 真正拉开人与人差距的, 不是技术, 而是... .md', type: 'md', date: '7月2日', size: '5.39 KB' },
  { id: '4', name: 'AI正在抹平技术差距, 真正决定未来... -BirdCoder思考.md', type: 'md', date: '7月2日', size: '3.31 KB' },
  { id: '5', name: '微信图片_20260701150814_369_2.jpg', type: 'image', date: '7月2日', size: '561 KB' },
  { id: '6', name: '未来AI企业平台展示图.png', type: 'image', date: '7月1日', size: '1.98 MB' },
  { id: '7', name: '未来科技企业平台展示.png', type: 'image', date: '7月1日', size: '2.00 MB' },
  { id: '8', name: 'product-intro-zh-business.slides(1).md', type: 'md', date: '7月1日', size: '35.5 KB' },
  { id: '9', name: 'SDKWork_商务产品介绍_示例版.pptx', type: 'ppt', date: '7月1日', size: '35.3 KB' },
  { id: '10', name: 'product-intro-zh-business.slides.md', type: 'md', date: '7月1日', size: '35.5 KB' },
  { id: '11', name: 'business-plan-zh.slides.md', type: 'md', date: '7月1日', size: '23.4 KB' },
  { id: '12', name: 'AI时代的职场未来.png', type: 'image', date: '6月30日', size: '1.88 MB' },
];

export const FileLibraryView = () => {
  const [activeTab, setActiveTab] = useState<'all' | 'image' | 'file'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  
  const getFileIcon = (type: string) => {
    switch (type) {
      case 'image':
        return (
          <div className="w-7 h-7 rounded bg-blue-500/20 flex items-center justify-center shrink-0">
            <ImageIcon size={14} className="text-blue-400" />
          </div>
        );
      case 'ppt':
        return (
          <div className="w-7 h-7 rounded bg-red-500/20 flex items-center justify-center shrink-0">
            <Presentation size={14} className="text-red-400" />
          </div>
        );
      case 'md':
        return (
          <div className="w-7 h-7 rounded bg-blue-500/20 flex items-center justify-center shrink-0">
            <FileText size={14} className="text-blue-400" />
          </div>
        );
      default:
        return (
          <div className="w-7 h-7 rounded bg-zinc-500/20 flex items-center justify-center shrink-0">
            <FileText size={14} className="text-zinc-400" />
          </div>
        );
    }
  };

  const filteredFiles = mockFiles.filter(file => {
    const matchesSearch = file.name.toLowerCase().includes(searchQuery.toLowerCase());
    if (!matchesSearch) return false;
    
    if (activeTab === 'image') return file.type === 'image';
    if (activeTab === 'file') return file.type !== 'image';
    return true;
  });

  return (
    <div className="flex flex-col h-full w-full bg-[#000000] text-gray-200 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-10 pt-10 pb-6 shrink-0">
        <h1 className="text-3xl font-bold text-white">资料库</h1>
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <Search size={16} className="text-zinc-400" />
            </div>
            <input
              type="text"
              placeholder="搜索"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="bg-[#1C1C1E] border border-white/5 rounded-full pl-9 pr-4 py-1.5 text-[14px] text-white placeholder:text-zinc-500 focus:outline-none focus:border-white/20 transition-colors w-[240px]"
            />
          </div>
          <button className="flex items-center gap-2 bg-white text-black hover:bg-gray-100 rounded-full px-4 py-1.5 text-[14px] font-medium transition-colors">
            新建
            <ChevronDown size={16} className="text-zinc-500" />
          </button>
        </div>
      </div>

      {/* Tabs & Controls */}
      <div className="flex items-center justify-between px-10 pb-4 shrink-0">
        <div className="flex items-center gap-2">
          {['全部', '图片', '文件'].map((tab, idx) => {
            const tabKey = idx === 0 ? 'all' : idx === 1 ? 'image' : 'file';
            return (
              <button
                key={tabKey}
                onClick={() => setActiveTab(tabKey as any)}
                className={cn(
                  "px-4 py-1.5 rounded-full text-[14px] font-medium transition-colors",
                  activeTab === tabKey 
                    ? "bg-[#2A2A2D] text-white" 
                    : "text-zinc-400 hover:text-white"
                )}
              >
                {tab}
              </button>
            );
          })}
        </div>
        
        <div className="flex items-center gap-4">
          <button className="text-zinc-400 hover:text-white transition-colors">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M3 6h18M6 12h12M10 18h4" />
            </svg>
          </button>
          <div className="w-[1px] h-4 bg-white/10" />
          <button className="text-zinc-400 hover:text-white transition-colors">
            <LayoutGrid size={18} />
          </button>
          <button className="w-8 h-8 rounded-full bg-[#2A2A2D] flex items-center justify-center text-white transition-colors">
            <ListIcon size={18} />
          </button>
        </div>
      </div>

      {/* Table Header */}
      <div className="flex items-center px-10 py-3 border-b border-white/5 text-[13px] text-zinc-400 font-medium shrink-0">
        <div className="flex-1 pr-4">名称</div>
        <div className="w-[180px] shrink-0">修改时间 &darr;</div>
        <div className="w-[120px] shrink-0">大小</div>
      </div>

      {/* File List */}
      <div className="flex-1 overflow-y-auto px-10 py-2 custom-scrollbar">
        <div className="flex flex-col space-y-1 pb-10">
          {filteredFiles.map((file) => (
            <div 
              key={file.id}
              className="flex items-center py-4 border-b border-white/5 hover:bg-white/[0.02] transition-colors cursor-pointer group"
            >
              <div className="flex-1 flex items-center gap-3 min-w-0 pr-4">
                {getFileIcon(file.type)}
                <span className="text-[14px] text-white truncate font-medium group-hover:text-blue-400 transition-colors">
                  {file.name}
                </span>
              </div>
              <div className="w-[180px] shrink-0 text-[14px] text-zinc-400">
                {file.date}
              </div>
              <div className="w-[120px] shrink-0 text-[14px] text-zinc-400">
                {file.size}
              </div>
            </div>
          ))}
          {filteredFiles.length === 0 && (
            <div className="py-20 text-center text-zinc-500 text-[14px]">
              无文件
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
