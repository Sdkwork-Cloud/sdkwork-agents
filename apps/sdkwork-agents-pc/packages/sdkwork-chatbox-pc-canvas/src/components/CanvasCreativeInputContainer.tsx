import React from 'react';
import { CanvasNode } from '../types';
import { CreativeInputBox } from '@/packages/sdkwork-chatbox-pc-commons/src/components/CreativeInputBox';
import { getAdaptedHeight } from '../utils/ratioHelper';

interface CanvasCreativeInputContainerProps {
  selectedNodeIds: string[];
  nodes: CanvasNode[];
  onClearSelection: () => void;
  triggerNodeGeneration: (id: string, customPrompt?: string, customSettings?: any) => void;
  containerRef: React.RefObject<HTMLDivElement>;
  pan: { x: number; y: number };
  zoom: number;
  saveHistory: () => void;
  setNodes: React.Dispatch<React.SetStateAction<CanvasNode[]>>;
  setSelectedNodeIds: (ids: string[]) => void;
  setSmoothView: (newPan: { x: number; y: number }, newZoom: number) => void;
  handleInputPromptChange: (val: string) => void;
  handleInputModeChange: (mode: string) => void;
  handleInputSettingsChange: (settings: any) => void;
  isReadOnly?: boolean;
}

export const CanvasCreativeInputContainer: React.FC<CanvasCreativeInputContainerProps> = ({
  selectedNodeIds,
  nodes,
  onClearSelection,
  triggerNodeGeneration,
  containerRef,
  pan,
  zoom,
  saveHistory,
  setNodes,
  setSelectedNodeIds,
  setSmoothView,
  handleInputPromptChange,
  handleInputModeChange,
  handleInputSettingsChange,
  isReadOnly = false
}) => {
  const selectedNode = selectedNodeIds.length === 1 ? nodes.find(n => n.id === selectedNodeIds[0]) : null;

  if (isReadOnly) {
    return (
      <div className="absolute bottom-6 left-1/2 -translate-x-1/2 w-[840px] max-w-[95vw] z-30 flex flex-col gap-2.5 no-export select-none animate-in fade-in slide-in-from-bottom-2 duration-300">
        <div className="flex items-center justify-between gap-4 px-6 py-4 bg-[#141416]/95 border border-amber-500/30 backdrop-blur-md rounded-2xl shadow-2xl">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-xl bg-amber-500/10 flex items-center justify-center border border-amber-500/20 text-amber-400 shrink-0">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="lucide lucide-lock"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
            </div>
            <div className="flex flex-col">
              <span className="text-sm font-bold text-zinc-100">演示与评审模式 (只读已开启)</span>
              <span className="text-[11px] text-zinc-400 font-medium">当前画布已被锁定。无法对节点进行创建、内容修改、拖拽或大小调整。请在顶部工具栏解锁。</span>
            </div>
          </div>
          <div className="text-xs text-amber-400 font-bold bg-amber-500/5 px-2.5 py-1 rounded-lg border border-amber-500/10 shrink-0 select-none">
            画布锁定保护中
          </div>
        </div>
      </div>
    );
  }

  const nodeTitles = {
    'text': '📝 正在编辑：文本创意',
    'image-gen': '🎨 正在编辑：AI 创意图源',
    'video-gen': '🎬 正在编辑：AI 镜头渲染'
  };

  const initialMode = selectedNode 
    ? (selectedNode.type === 'image-gen' ? 'image' : selectedNode.type === 'video-gen' ? 'video' : 'agent')
    : 'image'; // default to image
    
  const initialVal = selectedNode 
    ? (selectedNode.prompt || selectedNode.content || '') 
    : '';

  const initialSettings = selectedNode ? {
    model: selectedNode.model,
    ratio: selectedNode.ratio,
    resolution: selectedNode.resolution,
    duration: selectedNode.duration,
    videoMode: selectedNode.videoMode,
    count: selectedNode.count
  } : undefined;

  const handleSubmit = (val: string, mode: string, settings: any) => {
    if (selectedNode) {
      triggerNodeGeneration(selectedNode.id, val, settings);
    } else {
      if (!containerRef.current) return;
      const rect = containerRef.current.getBoundingClientRect();
      const centerWorldX = (rect.width / 2 - pan.x) / zoom;
      const centerWorldY = (rect.height / 2 - pan.y) / zoom;

      const newNodeId = `node-${Date.now()}`;
      const targetType = mode === 'image' ? 'image-gen' : mode === 'video' ? 'video-gen' : 'text';
      const titles = {
        'text': '创意大纲草稿',
        'image-gen': 'AI 创意图源',
        'video-gen': 'AI 镜头渲染'
      };

      const isImageOrVideo = targetType === 'image-gen' || targetType === 'video-gen';
      const targetWidth = isImageOrVideo ? 260 : 320;
      let targetHeight = targetType === 'text' ? 250 : targetType === 'image-gen' ? 280 : 190;
      if (isImageOrVideo) {
        targetHeight = getAdaptedHeight(targetType, targetWidth, settings?.ratio || '1:1');
      }

      // Filter out width, height, imageWidth, and imageHeight to prevent any layout size corruption
      const { width, height, imageWidth, imageHeight, ...cleanSettings } = settings || {};

      const newNode: CanvasNode = {
        id: newNodeId,
        type: targetType,
        x: Math.round(centerWorldX - targetWidth / 2),
        y: Math.round(centerWorldY - targetHeight / 2),
        width: targetWidth,
        height: targetHeight,
        title: `${titles[targetType]} #${nodes.length + 1}`,
        prompt: val,
        status: 'idle',
        ...cleanSettings
      };

      saveHistory();
      setNodes(prev => [...prev, newNode]);
      setSelectedNodeIds([newNodeId]);

      setTimeout(() => {
        triggerNodeGeneration(newNodeId, val, settings);
      }, 100);

      const targetPanX = rect.width / 2 - centerWorldX * zoom;
      const targetPanY = rect.height / 2 - centerWorldY * zoom;
      setSmoothView({ x: targetPanX, y: targetPanY }, zoom);
    }
  };

  return (
    <div className="absolute bottom-6 left-1/2 -translate-x-1/2 w-[840px] max-w-[95vw] z-30 flex flex-col gap-2.5 no-export">
      {selectedNodeIds.length === 1 && selectedNode && (
        <div className="flex items-center justify-between px-4 py-1.5 bg-[#121214]/90 backdrop-blur-md border border-cyan-500/30 rounded-xl text-xs text-cyan-400 font-semibold shadow-lg select-none animate-in fade-in slide-in-from-bottom-1 duration-150">
          <div className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
            <span>{nodeTitles[selectedNode.type] || '📝 正在编辑：画布节点'} (ID: {selectedNode.id})</span>
          </div>
          <button 
            onClick={onClearSelection}
            className="px-2 py-0.5 hover:bg-white/10 rounded-md text-zinc-400 hover:text-white transition-colors cursor-pointer text-[10px]"
          >
            取消选择
          </button>
        </div>
      )}
      
      <CreativeInputBox
        key={selectedNode ? `node-${selectedNode.id}` : 'global'}
        defaultValue={initialVal}
        initialMode={initialMode}
        initialSettings={initialSettings}
        onSubmit={handleSubmit}
        onChange={handleInputPromptChange}
        onModeChange={handleInputModeChange}
        onSettingsChange={handleInputSettingsChange}
        className="w-full shadow-2xl border-white/10"
      />
    </div>
  );
};
