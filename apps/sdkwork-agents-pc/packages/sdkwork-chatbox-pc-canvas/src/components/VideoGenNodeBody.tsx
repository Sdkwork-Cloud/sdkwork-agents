import React from 'react';
import { Play, Pause, Video as VideoIcon } from 'lucide-react';
import { CanvasNode } from '../types';

interface VideoGenNodeBodyProps {
  node: CanvasNode;
  connectedInputNode?: CanvasNode;
  onUpdate: (id: string, updates: Partial<CanvasNode>) => void;
  triggerGeneration: () => void;
  videoRef: React.RefObject<HTMLVideoElement>;
  isPlaying: boolean;
  handleVideoPlayToggle: (e: React.MouseEvent) => void;
}

export const VideoGenNodeBody: React.FC<VideoGenNodeBodyProps> = ({
  node,
  videoRef,
  isPlaying,
  handleVideoPlayToggle
}) => {
  return (
    <div className="flex flex-col flex-1 relative w-full h-full min-h-[160px]">
      {node.status === 'completed' && node.mediaUrl ? (
        <div className="relative w-full h-full rounded-2xl overflow-hidden group/video bg-black/40 border border-white/5 pointer-events-auto">
          <video
            ref={videoRef}
            src={node.mediaUrl}
            loop
            muted
            playsInline
            className="w-full h-full object-cover"
          />
          <div className="absolute inset-0 bg-black/40 opacity-0 group-hover/video:opacity-100 transition-opacity flex items-center justify-center">
            <button
              onClick={handleVideoPlayToggle}
              className="w-12 h-12 rounded-full bg-indigo-500/80 hover:bg-indigo-500 text-white flex items-center justify-center shadow-xl transition-transform hover:scale-105 cursor-pointer backdrop-blur-sm"
            >
              {isPlaying ? <Pause size={20} fill="currentColor" /> : <Play size={20} fill="currentColor" className="ml-1" />}
            </button>
          </div>
        </div>
      ) : node.status === 'generating' ? (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-black/10 rounded-2xl">
          <div className="relative w-10 h-10 flex items-center justify-center">
            <div className="absolute inset-0 border-[2px] border-indigo-500/10 rounded-full" />
            <div 
              style={{ strokeDasharray: 100, strokeDashoffset: 100 - (node.progress || 0) }}
              className="absolute inset-0 border-[2.5px] border-indigo-500 rounded-full animate-spin border-t-transparent" 
            />
            <Play size={14} className="text-indigo-400 animate-pulse ml-0.5" fill="currentColor" />
          </div>
        </div>
      ) : (
        <div className="absolute inset-0 flex flex-col items-center justify-center text-zinc-500 bg-white/5 rounded-2xl border border-dashed border-white/10">
          <VideoIcon size={32} className="opacity-40 mb-2" />
          <span className="text-[10px] uppercase font-bold opacity-50 tracking-wider">Video Node</span>
        </div>
      )}
    </div>
  );
};

