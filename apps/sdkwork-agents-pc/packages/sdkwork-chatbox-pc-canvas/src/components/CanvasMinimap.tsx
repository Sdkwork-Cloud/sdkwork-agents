import React, { useRef, useEffect, useState } from 'react';
import { CanvasNode, CanvasGroup, Connection } from '../types';
import { Map } from 'lucide-react';

interface CanvasMinimapProps {
  nodes: CanvasNode[];
  groups: CanvasGroup[];
  connections?: Connection[];
  pan: { x: number, y: number };
  zoom: number;
  viewportWidth: number;
  viewportHeight: number;
  onPanChange: (x: number, y: number) => void;
}

export const CanvasMinimap: React.FC<CanvasMinimapProps> = ({
  nodes,
  groups,
  connections = [],
  pan,
  zoom,
  viewportWidth,
  viewportHeight,
  onPanChange
}) => {
  const draggingRef = useRef(false);
  const startMouseRef = useRef({ x: 0, y: 0 });
  const startPanRef = useRef({ x: 0, y: 0 });

  // Compute bounds of all content
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  nodes.forEach(node => {
    const nodeW = typeof node.width === 'number' && !isNaN(node.width) ? node.width : 260;
    const nodeH = node.isCollapsed ? 36 : (typeof node.height === 'number' && !isNaN(node.height) ? node.height : 250);
    minX = Math.min(minX, node.x);
    minY = Math.min(minY, node.y);
    maxX = Math.max(maxX, node.x + nodeW);
    maxY = Math.max(maxY, node.y + nodeH);
  });

  groups.forEach(group => {
    minX = Math.min(minX, group.x);
    minY = Math.min(minY, group.y);
    maxX = Math.max(maxX, group.x + (group.width || 400));
    maxY = Math.max(maxY, group.y + (group.height || 300));
  });

  // Include viewport in bounds calculation so we can see the current view even if there's no content
  const viewMinX = -pan.x / zoom;
  const viewMinY = -pan.y / zoom;
  const viewMaxX = viewMinX + viewportWidth / zoom;
  const viewMaxY = viewMinY + viewportHeight / zoom;

  minX = Math.min(minX, viewMinX);
  minY = Math.min(minY, viewMinY);
  maxX = Math.max(maxX, viewMaxX);
  maxY = Math.max(maxY, viewMaxY);

  if (minX === Infinity) {
    minX = 0; minY = 0; maxX = 1000; maxY = 1000;
  }

  // Pad the bounds a bit
  const padding = 200;
  minX -= padding;
  minY -= padding;
  maxX += padding;
  maxY += padding;

  const contentWidth = maxX - minX;
  const contentHeight = maxY - minY;

  // Minimap dimensions (fixed size, say 180x120 for mapping area)
  const mapWidth = 180;
  const mapHeight = 120;
  const scale = Math.min(mapWidth / contentWidth, mapHeight / contentHeight);

  // Center it in the map widget
  const offsetX = (mapWidth - contentWidth * scale) / 2;
  const offsetY = (mapHeight - contentHeight * scale) / 2;

  const handleMapClick = (e: React.MouseEvent<HTMLDivElement>) => {
    // If we're clicking specifically on the drag handle, don't trigger general map click centering
    const target = e.target as HTMLElement;
    if (target.closest('.viewport-drag-handle')) return;

    const rect = e.currentTarget.getBoundingClientRect();
    const mapClickX = e.clientX - rect.left;
    const mapClickY = e.clientY - rect.top;

    // Inverse map to content coordinates
    const contentX = (mapClickX - offsetX) / scale + minX;
    const contentY = (mapClickY - offsetY) / scale + minY;

    // Set pan so the clicked content coordinate is in the center of the viewport
    onPanChange(
      viewportWidth / 2 - contentX * zoom,
      viewportHeight / 2 - contentY * zoom
    );
  };

  const handleViewportMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
    e.preventDefault();
    draggingRef.current = true;
    startMouseRef.current = { x: e.clientX, y: e.clientY };
    startPanRef.current = { x: pan.x, y: pan.y };
  };

  useEffect(() => {
    const handleGlobalMouseMove = (e: MouseEvent) => {
      if (!draggingRef.current) return;
      const dx = e.clientX - startMouseRef.current.x;
      const dy = e.clientY - startMouseRef.current.y;

      // Map displacement divided by scale equals content coordinate displacement
      const contentDX = dx / scale;
      const contentDY = dy / scale;

      // Panning represents the offset of the viewport origin relative to content origin.
      const newPanX = startPanRef.current.x - contentDX * zoom;
      const newPanY = startPanRef.current.y - contentDY * zoom;

      onPanChange(newPanX, newPanY);
    };

    const handleGlobalMouseUp = () => {
      draggingRef.current = false;
    };

    window.addEventListener('mousemove', handleGlobalMouseMove);
    window.addEventListener('mouseup', handleGlobalMouseUp);
    return () => {
      window.removeEventListener('mousemove', handleGlobalMouseMove);
      window.removeEventListener('mouseup', handleGlobalMouseUp);
    };
  }, [scale, zoom, onPanChange]);

  return (
    <div 
      className="w-[180px] h-[120px] bg-[#141416]/95 border border-white/10 backdrop-blur-md rounded-xl shadow-2xl overflow-hidden relative transition-all duration-300"
    >
      {/* Map Content */}
      <div 
        className="w-full h-full relative overflow-hidden cursor-pointer"
        onClick={handleMapClick}
      >
        <div className="relative w-full h-full pointer-events-none">
          {/* Draw Connections */}
          {connections && connections.length > 0 && (
            <svg className="absolute inset-0 w-full h-full pointer-events-none">
              {connections.map(conn => {
                const fromNode = nodes.find(n => n.id === conn.fromNodeId);
                const toNode = nodes.find(n => n.id === conn.toNodeId);
                if (!fromNode || !toNode) return null;

                const fromNodeW = typeof fromNode.width === 'number' && !isNaN(fromNode.width) ? fromNode.width : 260;
                const fromNodeH = fromNode.isCollapsed ? 36 : (typeof fromNode.height === 'number' && !isNaN(fromNode.height) ? fromNode.height : 250);
                const toNodeH = toNode.isCollapsed ? 36 : (typeof toNode.height === 'number' && !isNaN(toNode.height) ? toNode.height : 250);

                const sX = offsetX + ((fromNode.x + fromNodeW) - minX) * scale;
                const sY = offsetY + ((fromNode.y + fromNodeH / 2) - minY) * scale;
                const eX = offsetX + (toNode.x - minX) * scale;
                const eY = offsetY + ((toNode.y + toNodeH / 2) - minY) * scale;

                return (
                  <line
                    key={`map-conn-${conn.id}`}
                    x1={sX}
                    y1={sY}
                    x2={eX}
                    y2={eY}
                    stroke="rgba(255, 255, 255, 0.15)"
                    strokeWidth={1}
                  />
                );
              })}
            </svg>
          )}

          {/* Draw Groups */}
          {groups.map(group => (
            <div
              key={`map-group-${group.id}`}
              className="absolute border border-white/15 bg-white/5 rounded-sm"
              style={{
                left: offsetX + (group.x - minX) * scale,
                top: offsetY + (group.y - minY) * scale,
                width: (group.width || 400) * scale,
                height: (group.isCollapsed ? 40 : (group.height || 300)) * scale,
              }}
            />
          ))}

          {/* Draw Nodes */}
          {nodes.map(node => {
            const nodeW = typeof node.width === 'number' && !isNaN(node.width) ? node.width : 260;
            const nodeH = node.isCollapsed ? 36 : (typeof node.height === 'number' && !isNaN(node.height) ? node.height : 250);
            return (
              <div
                key={`map-node-${node.id}`}
                className="absolute bg-cyan-500/40 border border-cyan-400/10 rounded-sm"
                style={{
                  left: offsetX + (node.x - minX) * scale,
                  top: offsetY + (node.y - minY) * scale,
                  width: nodeW * scale,
                  height: nodeH * scale,
                }}
              />
            );
          })}

          {/* Draw Viewport Rect */}
          <div
            onMouseDown={handleViewportMouseDown}
            className="viewport-drag-handle absolute border border-yellow-500/80 bg-yellow-500/10 shadow-[0_0_8px_rgba(234,179,8,0.15)] pointer-events-auto cursor-grab active:cursor-grabbing hover:bg-yellow-500/20 transition-colors"
            style={{
              left: offsetX + (viewMinX - minX) * scale,
              top: offsetY + (viewMinY - minY) * scale,
              width: (viewportWidth / zoom) * scale,
              height: (viewportHeight / zoom) * scale,
            }}
          />
        </div>
      </div>
    </div>
  );
};
