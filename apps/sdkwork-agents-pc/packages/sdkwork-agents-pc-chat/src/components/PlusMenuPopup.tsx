import React from 'react';
import { Paperclip, ImageIcon, Globe, Telescope } from 'lucide-react';

interface PlusMenuPopupProps {
  menuRef: React.RefObject<HTMLDivElement>;
  setShowMenu: (show: boolean) => void;
  setInputMode: (mode: 'image' | 'search' | 'research' | null) => void;
  fileInputRef: React.RefObject<HTMLInputElement>;
}

export const PlusMenuPopup: React.FC<PlusMenuPopupProps> = ({
  menuRef,
  setShowMenu,
  setInputMode,
  fileInputRef
}) => {
  return (
    <div 
      ref={menuRef} 
      className="absolute bottom-full left-0 mb-3 w-[600px] max-w-[calc(100vw-32px)] max-h-[60vh] flex flex-col bg-[#1e1e1e] border border-white/10 rounded-2xl shadow-xl z-50 animate-in fade-in zoom-in-95 duration-200"
    >
      <div className="py-2 flex flex-col overflow-y-auto custom-scrollbar">
        <button 
          onClick={() => {
            setShowMenu(false);
            fileInputRef.current?.click();
          }}
          className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition-colors text-left group"
        >
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-white/5 text-zinc-300 shrink-0">
            <Paperclip size={16} className="text-zinc-300" />
          </div>
          <div className="flex-1">
            <div className="text-zinc-200 text-[14px] font-medium flex items-center gap-2">添加照片和文件 <span className="text-zinc-500 text-[12px] font-normal">从电脑上传</span></div>
          </div>
        </button>
        
        <button 
          onClick={() => { setInputMode('image'); setShowMenu(false); }}
          className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition-colors text-left group">
          <div className="flex items-center justify-center w-8 h-8 shrink-0">
            <ImageIcon size={20} className="text-[#3b82f6]" />
          </div>
          <div className="flex-1">
            <div className="text-zinc-200 text-[14px] font-medium">创建图片 <span className="text-zinc-500 text-[12px] font-normal ml-2">可视化呈现任何内容</span></div>
          </div>
        </button>
        
        <button 
          onClick={() => { setInputMode('search'); setShowMenu(false); }}
          className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition-colors text-left group">
          <div className="flex items-center justify-center w-8 h-8 shrink-0">
            <Globe size={20} className="text-[#10b981]" />
          </div>
          <div className="flex-1">
            <div className="text-zinc-200 text-[14px] font-medium">网页搜索 <span className="text-zinc-500 text-[12px] font-normal ml-2">查找实时新闻和信息</span></div>
          </div>
        </button>
        
        <button 
          onClick={() => { setInputMode('research'); setShowMenu(false); }}
          className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition-colors text-left group">
          <div className="flex items-center justify-center w-8 h-8 shrink-0">
            <Telescope size={20} className="text-[#6366f1]" />
          </div>
          <div className="flex-1">
            <div className="text-zinc-200 text-[14px] font-medium">深度研究 <span className="text-zinc-500 text-[12px] font-normal ml-2">获取详细报告</span></div>
          </div>
        </button>

        <button className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition-colors text-left group">
          <div className="flex items-center justify-center w-6 h-6 rounded-full bg-[#10a37f] shrink-0 overflow-hidden">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" className="text-white">
              <path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A6.0651 6.0651 0 0 0 19.0193 19.818a5.9847 5.9847 0 0 0 3.9977-2.9 6.0462 6.0462 0 0 0-.735-7.0968zM8.7505 6.7392a4.469 4.469 0 0 1 7.4984 2.14l-4.524 2.6122-2.9744-4.7522zm2.0622 13.931a4.469 4.469 0 0 1-5.6983-4.5024l4.524-2.6121 2.9744 4.7521-1.8001 2.3624zm11.3986-7.3822a4.469 4.469 0 0 1-5.6983 4.5024l-1.8-2.3624 4.524-2.6122 2.9743 4.7522zm-7.4984 2.14l4.524-2.6122 2.9744 4.7522a4.469 4.469 0 0 1-7.4984-2.14zm-4.524 2.6122l2.9744-4.7522 1.8 2.3624-4.524 2.6122z" fill="currentColor"/>
            </svg>
          </div>
          <div className="flex-1 truncate">
            <div className="text-zinc-200 text-[14px] font-medium flex items-center gap-2">
              OpenAI Platform <span className="text-zinc-500 text-[12px] font-normal truncate">Create an OpenAI API key after connecting Platform.</span>
            </div>
          </div>
        </button>

        <button className="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition-colors text-left group">
          <div className="flex items-center justify-center w-6 h-6 rounded-full bg-red-600 text-white font-bold text-[10px] shrink-0">
            A
          </div>
          <div className="flex-1 truncate">
            <div className="text-zinc-200 text-[14px] font-medium flex items-center gap-2">
              AutoTrader <span className="text-zinc-500 text-[12px] font-normal truncate">AutoTrader - Canada's Most Trusted Marketplace to Buy and Sell Cars, Trucks, Boats & Mor...</span>
            </div>
          </div>
        </button>

        <div className="h-px bg-white/5 my-1 mx-4"></div>

        <div className="px-4 py-2 flex items-center text-[12px] text-zinc-500">
          <input type="text" placeholder="输入以搜索插件、文件和技能" className="w-full bg-transparent border-none focus:ring-0 outline-none text-zinc-200 placeholder-zinc-500 p-0" />
        </div>
      </div>
    </div>
  );
};
