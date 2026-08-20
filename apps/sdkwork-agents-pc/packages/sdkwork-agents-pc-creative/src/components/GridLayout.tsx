import React from 'react';
import { Play } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { CreativeMessage } from '../types';

interface GridLayoutProps {
  message: CreativeMessage;
  onPreviewImage: (message: CreativeMessage, index: number) => void;
}

export const GridLayout: React.FC<GridLayoutProps> = ({ message, onPreviewImage }) => {
  const imageUrls = message.imageUrls;
  const isVideo = message.mode === 'video';

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-3" id="grid-layout">
      {imageUrls && imageUrls.map((url, index) => (
        <div 
          key={index} 
          onClick={() => onPreviewImage(message, index)}
          className="relative aspect-square bg-white border border-black/5 rounded-xl overflow-hidden shadow-lg group/card transition-all duration-300 hover:scale-[1.01] hover:border-cyan-500/40 cursor-pointer dark:bg-[#1a1a1c] dark:border-white/5"
        >
          <img 
            src={url} 
            alt={`Generated variant ${index + 1}`} 
            referrerPolicy="no-referrer"
            className="absolute inset-0 w-full h-full object-cover"
          />

          {/* Centered Play button for video generation */}
          {isVideo && (
            <div className="absolute inset-0 flex items-center justify-center bg-black/20 group-hover/card:bg-black/30 transition-colors z-10">
              <div className="w-10 h-10 rounded-full bg-cyan-500 text-black flex items-center justify-center shadow-2xl transform scale-90 group-hover/card:scale-100 transition-transform duration-300">
                <Play size={16} fill="currentColor" className="ml-0.5" />
              </div>
            </div>
          )}

          {/* Hover click action overlay */}
          <div className="absolute inset-0 bg-black/20 opacity-0 group-hover/card:opacity-100 transition-opacity flex items-end p-2 z-20">
            <span className="text-[10px] bg-black/75 backdrop-blur-sm px-2 py-1 rounded text-cyan-300 font-medium">
              {isVideo ? '点击播放视频' : '点击放大'}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
};
