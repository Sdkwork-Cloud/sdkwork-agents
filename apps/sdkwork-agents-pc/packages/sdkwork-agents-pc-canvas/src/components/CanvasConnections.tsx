import React from 'react';
import { Connection, CanvasNode } from '../types';
import { cn } from '@sdkwork/agents-pc-commons';

interface CanvasConnectionsProps {
  connections: Connection[];
  nodes: CanvasNode[];
  activePortDrag: { nodeId: string, type: 'input' | 'output', startX: number, startY: number } | null;
  portDragCurrentPos: { x: number, y: number };
  snappingTargetNodeId: string | null;
  selectedConnectionId: string | null;
  onSelectConnection: (e: React.MouseEvent, id: string) => void;
  onContextMenuConnection?: (e: React.MouseEvent, id: string) => void;
  onStartDragControlPoint?: (e: React.MouseEvent, connectionId: string) => void;
  draggingConnectionId?: string | null;
  onResetControlPoint?: (connectionId: string) => void;
}

const calculateBezierPath = (startX: number, startY: number, endX: number, endY: number) => {
  const dx = Math.abs(endX - startX) * 0.5;
  return `M ${startX} ${startY} C ${startX + dx} ${startY}, ${endX - dx} ${endY}, ${endX} ${endY}`;
};

const getObstacleAvoidingPath = (
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  nodes: CanvasNode[],
  fromNodeId: string,
  toNodeId: string,
  customControlPoint?: { x: number; y: number }
) => {
  if (customControlPoint) {
    return {
      path: `M ${startX} ${startY} Q ${customControlPoint.x} ${customControlPoint.y} ${endX} ${endY}`,
      cpX: customControlPoint.x,
      cpY: customControlPoint.y,
      isRouted: true
    };
  }

  // Find any obstacle node that intersects the direct line from (startX, startY) to (endX, endY)
  const lineSegmentsIntersect = (x1: number, y1: number, x2: number, y2: number, x3: number, y3: number, x4: number, y4: number): boolean => {
    const det = (x2 - x1) * (y4 - y3) - (y2 - y1) * (x4 - x3);
    if (det === 0) return false;
    const lambda = ((y4 - y3) * (x4 - x1) + (x3 - x4) * (y4 - y1)) / det;
    const gamma = ((y1 - y2) * (x4 - x1) + (x2 - x1) * (y4 - y1)) / det;
    return (0 <= lambda && lambda <= 1) && (0 <= gamma && gamma <= 1);
  };

  const obstacles = nodes.filter(node => {
    if (node.id === fromNodeId || node.id === toNodeId) return false;
    
    const w = typeof node.width === 'number' && !isNaN(node.width) ? node.width : 260;
    const h = node.isCollapsed ? 36 : (typeof node.height === 'number' && !isNaN(node.height) ? node.height : 250);
    
    // Add a buffer around the node to prevent close scrapes
    const buffer = 40;
    const left = node.x - buffer;
    const right = node.x + w + buffer;
    const top = node.y - buffer;
    const bottom = node.y + h + buffer;

    // Check if the line segment intersects any of the four buffered rectangle boundaries
    const intersectsAnyEdge = 
      lineSegmentsIntersect(startX, startY, endX, endY, left, top, right, top) ||
      lineSegmentsIntersect(startX, startY, endX, endY, left, bottom, right, bottom) ||
      lineSegmentsIntersect(startX, startY, endX, endY, left, top, left, bottom) ||
      lineSegmentsIntersect(startX, startY, endX, endY, right, top, right, bottom);

    if (intersectsAnyEdge) return true;

    // Check if start/end points are inside (just in case they overlap)
    const startInside = startX >= left && startX <= right && startY >= top && startY <= bottom;
    const endInside = endX >= left && endX <= right && endY >= top && endY <= bottom;

    return startInside || endInside;
  });

  if (obstacles.length === 0) {
    const dx = Math.abs(endX - startX) * 0.5;
    return {
      path: `M ${startX} ${startY} C ${startX + dx} ${startY}, ${endX - dx} ${endY}, ${endX} ${endY}`,
      cpX: (startX + endX) / 2,
      cpY: (startY + endY) / 2,
      isRouted: false
    };
  }

  // Find the obstacle closest to the center of the path
  const midX = (startX + endX) / 2;
  const midY = (startY + endY) / 2;
  
  obstacles.sort((a, b) => {
    const aW = typeof a.width === 'number' && !isNaN(a.width) ? a.width : 260;
    const aH = a.isCollapsed ? 36 : (typeof a.height === 'number' && !isNaN(a.height) ? a.height : 250);
    const bW = typeof b.width === 'number' && !isNaN(b.width) ? b.width : 260;
    const bH = b.isCollapsed ? 36 : (typeof b.height === 'number' && !isNaN(b.height) ? b.height : 250);

    const distA = Math.hypot((a.x + aW/2) - midX, (a.y + aH/2) - midY);
    const distB = Math.hypot((b.x + bW/2) - midX, (b.y + bH/2) - midY);
    return distA - distB;
  });

  const obstacle = obstacles[0];
  const obW = typeof obstacle.width === 'number' && !isNaN(obstacle.width) ? obstacle.width : 260;
  const obH = obstacle.isCollapsed ? 36 : (typeof obstacle.height === 'number' && !isNaN(obstacle.height) ? obstacle.height : 250);
  const cx = obstacle.x + obW / 2;
  const cy = obstacle.y + obH / 2;

  let bypassX = cx;
  let bypassY = cy;
  const bufferDist = 80; // Distance away from the obstacle edge

  if (Math.abs(endX - startX) > 10) {
    const lineYAtCx = startY + (endY - startY) * (cx - startX) / (endX - startX);
    if (lineYAtCx < cy) {
      bypassY = obstacle.y - bufferDist;
    } else {
      bypassY = obstacle.y + obH + bufferDist;
    }
  } else {
    if (startX < cx) {
      bypassX = obstacle.x - bufferDist;
    } else {
      bypassX = obstacle.x + obW + bufferDist;
    }
  }

  // Calculate quadratic Bezier control point
  const cpX = 2 * bypassX - 0.5 * startX - 0.5 * endX;
  const cpY = 2 * bypassY - 0.5 * startY - 0.5 * endY;

  return {
    path: `M ${startX} ${startY} Q ${cpX} ${cpY} ${endX} ${endY}`,
    cpX,
    cpY,
    isRouted: true
  };
};

