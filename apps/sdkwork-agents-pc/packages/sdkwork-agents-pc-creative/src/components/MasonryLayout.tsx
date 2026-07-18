import React from 'react';
import { Play } from 'lucide-react';
import { CreativeMessage } from '../types';

interface MasonryLayoutProps {
  message: CreativeMessage;
  onPreviewImage: (message: CreativeMessage, index: number) => void;
}

export const MasonryLayout: React.FC<MasonryLayoutProps> = ({ message, onPreviewImage }) => {
  const imageUrls = message.imageUrls;
  const isVideo = message.mode === 'video';

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-3 h-[420px]" id="masonry-layout">
      {/* Main Large Item */}
      <div 
        className="md:col-span-2 relative h-full bg-[#1a1a1c] border border-white/5 rounded-2xl overflow-hidden group/card transition-all hover:scale-[1.005] hover:border-cyan-500/40 cursor-pointer shadow-xl"
        onClick={() => onPreviewImage(message, 0)}
      >
        <img src={imageUrls?.[0]} className="absolute inset-0 w-full h-full object-cover" alt="" referrerPolicy="no-referrer" />
        <div className="absolute top-3 left-3 bg-cyan-400/10 text-cyan-400 text-[10px] font-bold px-2 py-0.5 rounded border border-cyan-400/20 z-20">
          {isVideo ? '精选视频' : '主图设计'}
        </div>

        {/* Video Overlay */}
        {isVideo && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/20 group-hover/card:bg-black/30 transition-colors z-10">
            <div className="w-16 h-16 rounded-full bg-cyan-500 text-black flex items-center justify-center shadow-2xl transform scale-95 group-hover/card:scale-100 transition-transform duration-300">
              <Play size={24} fill="currentColor" className="ml-1" />
            </div>
          </div>
        )}

        <div className="absolute inset-0 bg-black/20 opacity-0 group-hover/card:opacity-100 transition-opacity flex items-end p-3 z-20">
          <span className="text-[10px] bg-black/75 backdrop-blur-sm px-2.5 py-1 rounded text-cyan-300 font-medium">
            {isVideo ? '点击播放精选视频' : '点击放大查看'}
          </span>
        </div>
      </div>

      {/* Grid for other 3 items */}
      <div className="grid grid-rows-3 gap-3 h-full">
        {imageUrls?.slice(1, 4).map((url, idx) => (
          <div 
            key={idx} 
            className="relative h-full bg-[#1a1a1c] border border-white/5 rounded-xl overflow-hidden group/card transition-all hover:scale-[1.01] hover:border-cyan-500/40 cursor-pointer shadow-md"
            onClick={() => onPreviewImage(message, idx + 1)}
          >
            <img src={url} className="absolute inset-0 w-full h-full object-cover" alt="" referrerPolicy="no-referrer" />

            {/* Video Overlay on mini items */}
            {isVideo && (
              <div className="absolute inset-0 flex items-center justify-center bg-black/20 group-hover/card:bg-black/30 transition-colors z-10">
                <div className="w-8 h-8 rounded-full bg-cyan-500 text-black flex items-center justify-center shadow-2xl transform scale-90 group-hover/card:scale-100 transition-transform duration-300">
                  <Play size={14} fill="currentColor" className="ml-0.5" />
                </div>
              </div>
            )}

            <div className="absolute inset-0 bg-black/20 opacity-0 group-hover/card:opacity-100 transition-opacity flex items-end p-2 z-20">
              <span className="text-[9px] bg-black/75 backdrop-blur-sm px-2 py-0.5 rounded text-cyan-300">
                {isVideo ? '播放' : '放大'}
              </span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
