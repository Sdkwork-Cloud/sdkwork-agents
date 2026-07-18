import React from 'react';
import { useCanvasLogic } from './hooks/useCanvasLogic';
import { 
  HelpCircle, 
  Sparkles, 
  Layers, 
  Trash2, 
  RotateCcw, 
  Check, 
  HelpCircle as InfoIcon,
  MousePointer2,
  Hand,
  FolderOpen,
  Plus,
  FolderPlus,
  X,
  Settings,
  Link2,
  FolderMinus,
  Maximize2,
  FolderDot,
  Grid,
  Camera,
  Library
} from 'lucide-react';
import { CanvasNode, Connection, CanvasTool, PanPosition, CanvasGroup } from './types';
import { Toolbar } from './components/Toolbar';
import { FlowCard } from './components/FlowCard';
import { CanvasGroupItem } from './components/CanvasGroupItem';
import { CanvasConnections } from './components/CanvasConnections';
import { CanvasAlignGuides } from './components/CanvasAlignGuides';
import { CanvasSelectionBox } from './components/CanvasSelectionBox';
import { CanvasMultiSelectionBounds } from './components/CanvasMultiSelectionBounds';
import { CanvasStickyConnectionHud } from './components/CanvasStickyConnectionHud';
import { CanvasSelectionToolbar } from './components/CanvasSelectionToolbar';
import { CanvasHelpPanel } from './components/CanvasHelpPanel';
import { CanvasSnapshotPanel } from './components/CanvasSnapshotPanel';
import { CanvasTemplatesPanel } from './components/CanvasTemplatesPanel';
import { cn } from '@sdkwork/agents-pc-commons';
import { CanvasService } from './services/CanvasService';
import { exportCanvasToPDF } from './utils/pdfExport';

import { CanvasMinimap } from './components/CanvasMinimap';
import { CanvasZoomControls } from './components/CanvasZoomControls';
import { CanvasContextMenu } from './components/CanvasContextMenu';
import { CanvasToastBanner } from './components/CanvasToastBanner';
import { CanvasTopActionBar } from './components/CanvasTopActionBar';
import { CanvasCreativeInputContainer } from './components/CanvasCreativeInputContainer';


