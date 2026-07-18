import React, { useState, useEffect } from 'react';
import { 
  Camera, 
  Trash2, 
  Plus, 
  Bookmark, 
  Play, 
  Edit2, 
  Check, 
  X, 
  Download, 
  Upload, 
  History,
  FileJson
} from 'lucide-react';
import { CanvasNode, CanvasGroup, Connection } from '../types';

interface Snapshot {
  id: string;
  name: string;
  createdAt: string;
  nodes: CanvasNode[];
  groups: CanvasGroup[];
  connections: Connection[];
  pan: { x: number; y: number };
  zoom: number;
}

interface CanvasSnapshotPanelProps {
  showSnapshots: boolean;
  setShowSnapshots: (show: boolean) => void;
  nodes: CanvasNode[];
  groups: CanvasGroup[];
  connections: Connection[];
  pan: { x: number; y: number };
  zoom: number;
  onRestoreSnapshot: (snapshot: {
    nodes: CanvasNode[];
    groups: CanvasGroup[];
    connections: Connection[];
    pan: { x: number; y: number };
    zoom: number;
  }) => void;
  showToast: (message: string, type: 'success' | 'error' | 'info' | 'loading') => void;
}

const LOCAL_STORAGE_KEY = 'sdkwork_agents_canvas_snapshots_v1';

