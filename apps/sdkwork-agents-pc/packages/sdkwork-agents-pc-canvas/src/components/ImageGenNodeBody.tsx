import React from 'react';
import { Sparkles, Image as ImageIcon } from 'lucide-react';
import { CanvasNode } from '../types';
import { cn } from '@sdkwork/agents-pc-commons';

interface ImageGenNodeBodyProps {
  node: CanvasNode;
  connectedInputNode?: CanvasNode;
  onUpdate: (id: string, updates: Partial<CanvasNode>) => void;
  triggerGeneration: () => void;
}

export const ImageGenNodeBody: React.FC<ImageGenNodeBodyProps> = ({
  node,
}) => {
  return (
    <div className="flex flex-col flex-1 relative w-full h-full min-h-[160px]">
      {node.status === 'completed' && node.mediaUrl ? (
        <img 
          src={node.mediaUrl} 
          alt="Generated" 
          className="w-full h-full object-cover rounded-2xl pointer-events-none"
          referrerPolicy="no-referrer"
        />
      ) : node.status === 'generating' ? (
        <div className="absolute inset-0 flex flex-col items-center justify-center bg-black/10 rounded-2xl">
          <div className="relative w-10 h-10 flex items-center justify-center">
            <div className="absolute inset-0 border-[2px] border-yellow-500/10 rounded-full" />
            <div 
              style={{ strokeDasharray: 100, strokeDashoffset: 100 - (node.progress || 0) }}
              className="absolute inset-0 border-[2.5px] border-yellow-500 rounded-full animate-spin border-t-transparent" 
            />
            <Sparkles size={16} className="text-yellow-400 animate-pulse" />
          </div>
        </div>
      ) : (
        <div className="absolute inset-0 flex flex-col items-center justify-center text-zinc-500 bg-white/5 rounded-2xl border border-dashed border-white/10">
          <ImageIcon size={32} className="opacity-40 mb-2" />
          <span className="text-[10px] uppercase font-bold opacity-50 tracking-wider">Image Node</span>
        </div>
      )}
    </div>
  );
};