export const CanvasConnections: React.FC<CanvasConnectionsProps> = ({
  connections,
  nodes,
  activePortDrag,
  portDragCurrentPos,
  snappingTargetNodeId,
  selectedConnectionId,
  onSelectConnection,
  onContextMenuConnection,
  onStartDragControlPoint,
  draggingConnectionId,
  onResetControlPoint
}) => {
  return (
    <svg className="absolute inset-0 w-full h-full pointer-events-none z-20 grid-svg" style={{ overflow: 'visible' }}>
      <defs>
        <marker id="arrowhead" markerWidth="6" markerHeight="4" refX="5" refY="2" orient="auto">
          <path d="M0,0 L6,2 L0,4" fill="#3b82f6" />
        </marker>
        <marker id="arrowhead-selected" markerWidth="6" markerHeight="4" refX="5" refY="2" orient="auto">
          <path d="M0,0 L6,2 L0,4" fill="#22d3ee" />
        </marker>
        <marker id="arrowhead-dragging" markerWidth="6" markerHeight="4" refX="5" refY="2" orient="auto">
          <path d="M0,0 L6,2 L0,4" fill="#3b82f6" />
        </marker>
        
        {/* Animated Gradient for Data Flow */}
        <linearGradient id="flow-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor="#3b82f6" stopOpacity="0.8" />
          <stop offset="50%" stopColor="#22d3ee" stopOpacity="1" />
          <stop offset="100%" stopColor="#3b82f6" stopOpacity="0.8" />
        </linearGradient>
      </defs>
      
      {connections.map(conn => {
        const fromNode = nodes.find(n => n.id === conn.fromNodeId);
        const toNode = nodes.find(n => n.id === conn.toNodeId);
        if (!fromNode || !toNode) return null;

        const fromW = typeof fromNode.width === 'number' && !isNaN(fromNode.width) ? fromNode.width : 260;
        const fromH = fromNode.isCollapsed ? 36 : (typeof fromNode.height === 'number' && !isNaN(fromNode.height) ? fromNode.height : 250);
        const toH = toNode.isCollapsed ? 36 : (typeof toNode.height === 'number' && !isNaN(toNode.height) ? toNode.height : 250);

        const startX = fromNode.x + fromW;
        const startY = fromNode.y + (fromH / 2);
        
        const endX = toNode.x;
        const endY = toNode.y + (toH / 2);

        const isSelected = selectedConnectionId === conn.id;
        
        // Check if either node is actively generating to enhance the flow line
        const isGenerating = fromNode.status === 'generating' || toNode.status === 'generating';

        // Calculate routed / non-routed path and control points
        const { path: pathData, cpX, cpY } = getObstacleAvoidingPath(
          startX,
          startY,
          endX,
          endY,
          nodes,
          conn.fromNodeId,
          conn.toNodeId,
          conn.controlPoint
        );

        return (
          <g 
            key={conn.id} 
            className="group cursor-pointer pointer-events-auto"
            onClick={(e) => onSelectConnection(e, conn.id)}
            onContextMenu={(e) => onContextMenuConnection?.(e, conn.id)}
          >
            {/* Invisible thicker path for easier clicking */}
            <path
              d={pathData}
              fill="none"
              stroke="transparent"
              strokeWidth={20}
              className="transition-colors"
            />
            
            {/* Background path to show the track */}
            <path
              d={pathData}
              fill="none"
              stroke={isSelected ? '#22d3ee' : '#27272a'}
              strokeWidth={isSelected ? 4 : 2}
              className={isSelected ? "drop-shadow-[0_0_8px_rgba(34,211,238,0.5)]" : "transition-colors"}
            />

            {/* Foreground path for the animated data flow dashes */}
            <path
              d={pathData}
              fill="none"
              stroke={isGenerating ? "url(#flow-gradient)" : isSelected ? '#22d3ee' : '#3b82f6'}
              strokeWidth={isSelected ? 4 : 2}
              markerEnd={isSelected || isGenerating ? "url(#arrowhead-selected)" : "url(#arrowhead)"}
              className={cn(
                "transition-all duration-300",
                (isGenerating || isSelected) ? "opacity-100 flow-line-animated drop-shadow-[0_0_5px_rgba(34,211,238,0.6)]" : "opacity-60 hover:opacity-100",
                !isGenerating && !isSelected && "flow-line-animated-slow"
              )}
            />

            {/* Interactive Control Point Drag Handle */}
            <circle
              cx={cpX}
              cy={cpY}
              r={6}
              fill={isSelected ? "#22d3ee" : "#3b82f6"}
              stroke="#0d0d0e"
              strokeWidth={2}
              className={cn(
                "cursor-grab active:cursor-grabbing pointer-events-auto transition-all hover:scale-150 z-50",
                isSelected ? "opacity-100" : "opacity-0 group-hover:opacity-100 scale-90 hover:opacity-100",
                draggingConnectionId === conn.id && "scale-150 fill-cyan-400 animate-pulse"
              )}
              onMouseDown={(e) => {
                e.stopPropagation();
                e.preventDefault();
                onStartDragControlPoint?.(e, conn.id);
              }}
              onDoubleClick={(e) => {
                e.stopPropagation();
                e.preventDefault();
                onResetControlPoint?.(conn.id);
              }}
            >
              <title>双击重置曲线，拖拽调整连线弯曲</title>
            </circle>
          </g>
        );
      })}

      {activePortDrag && (
        <path
          d={
            activePortDrag.type === 'output' 
              ? calculateBezierPath(activePortDrag.startX, activePortDrag.startY, portDragCurrentPos.x, portDragCurrentPos.y)
              : calculateBezierPath(portDragCurrentPos.x, portDragCurrentPos.y, activePortDrag.startX, activePortDrag.startY)
          }
          fill="none"
          stroke={snappingTargetNodeId ? '#22d3ee' : '#3b82f6'}
          strokeWidth={snappingTargetNodeId ? 4 : 2}
          markerEnd={snappingTargetNodeId ? "url(#arrowhead-selected)" : "url(#arrowhead-dragging)"}
          className={cn(
            "flow-line-animated",
            snappingTargetNodeId ? "drop-shadow-[0_0_10px_rgba(34,211,238,0.8)]" : "opacity-80"
          )}
        />
      )}
    </svg>
  );
};

