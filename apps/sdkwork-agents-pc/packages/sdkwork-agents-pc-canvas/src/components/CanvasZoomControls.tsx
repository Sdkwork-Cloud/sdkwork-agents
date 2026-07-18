import React from 'react';
import { 
  ZoomIn, 
  ZoomOut, 
  Maximize2, 
  RefreshCw,
  Maximize,
  Map
} from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';

interface CanvasZoomControlsProps {
  zoom: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onResetZoom: () => void;
  onZoomToFit: () => void;
  showMinimap?: boolean;
  onToggleMinimap?: () => void;
}

export const CanvasZoomControls: React.FC<CanvasZoomControlsProps> = ({
  zoom,
  onZoomIn,
  onZoomOut,
  onResetZoom,
  onZoomToFit,
  showMinimap = true,
  onToggleMinimap
}) => {
  const percentage = Math.round(zoom * 100);

  return (
    <div 
      className={cn(
        "flex items-center gap-1.5 bg-[#141416]/95 border border-white/10 backdrop-blur-md p-1.5 rounded-xl shadow-2xl z-40 select-none transition-all duration-300 ease-out"
      )}
      id="canvas-zoom-controls-panel"
    >
      {/* Zoom Out Button */}
      <button
        onClick={onZoomOut}
        className="w-8 h-8 rounded-lg flex items-center justify-center text-zinc-400 hover:text-white hover:bg-white/5 transition-all cursor-pointer relative group"
        id="zoom-out-btn"
        title="缩小"
      >
        <ZoomOut size={15} />
        <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 bg-[#121214] border border-white/10 text-[10px] text-zinc-300 px-2 py-1 rounded shadow-xl opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
          缩小 (Zoom Out)
        </span>
      </button>

      {/* Percentage Indicator & Reset to 100% */}
      <button
        onClick={onResetZoom}
        className={cn(
          "px-2 h-8 rounded-lg text-[11px] font-mono font-semibold transition-all cursor-pointer relative group flex items-center justify-center gap-1 min-w-[56px]",
          percentage === 100 
            ? "text-cyan-400 bg-cyan-500/10 border border-cyan-500/15" 
            : "text-zinc-300 hover:text-white hover:bg-white/5 border border-transparent"
        )}
        id="reset-zoom-100-btn"
        title="重置缩放至 100%"
      >
        <span>{percentage}%</span>
        <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 bg-[#121214] border border-white/10 text-[10px] text-zinc-300 px-2 py-1 rounded shadow-xl opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
          重置为 100% (Reset Zoom)
        </span>
      </button>

      {/* Zoom In Button */}
      <button
        onClick={onZoomIn}
        className="w-8 h-8 rounded-lg flex items-center justify-center text-zinc-400 hover:text-white hover:bg-white/5 transition-all cursor-pointer relative group"
        id="zoom-in-btn"
        title="放大"
      >
        <ZoomIn size={15} />
        <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 bg-[#121214] border border-white/10 text-[10px] text-zinc-300 px-2 py-1 rounded shadow-xl opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
          放大 (Zoom In)
        </span>
      </button>

      {/* Separator */}
      <div className="w-[1px] h-4 bg-white/10 mx-0.5"></div>

      {/* Zoom to Fit / Reset View */}
      <button
        onClick={onZoomToFit}
        className="w-8 h-8 rounded-lg flex items-center justify-center text-zinc-400 hover:text-white hover:bg-white/5 transition-all cursor-pointer relative group"
        id="zoom-to-fit-btn"
        title="适应画布"
      >
        <Maximize size={14} />
        <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 bg-[#121214] border border-white/10 text-[10px] text-zinc-300 px-2 py-1 rounded shadow-xl opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
          适应全部 (Zoom to Fit)
        </span>
      </button>

      {/* Separator */}
      <div className="w-[1px] h-4 bg-white/10 mx-0.5"></div>

      {/* Toggle Minimap Button */}
      <button
        onClick={onToggleMinimap}
        className={cn(
          "w-8 h-8 rounded-lg flex items-center justify-center transition-all cursor-pointer relative group",
          showMinimap 
            ? "text-cyan-400 bg-cyan-500/10 hover:bg-cyan-500/15" 
            : "text-zinc-400 hover:text-white hover:bg-white/5"
        )}
        id="toggle-minimap-btn"
        title={showMinimap ? "隐藏小地图" : "显示小地图"}
      >
        <Map size={14} />
        <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 bg-[#121214] border border-white/10 text-[10px] text-zinc-300 px-2 py-1 rounded shadow-xl opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
          {showMinimap ? "隐藏小地图" : "显示小地图"}
        </span>
      </button>

    </div>
  );
};