export const CanvasView: React.FC = () => {
  const {
    containerRef,
    nodes,
    groups,
    connections,
    toast,
    contextMenu,
    pan,
    zoom,
    isAnimatingView,
    viewportSize,
    showMinimap,
    clipboard,
    activeTool,
    selectedNodeIds,
    selectedConnectionId,
    selectedGroupId,
    draggingNodeId,
    draggingConnectionId,
    snapToGrid,
    showGrid,
    selectionBox,
    activePortDrag,
    portDragCurrentPos,
    snappingTargetNodeId,
    isStickyConnection,
    showHelp,
    showSnapshots,
    showTemplates,
    spacePressed,
    alignGuides,
    history,
    redoStack,
    setNodes,
    setContextMenu,
    setShowMinimap,
    setActiveTool,
    setSelectedNodeIds,
    setSelectedConnectionId,
    setSelectedGroupId,
    setSnapToGrid,
    setShowGrid,
    setShowHelp,
    setShowSnapshots,
    setShowTemplates,
    handleUndo,
    handleRedo,
    handleContextMenuAction,
    handleZoomIn,
    handleZoomOut,
    handleResetZoom,
    handleWheel,
    handleMouseDown,
    handleMouseMove,
    handleMouseUp,
    handleNodeDragStart,
    handlePortMouseDown,
    handleGroupDragStart,
    handleGroupResizeStart,
    handleNodeResizeStart,
    handleUpdateNode,
    handleInputPromptChange,
    handleInputSettingsChange,
    handleInputModeChange,
    handleDeleteNode,
    handleSelectNode,
    handleAddNode,
    handleCreateGroupFromSelection,
    handleBatchDelete,
    handleDisbandGroup,
    handleAlignNodes,
    handleToggleGroupCollapse,
    handleUpdateGroupTitle,
    handleUpdateGroupColor,
    handleResetView,
    handleAutoLayout,
    handleExport,
    handleExportPNG,
    handleExportPDF,
    handleRestoreSnapshot,
    handleLoadTemplate,
    handleImportJSON,
    handleStartDragControlPoint,
    handleResetControlPoint,
    handleClearCanvas,
    saveHistory,
    triggerNodeGeneration,
    showToastMessage,
    clearToast,
    setSmoothView
  } = useCanvasLogic();

  const visibleNodes = nodes.filter(node => {
    if (!node.groupId) return true;
    const g = groups.find(grp => grp.id === node.groupId);
    return !g?.isCollapsed;
  });

  const visibleConnections = connections.filter(conn => {
    const fromNode = nodes.find(n => n.id === conn.fromNodeId);
    const toNode = nodes.find(n => n.id === conn.toNodeId);
    if (!fromNode || !toNode) return false;
    
    const fromGroup = fromNode.groupId ? groups.find(g => g.id === fromNode.groupId) : null;
    const toGroup = toNode.groupId ? groups.find(g => g.id === toNode.groupId) : null;
    
    return !fromGroup?.isCollapsed && !toGroup?.isCollapsed;
  });

  return (
    <div className="flex-1 h-full w-full bg-[#0d0d0e] text-zinc-200 relative overflow-hidden flex flex-col font-sans">
      
      {/* GLOBAL TOAST BANNER */}
      <CanvasToastBanner toast={toast} onClear={clearToast} />
      
      {/* Inline dynamic style animation for extreme link flow polished feel */}
      <style>{`
        @keyframes flow-dash {
          to {
            stroke-dashoffset: -28;
          }
        }
        .flow-line-animated {
          stroke-dasharray: 8, 4;
          animation: flow-dash 1.2s linear infinite;
        }
        .flow-line-animated-slow {
          stroke-dasharray: 8, 8;
          animation: flow-dash 2.5s linear infinite;
        }
      `}</style>

      {/* INFINITE VIEWPORT CANVAS */}
      <div 
        ref={containerRef}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onWheel={handleWheel}
        onDoubleClick={(e) => {
          if (e.target instanceof HTMLElement && (e.target.classList.contains('canvas-background') || e.target.classList.contains('grid-svg'))) {
            if (!containerRef.current) return;
            const rect = containerRef.current.getBoundingClientRect();
            const worldX = (e.clientX - rect.left - pan.x) / zoom;
            const worldY = (e.clientY - rect.top - pan.y) / zoom;

            const newNodeId = `node-${Date.now()}`;
            const newNode: CanvasNode = {
              id: newNodeId,
              type: 'text',
              x: Math.round(worldX - 160),
              y: Math.round(worldY - 125),
              width: 320,
              height: 250,
              title: `文本创意 #${nodes.length + 1}`,
              status: 'idle'
            };

            saveHistory();
            setNodes(prev => [...prev, newNode]);
            setSelectedNodeIds([newNodeId]);
          }
        }}
        onContextMenu={(e) => {
          e.preventDefault();
          if (e.target instanceof HTMLElement && (e.target.classList.contains('canvas-background') || e.target.classList.contains('grid-svg'))) {
            setSelectedNodeIds([]);
            setSelectedConnectionId(null);
            setSelectedGroupId(null);
            setContextMenu({ x: e.clientX, y: e.clientY, target: 'canvas' });
          }
        }}
        className={cn(
          "canvas-background flex-1 w-full h-full relative overflow-hidden select-none outline-none",
          activeTool === 'hand' || spacePressed ? "cursor-grab active:cursor-grabbing" : "cursor-default"
        )}
        style={{
          backgroundImage: showGrid ? `radial-gradient(circle at 1px 1px, rgba(255, 255, 255, 0.08) 1.5px, transparent 0)` : 'none',
          backgroundSize: `${32 * zoom}px ${32 * zoom}px`,
          backgroundPosition: `${pan.x}px ${pan.y}px`,
          transition: isAnimatingView ? 'background-position 0.4s cubic-bezier(0.16, 1, 0.3, 1), background-size 0.4s cubic-bezier(0.16, 1, 0.3, 1)' : 'none'
        }}
      >
        {/* GRAPH SCALE & PAN STAGE */}
        <div
          style={{
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
            transformOrigin: '0 0',
            transition: isAnimatingView ? 'transform 0.4s cubic-bezier(0.16, 1, 0.3, 1)' : 'none'
          }}
          className="absolute inset-0 pointer-events-none"
        >
          {/* GROUP FRAMES CONTAINER (RENDERED AT LOWER DEPTH Z-INDEX: 10) */}
          <div className="absolute inset-0">
            {groups.map(group => (
              <CanvasGroupItem 
                key={group.id}
                group={group}
                isSelected={selectedGroupId === group.id}
                onMouseDown={handleGroupDragStart}
                onResizeMouseDown={handleGroupResizeStart}
                onTitleChange={handleUpdateGroupTitle}
                onColorChange={handleUpdateGroupColor}
                onDisband={handleDisbandGroup}
                onToggleCollapse={handleToggleGroupCollapse}
                onContextMenu={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  setSelectedGroupId(group.id);
                  setContextMenu({ x: e.clientX, y: e.clientY, target: 'group' });
                }}
              />
            ))}
          </div>

          {/* SMART ALIGNMENT GUIDELINES (智能对齐辅助线) */}
          <CanvasAlignGuides guides={alignGuides} />

          {/* DYNAMIC SVG LINK CONNECTION PATHS overlay */}
          <CanvasConnections
            connections={visibleConnections}
            nodes={visibleNodes}
            activePortDrag={activePortDrag}
            portDragCurrentPos={portDragCurrentPos}
            snappingTargetNodeId={snappingTargetNodeId}
            selectedConnectionId={selectedConnectionId}
            onSelectConnection={(e, id) => {
              e.stopPropagation();
              setSelectedConnectionId(id);
              setSelectedNodeIds([]);
              setSelectedGroupId(null);
            }}
            onContextMenuConnection={(e, id) => {
              e.preventDefault();
              e.stopPropagation();
              setSelectedConnectionId(id);
              setSelectedNodeIds([]);
              setSelectedGroupId(null);
              setContextMenu({ x: e.clientX, y: e.clientY, target: 'connection' });
            }}
            onStartDragControlPoint={handleStartDragControlPoint}
            draggingConnectionId={draggingConnectionId}
            onResetControlPoint={handleResetControlPoint}
          />

          {/* DYNAMIC REAL-TIME SELECTION BOX (圈选) */}
          <CanvasSelectionBox selectionBox={selectionBox} />

          {/* DYNAMIC MULTI-SELECTION BOUNDS HIGHLIGHT */}
          <CanvasMultiSelectionBounds selectedNodeIds={selectedNodeIds} nodes={visibleNodes} />

          {/* INTERACTIVE CARDS LAYERS (Z-INDEX: 20) */}
          <div className="absolute inset-0">
            {visibleNodes.map(node => {
              const incomingConn = visibleConnections.find(c => c.toNodeId === node.id);
              const incomingNode = incomingConn ? visibleNodes.find(n => n.id === incomingConn.fromNodeId) : undefined;
              
              const outgoingConns = visibleConnections.filter(c => c.fromNodeId === node.id);
              const outgoingNodes = outgoingConns.map(c => visibleNodes.find(n => n.id === c.toNodeId)).filter(Boolean) as CanvasNode[];

              return (
                <FlowCard
                  key={node.id}
                  node={node}
                  isSelected={selectedNodeIds.includes(node.id)}
                  onSelect={handleSelectNode}
                  onUpdate={handleUpdateNode}
                  onDelete={handleDeleteNode}
                  onDragStart={handleNodeDragStart}
                  onPortMouseDown={handlePortMouseDown}
                  connectedInputNode={incomingNode}
                  connectedOutputNodes={outgoingNodes}
                  isSnappingTarget={snappingTargetNodeId === node.id}
                  isDragging={draggingNodeId === node.id || (selectedNodeIds.includes(node.id) && draggingNodeId !== null)}
                  onResizeStart={(e, dir) => handleNodeResizeStart(node.id, e, dir)}
                  onContextMenu={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    if (!selectedNodeIds.includes(node.id)) {
                      setSelectedNodeIds([node.id]);
                    }
                    setContextMenu({ x: e.clientX, y: e.clientY, target: 'node' });
                  }}
                />
              );
            })}
          </div>
        </div>
      </div>

      {/* STICKY CLICK-TO-CONNECT HUD (点击连线模式HUD) */}
      <div className="no-export">
        <CanvasStickyConnectionHud isVisible={isStickyConnection && activePortDrag !== null} />
      </div>

      {/* FLOATING ACTION TOOLBAR ON MULTIPLE SELECTIONS (批量操作浮标) */}
      <div className="no-export">
        <CanvasSelectionToolbar 
          selectedCount={selectedNodeIds.length} 
          onCreateGroup={handleCreateGroupFromSelection} 
          onBatchDelete={handleBatchDelete} 
          onClearSelection={() => setSelectedNodeIds([])} 
          onAlignNodes={handleAlignNodes}
        />
      </div>

      {/* FLOAT SIDEBAR USAGE GUIDE */}
      <div className="no-export">
        <CanvasHelpPanel showHelp={showHelp} setShowHelp={setShowHelp} />
      </div>

      <div className="no-export">
        <CanvasSnapshotPanel
          showSnapshots={showSnapshots}
          setShowSnapshots={(val) => {
            setShowSnapshots(val);
            if (val) setShowTemplates(false);
          }}
          nodes={nodes}
          groups={groups}
          connections={connections}
          pan={pan}
          zoom={zoom}
          onRestoreSnapshot={handleRestoreSnapshot}
          showToast={showToastMessage}
        />
      </div>

      <div className="no-export">
        <CanvasTemplatesPanel
          showTemplates={showTemplates}
          setShowTemplates={(val) => {
            setShowTemplates(val);
            if (val) setShowSnapshots(false);
          }}
          nodes={nodes}
          groups={groups}
          connections={connections}
          onLoadTemplate={handleLoadTemplate}
          showToast={showToastMessage}
        />
      </div>

      {/* FLOAT TOP ACTION BAR */}
      <CanvasTopActionBar
        snapToGrid={snapToGrid}
        onToggleSnapToGrid={() => setSnapToGrid(!snapToGrid)}
        showSnapshots={showSnapshots}
        onToggleSnapshots={setShowSnapshots}
        showTemplates={showTemplates}
        onToggleTemplates={setShowTemplates}
        showHelp={showHelp}
        onShowHelp={() => setShowHelp(true)}
      />

      {/* TOOLBAR CONTROLLER */}
      <div className="no-export">
        <Toolbar
          activeTool={activeTool}
          setActiveTool={setActiveTool}
          zoom={zoom}
          onZoomIn={handleZoomIn}
          onZoomOut={handleZoomOut}
          onResetView={handleResetView}
          onAutoLayout={handleAutoLayout}
          onClearCanvas={handleClearCanvas}
          onAddNode={handleAddNode}
          setShowHelp={setShowHelp}
          showHelp={showHelp}
          onUndo={handleUndo}
          onRedo={handleRedo}
          canUndo={history.length > 0}
          canRedo={redoStack.length > 0}
          onExportJSON={handleExport}
          onExportPNG={handleExportPNG}
          onExportPDF={handleExportPDF}
          onImportJSON={handleImportJSON}
          showGrid={showGrid}
          onToggleGrid={() => setShowGrid(!showGrid)}
        />
      </div>

      {/* CONTEXT MENU */}
      {contextMenu && (
        <div className="no-export">
          <CanvasContextMenu
            x={contextMenu.x}
            y={contextMenu.y}
            target={contextMenu.target}
            onClose={() => setContextMenu(null)}
            onAction={handleContextMenuAction}
            hasClipboardContent={clipboard.nodes.length > 0}
            hasMultipleSelection={selectedNodeIds.length > 1}
            isInGroup={selectedNodeIds.length === 1 && !!nodes.find(n => n.id === selectedNodeIds[0])?.groupId}
          />
        </div>
      )}

      {/* BOTTOM LEFT MINIMAP */}
      {showMinimap && (
        <div className="absolute bottom-6 left-6 z-40 no-export">
          <CanvasMinimap
            nodes={visibleNodes}
            groups={groups}
            connections={visibleConnections}
            pan={pan}
            zoom={zoom}
            viewportWidth={viewportSize.width}
            viewportHeight={viewportSize.height}
            onPanChange={(x, y) => setSmoothView({ x, y }, zoom)}
          />
        </div>
      )}

      {/* BOTTOM RIGHT CONTROLS */}
      <div className="absolute bottom-6 right-6 z-40 flex flex-col items-end gap-2 no-export">
        <CanvasZoomControls
          zoom={zoom}
          onZoomIn={handleZoomIn}
          onZoomOut={handleZoomOut}
          onResetZoom={handleResetZoom}
          onZoomToFit={handleResetView}
          showMinimap={showMinimap}
          onToggleMinimap={() => setShowMinimap(!showMinimap)}
        />
      </div>

      {/* PERSISTENT GLOBAL BOTTOM CREATIVE INPUT BOX */}
      <CanvasCreativeInputContainer
        selectedNodeIds={selectedNodeIds}
        nodes={nodes}
        onClearSelection={() => setSelectedNodeIds([])}
        triggerNodeGeneration={triggerNodeGeneration}
        containerRef={containerRef}
        pan={pan}
        zoom={zoom}
        saveHistory={saveHistory}
        setNodes={setNodes}
        setSelectedNodeIds={setSelectedNodeIds}
        setSmoothView={setSmoothView}
        handleInputPromptChange={handleInputPromptChange}
        handleInputModeChange={handleInputModeChange}
        handleInputSettingsChange={handleInputSettingsChange}
      />
    </div>
  );
};