export const CanvasSnapshotPanel: React.FC<CanvasSnapshotPanelProps> = ({
  showSnapshots,
  setShowSnapshots,
  nodes,
  groups,
  connections,
  pan,
  zoom,
  onRestoreSnapshot,
  showToast
}) => {
  const [snapshots, setSnapshots] = useState<Snapshot[]>([]);
  const [newSnapshotName, setNewSnapshotName] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState('');

  // Load snapshots from localStorage on mount
  useEffect(() => {
    try {
      const saved = localStorage.getItem(LOCAL_STORAGE_KEY);
      if (saved) {
        setSnapshots(JSON.parse(saved));
      }
    } catch (e) {
      console.error('Failed to load snapshots:', e);
    }
  }, []);

  // Save snapshots to localStorage helper
  const saveToStorage = (updatedList: Snapshot[]) => {
    try {
      localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(updatedList));
      setSnapshots(updatedList);
    } catch (e) {
      console.error('Failed to save snapshots:', e);
      showToast('快照存储失败，空间可能已满', 'error');
    }
  };

  // Create Snapshot
  const handleCreateSnapshot = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    
    const trimmedName = newSnapshotName.trim();
    const finalName = trimmedName || `快照 #${snapshots.length + 1} (${new Date().toLocaleTimeString()})`;

    const newSnapshot: Snapshot = {
      id: `snapshot-${Date.now()}`,
      name: finalName,
      createdAt: new Date().toISOString(),
      nodes: JSON.parse(JSON.stringify(nodes)),
      groups: JSON.parse(JSON.stringify(groups)),
      connections: JSON.parse(JSON.stringify(connections)),
      pan: { ...pan },
      zoom
    };

    const updated = [newSnapshot, ...snapshots];
    saveToStorage(updated);
    setNewSnapshotName('');
    showToast(`快照 "${finalName}" 保存成功`, 'success');
  };

  // Delete Snapshot
  const handleDeleteSnapshot = (id: string, name: string) => {
    const updated = snapshots.filter(s => s.id !== id);
    saveToStorage(updated);
    showToast(`快照 "${name}" 已删除`, 'success');
  };

  // Start Editing Name
  const startEditing = (id: string, currentName: string) => {
    setEditingId(id);
    setEditingName(currentName);
  };

  // Save Edited Name
  const saveNameEdit = (id: string) => {
    const trimmed = editingName.trim();
    if (!trimmed) return;
    const updated = snapshots.map(s => s.id === id ? { ...s, name: trimmed } : s);
    saveToStorage(updated);
    setEditingId(null);
    showToast('快照重命名成功', 'success');
  };

  // Restore Snapshot
  const handleRestore = (snapshot: Snapshot) => {
    onRestoreSnapshot({
      nodes: JSON.parse(JSON.stringify(snapshot.nodes)),
      groups: JSON.parse(JSON.stringify(snapshot.groups)),
      connections: JSON.parse(JSON.stringify(snapshot.connections)),
      pan: { ...snapshot.pan },
      zoom: snapshot.zoom
    });
    showToast(`已成功载入快照 "${snapshot.name}"`, 'success');
  };

  // Export Snapshots as JSON File
  const handleExportSnapshots = () => {
    if (snapshots.length === 0) {
      showToast('暂无快照可供导出', 'error');
      return;
    }
    try {
      const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(snapshots, null, 2));
      const downloadAnchor = document.createElement('a');
      downloadAnchor.setAttribute("href", dataStr);
      downloadAnchor.setAttribute("download", `canvas-snapshots-${new Date().toISOString().slice(0, 10)}.json`);
      document.body.appendChild(downloadAnchor);
      downloadAnchor.click();
      downloadAnchor.remove();
      showToast('快照配置成功导出', 'success');
    } catch (e) {
      showToast('导出快照失败', 'error');
    }
  };

  // Import Snapshots from JSON File
  const handleImportSnapshots = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const parsed = JSON.parse(event.target?.result as string);
        if (Array.isArray(parsed) && parsed.every(item => item.id && item.name && Array.isArray(item.nodes))) {
          // Merge imported snapshots with current, avoiding duplicate IDs
          const existingIds = new Set(snapshots.map(s => s.id));
          const filteredImports = parsed.map(s => {
            if (existingIds.has(s.id)) {
              return { ...s, id: `${s.id}-imported-${Date.now()}` };
            }
            return s;
          });

          const merged = [...filteredImports, ...snapshots];
          saveToStorage(merged);
          showToast(`成功导入 ${filteredImports.length} 个快照`, 'success');
        } else {
          showToast('非法的快照配置文件格式', 'error');
        }
      } catch (err) {
        showToast('读取文件失败，请确保是正确的 JSON 文件', 'error');
      }
    };
    reader.readAsText(file);
    e.target.value = ''; // Reset input
  };

  if (!showSnapshots) return null;

  return (
    <div className="absolute top-[84px] right-6 w-80 bg-[#141416]/95 border border-white/10 backdrop-blur-md p-5 rounded-2xl shadow-2xl z-40 select-none flex flex-col max-h-[calc(100vh-120px)] animate-in slide-in-from-right duration-200">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-white/5 pb-3 mb-4 shrink-0">
        <div className="flex items-center gap-2">
          <span className="p-1 rounded bg-amber-500/10 text-amber-400">
            <Camera size={14} />
          </span>
          <span className="text-[13px] font-extrabold text-zinc-100 tracking-wide">快照书签管理</span>
        </div>
        <button 
          onClick={() => setShowSnapshots(false)}
          className="text-xs text-zinc-500 hover:text-zinc-300 bg-white/5 hover:bg-white/10 px-2 py-1 rounded cursor-pointer transition-colors"
        >
          关闭
        </button>
      </div>

      {/* Description */}
      <p className="text-zinc-500 text-[11px] mb-4 leading-relaxed shrink-0">
        记录当前画布的所有卡片位置、连接流向和分组信息，随时一键回滚/恢复，适合用于备份多种创意备选方案。
      </p>

      {/* Create New Snapshot Form */}
      <form onSubmit={handleCreateSnapshot} className="flex gap-2 mb-4 shrink-0">
        <input
          type="text"
          placeholder="给当前进度起个名字..."
          value={newSnapshotName}
          onChange={(e) => setNewSnapshotName(e.target.value)}
          className="flex-1 bg-black/40 border border-white/10 rounded-xl px-3 py-2 text-xs text-white placeholder-zinc-600 focus:outline-none focus:border-cyan-500/50 transition-colors"
        />
        <button
          type="submit"
          className="bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-white p-2 rounded-xl flex items-center justify-center transition-all cursor-pointer shadow-[0_4px_12px_rgba(6,182,212,0.15)] shrink-0"
          title="保存当前快照"
        >
          <Plus size={16} />
        </button>
      </form>

      {/* Action utilities (Export/Import) */}
      <div className="flex gap-2 mb-4 text-[10px] shrink-0 border-b border-white/5 pb-3">
        <button
          onClick={handleExportSnapshots}
          className="flex-1 flex items-center justify-center gap-1 py-1.5 rounded-lg border border-white/5 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10 cursor-pointer transition-colors"
        >
          <Download size={11} />
          <span>导出备份</span>
        </button>
        <label
          className="flex-1 flex items-center justify-center gap-1 py-1.5 rounded-lg border border-white/5 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10 cursor-pointer transition-colors text-center"
        >
          <Upload size={11} />
          <span>导入备份</span>
          <input
            type="file"
            accept=".json"
            onChange={handleImportSnapshots}
            className="hidden"
          />
        </label>
      </div>

      {/* Snapshots List */}
      <div className="flex-1 overflow-y-auto pr-1 space-y-2.5 scrollbar-thin max-h-[320px]">
        {snapshots.length === 0 ? (
          <div className="py-8 text-center flex flex-col items-center justify-center gap-2">
            <Bookmark size={24} className="text-zinc-700 animate-pulse" />
            <span className="text-[11px] text-zinc-600">暂无任何快照，在上方创建一个吧</span>
          </div>
        ) : (
          snapshots.map((snap) => {
            const hasGroups = snap.groups && snap.groups.length > 0;
            const hasConnections = snap.connections && snap.connections.length > 0;
            
            // Count type frequencies
            const counts = { text: 0, image: 0, video: 0, sticky: 0 };
            snap.nodes.forEach(n => {
              if (n.type === 'text') counts.text++;
              else if (n.type === 'image-gen') counts.image++;
              else if (n.type === 'video-gen') counts.video++;
              else if (n.type === 'sticky') counts.sticky++;
            });

            return (
              <div 
                key={snap.id} 
                className="group p-3 bg-white/5 hover:bg-white/[0.08] border border-white/5 hover:border-white/10 rounded-xl transition-all duration-200"
              >
                {/* Name / Rename Section */}
                <div className="flex items-start justify-between gap-2 mb-1.5">
                  {editingId === snap.id ? (
                    <div className="flex items-center gap-1.5 flex-1 min-w-0">
                      <input
                        type="text"
                        value={editingName}
                        onChange={(e) => setEditingName(e.target.value)}
                        className="flex-1 bg-black/60 border border-cyan-500/40 rounded px-1.5 py-0.5 text-xs text-white focus:outline-none focus:border-cyan-500"
                        autoFocus
                      />
                      <button 
                        onClick={() => saveNameEdit(snap.id)}
                        className="text-emerald-400 hover:text-emerald-300 p-0.5"
                      >
                        <Check size={12} />
                      </button>
                      <button 
                        onClick={() => setEditingId(null)}
                        className="text-zinc-500 hover:text-zinc-400 p-0.5"
                      >
                        <X size={12} />
                      </button>
                    </div>
                  ) : (
                    <div className="flex-1 min-w-0">
                      <h4 className="text-xs font-bold text-zinc-200 truncate group-hover:text-cyan-400 transition-colors">
                        {snap.name}
                      </h4>
                      <span className="text-[9px] text-zinc-500 block mt-0.5">
                        {new Date(snap.createdAt).toLocaleString()}
                      </span>
                    </div>
                  )}

                  {editingId !== snap.id && (
                    <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                      <button
                        onClick={() => startEditing(snap.id, snap.name)}
                        className="text-zinc-500 hover:text-zinc-300 p-1 rounded hover:bg-white/5"
                        title="重命名快照"
                      >
                        <Edit2 size={11} />
                      </button>
                      <button
                        onClick={() => handleDeleteSnapshot(snap.id, snap.name)}
                        className="text-zinc-500 hover:text-rose-400 p-1 rounded hover:bg-white/5"
                        title="删除快照"
                      >
                        <Trash2 size={11} />
                      </button>
                    </div>
                  )}
                </div>

                {/* Info / Metadata Badges */}
                <div className="flex flex-wrap gap-1 mb-2.5">
                  <span className="px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 text-[9px] font-medium">
                    卡片: {snap.nodes.length}
                  </span>
                  {hasConnections && (
                    <span className="px-1.5 py-0.5 rounded bg-cyan-950/40 text-cyan-400 text-[9px] font-medium">
                      连线: {snap.connections.length}
                    </span>
                  )}
                  {hasGroups && (
                    <span className="px-1.5 py-0.5 rounded bg-amber-950/40 text-amber-400 text-[9px] font-medium">
                      分组: {snap.groups.length}
                    </span>
                  )}
                </div>

                {/* Nodes breakdown row */}
                <div className="flex items-center gap-2 border-t border-white/5 pt-2 mt-2 shrink-0">
                  <div className="flex-1 flex gap-2 text-[9px] text-zinc-500">
                    {counts.text > 0 && <span>文本:{counts.text}</span>}
                    {counts.image > 0 && <span className="text-teal-500">图片:{counts.image}</span>}
                    {counts.video > 0 && <span className="text-purple-400">视频:{counts.video}</span>}
                    {counts.sticky > 0 && <span className="text-yellow-500">便签:{counts.sticky}</span>}
                  </div>
                  
                  {/* Restore CTA */}
                  <button
                    onClick={() => handleRestore(snap)}
                    className="px-2 py-1 rounded bg-cyan-500/10 hover:bg-cyan-500 hover:text-black text-cyan-400 flex items-center gap-1 text-[10px] font-bold cursor-pointer transition-all"
                  >
                    <Play size={10} className="fill-current shrink-0" />
                    <span>恢复此进度</span>
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
