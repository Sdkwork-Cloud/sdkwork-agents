import React from 'react';
import { X, Github, Slack, Triangle } from 'lucide-react';

interface BirdCoderModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const BirdCoderModal: React.FC<BirdCoderModalProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
      <div 
        className="w-[600px] bg-[#222222] rounded-2xl overflow-hidden shadow-2xl relative animate-in zoom-in-95 duration-200 flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        <button 
          onClick={onClose}
          className="absolute top-4 right-4 z-10 p-2 text-black/50 hover:text-black hover:bg-white/10 rounded-full transition-colors"
        >
          <X size={20} />
        </button>

        {/* Top Graphic Section */}
        <div className="h-[300px] w-full bg-gradient-to-br from-[#74baff] via-[#9dcbfb] to-[#c7d9fc] flex items-center justify-center relative p-8">
          <div className="bg-white/60 backdrop-blur-md rounded-2xl p-6 shadow-xl w-full max-w-md">
            <div className="text-[#333] text-[17px] leading-[1.8] font-medium text-center">
              Use 
              <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-white/80 rounded-full mx-2 shadow-sm text-[15px]">
                <div className="flex flex-col gap-[2px]">
                   <div className="w-3.5 h-[2px] bg-indigo-500 rounded-full"></div>
                   <div className="w-3.5 h-[2px] bg-indigo-500 rounded-full ml-1"></div>
                </div>
                <span className="text-indigo-600 font-semibold">Linear</span>
              </span> 
              to find the ticket from this 
              <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-white/80 rounded-full mx-2 shadow-sm text-[15px]">
                <Slack size={16} className="text-blue-600" />
                <span className="text-blue-600 font-semibold">Slack</span>
              </span> 
              thread, then use 
              <span className="inline-flex items-center gap-1.5 px-3 py-1 bg-white/80 rounded-full mx-2 mt-2 shadow-sm text-[15px]">
                <Github size={16} className="text-zinc-800" />
                <span className="text-purple-600 font-semibold">GitHub</span>
              </span> 
              to open a PR
            </div>
          </div>
        </div>

        {/* Bottom Content Section */}
        <div className="bg-[#222222] p-10 flex flex-col items-center text-center">
          <h2 className="text-[26px] font-bold text-white mb-4">用 BirdCoder 构建应用、网站和工具</h2>
          <p className="text-[15px] text-zinc-400 leading-relaxed mb-8 max-w-md mx-auto">
            从一个想法开始，其余的交给 BirdCoder。它可以更新代码、文档和项目文件，然后向你展示变更内容，方便你审阅并继续推进。
          </p>
          
          <div className="flex items-center gap-4 w-full justify-center">
            <button className="px-6 py-2.5 rounded-full border border-zinc-600 text-zinc-300 hover:bg-white/5 hover:text-white transition-colors text-[15px] font-medium min-w-[140px]">
              了解更多
            </button>
            <button className="px-6 py-2.5 rounded-full bg-white text-black hover:bg-zinc-200 transition-colors text-[15px] font-medium min-w-[140px]">
              下载 Windows 版
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
