import React, { useState, useEffect } from 'react';
import { 
  Library, 
  Trash2, 
  Plus, 
  Workflow, 
  GitFork, 
  Share2, 
  Sparkles,
  Layers,
  HelpCircle,
  Play,
  FileCode,
  Download,
  Upload
} from 'lucide-react';
import { CanvasNode, CanvasGroup, Connection } from '../types';

import { Template, SYSTEM_TEMPLATES } from '../data/templates';

interface CanvasTemplatesPanelProps {
  showTemplates: boolean;
  setShowTemplates: (show: boolean) => void;
  nodes: CanvasNode[];
  groups: CanvasGroup[];
  connections: Connection[];
  onLoadTemplate: (template: {
    nodes: CanvasNode[];
    groups: CanvasGroup[];
    connections: Connection[];
  }, mode: 'append' | 'replace') => void;
  showToast: (message: string, type: 'success' | 'error' | 'info' | 'loading') => void;
}

const LOCAL_STORAGE_KEY = 'chatbox_canvas_templates_v1';

export const CanvasTemplatesPanel: React.FC<CanvasTemplatesPanelProps> = ({
  showTemplates,
  setShowTemplates,
  nodes,
  groups,
  connections,
  onLoadTemplate,
  showToast
}) => {
  const [customTemplates, setCustomTemplates] = useState<Template[]>([]);
  const [activeTab, setActiveTab] = useState<'system' | 'custom'>('system');
  const [newTemplateName, setNewTemplateName] = useState('');
  const [newTemplateDesc, setNewTemplateDesc] = useState('');

  // Load custom templates on mount
  useEffect(() => {
    try {
      const saved = localStorage.getItem(LOCAL_STORAGE_KEY);
      if (saved) {
        setCustomTemplates(JSON.parse(saved));
      }
    } catch (e) {
      console.error('Failed to load custom templates:', e);
    }
  }, []);

  const saveToStorage = (updated: Template[]) => {
    try {
      localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(updated));
      setCustomTemplates(updated);
    } catch (e) {
      showToast('模板保存失败，可能存储空间不足', 'error');
    }
  };

  // Save current canvas layout as template
  const handleSaveAsTemplate = (e: React.FormEvent) => {
    e.preventDefault();
    if (nodes.length === 0) {
      showToast('当前画布无内容，无法保存为模板', 'error');
      return;
    }

    const trimmedName = newTemplateName.trim();
    if (!trimmedName) {
      showToast('请输入模板名称', 'error');
      return;
    }

    // Capture current offset so we can normalize the node positions
    // This allows newly loaded templates to center gracefully
    let minX = Infinity;
    let minY = Infinity;
    nodes.forEach(n => {
      if (n.x < minX) minX = n.x;
      if (n.y < minY) minY = n.y;
    });

    const normalizedNodes = nodes.map(n => ({
      ...n,
      x: n.x - minX,
      y: n.y - minY
    }));

    const normalizedGroups = groups.map(g => ({
      ...g,
      x: g.x - minX,
      y: g.y - minY
    }));

    const newTemplate: Template = {
      id: `tmpl-${Date.now()}`,
      name: trimmedName,
      description: newTemplateDesc.trim() || '用户自定义画布预设模板',
      category: 'custom',
      nodes: normalizedNodes,
      groups: normalizedGroups,
      connections: JSON.parse(JSON.stringify(connections))
    };

    const updated = [newTemplate, ...customTemplates];
    saveToStorage(updated);
    setNewTemplateName('');
    setNewTemplateDesc('');
    showToast(`模板 "${trimmedName}" 保存成功`, 'success');
  };

  // Delete Custom Template
  const handleDeleteTemplate = (id: string, name: string) => {
    const updated = customTemplates.filter(t => t.id !== id);
    saveToStorage(updated);
    showToast(`已删除模板 "${name}"`, 'success');
  };

  // Apply template
  const handleApplyTemplate = (tmpl: Template, mode: 'append' | 'replace') => {
    onLoadTemplate({
      nodes: tmpl.nodes,
      groups: tmpl.groups,
      connections: tmpl.connections
    }, mode);
    
    setShowTemplates(false);
  };

  if (!showTemplates) return null;

  return (
    <div className="absolute top-[84px] right-6 w-88 bg-[#141416]/95 border border-white/10 backdrop-blur-md p-5 rounded-2xl shadow-2xl z-40 select-none flex flex-col max-h-[calc(100vh-120px)] animate-in slide-in-from-right duration-200">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-white/5 pb-3 mb-4 shrink-0">
        <div className="flex items-center gap-2">
          <span className="p-1 rounded bg-cyan-500/10 text-cyan-400">
            <Library size={14} />
          </span>
          <span className="text-[13px] font-extrabold text-zinc-100 tracking-wide">高级模板库 (Templates)</span>
        </div>
        <button 
          onClick={() => setShowTemplates(false)}
          className="text-xs text-zinc-500 hover:text-zinc-300 bg-white/5 hover:bg-white/10 px-2 py-1 rounded cursor-pointer transition-colors"
        >
          关闭
        </button>
      </div>

      {/* Tabs */}
      <div className="flex bg-black/40 p-1 rounded-xl mb-4 shrink-0 border border-white/5">
        <button
          onClick={() => setActiveTab('system')}
          className={`flex-1 py-1.5 text-center text-xs font-bold rounded-lg cursor-pointer transition-all ${
            activeTab === 'system'
              ? 'bg-[#1e1e20] text-cyan-400 shadow-sm'
              : 'text-zinc-500 hover:text-zinc-300'
          }`}
        >
          系统预设工作流
        </button>
        <button
          onClick={() => setActiveTab('custom')}
          className={`flex-1 py-1.5 text-center text-xs font-bold rounded-lg cursor-pointer transition-all ${
            activeTab === 'custom'
              ? 'bg-[#1e1e20] text-cyan-400 shadow-sm'
              : 'text-zinc-500 hover:text-zinc-300'
          }`}
        >
          自定义模板
        </button>
      </div>

      {/* Tab Contents */}
      <div className="flex-1 overflow-y-auto pr-1 space-y-3.5 scrollbar-thin max-h-[350px]">
        {activeTab === 'system' ? (
          SYSTEM_TEMPLATES.map(tmpl => (
            <div 
              key={tmpl.id}
              className="p-3.5 bg-white/5 hover:bg-white/[0.08] border border-white/5 hover:border-white/10 rounded-xl transition-all duration-200"
            >
              <div className="flex items-start justify-between gap-2 mb-1">
                <span className={`text-[9px] px-1.5 py-0.5 rounded font-extrabold tracking-wider ${
                  tmpl.category === 'workflow' 
                    ? 'bg-cyan-500/10 text-cyan-400' 
                    : tmpl.category === 'collaboration'
                    ? 'bg-purple-500/10 text-purple-400'
                    : 'bg-amber-500/10 text-amber-400'
                }`}>
                  {tmpl.category === 'workflow' ? 'AI 创作流' : tmpl.category === 'collaboration' ? '多代理并联' : '脑暴图谱'}
                </span>
                <span className="text-[9px] text-zinc-500 font-medium">包含: {tmpl.nodes.length}卡片 / {tmpl.connections.length}流向</span>
              </div>

              <h4 className="text-xs font-bold text-zinc-200 mb-1">{tmpl.name}</h4>
              <p className="text-[10px] text-zinc-500 leading-relaxed mb-3">{tmpl.description}</p>

              {/* Action Buttons */}
              <div className="flex gap-2 border-t border-white/5 pt-2.5">
                <button
                  onClick={() => handleApplyTemplate(tmpl, 'append')}
                  className="flex-1 py-1.5 bg-white/5 hover:bg-white/10 text-zinc-300 hover:text-white rounded-lg text-[10px] font-bold cursor-pointer transition-colors flex items-center justify-center gap-1"
                >
                  <Plus size={10} />
                  <span>追加到画布</span>
                </button>
                <button
                  onClick={() => handleApplyTemplate(tmpl, 'replace')}
                  className="flex-1 py-1.5 bg-cyan-500/10 hover:bg-cyan-500 hover:text-black text-cyan-400 rounded-lg text-[10px] font-bold cursor-pointer transition-colors flex items-center justify-center gap-1"
                >
                  <Play size={10} className="fill-current shrink-0" />
                  <span>覆盖载入</span>
                </button>
              </div>
            </div>
          ))
        ) : (
          /* Custom User Templates */
          <div className="space-y-3.5">
            {/* Create form inside Custom Tab */}
            <form onSubmit={handleSaveAsTemplate} className="p-3 bg-zinc-900/50 border border-white/5 rounded-xl shrink-0 space-y-2.5">
              <h5 className="text-[10px] font-bold text-zinc-400 uppercase tracking-wide">保存当前画布为新模板</h5>
              <input
                type="text"
                placeholder="模板名称 (如: 我的写稿流程)..."
                value={newTemplateName}
                onChange={(e) => setNewTemplateName(e.target.value)}
                className="w-full bg-black/40 border border-white/10 rounded-lg px-2.5 py-1.5 text-xs text-white placeholder-zinc-600 focus:outline-none focus:border-cyan-500"
              />
              <input
                type="text"
                placeholder="模板简介 (可选)..."
                value={newTemplateDesc}
                onChange={(e) => setNewTemplateDesc(e.target.value)}
                className="w-full bg-black/40 border border-white/10 rounded-lg px-2.5 py-1.5 text-xs text-white placeholder-zinc-600 focus:outline-none focus:border-cyan-500"
              />
              <button
                type="submit"
                className="w-full bg-gradient-to-r from-cyan-500 to-blue-600 hover:from-cyan-400 hover:to-blue-500 text-white py-1.5 rounded-lg text-xs font-bold transition-all cursor-pointer shadow-lg flex items-center justify-center gap-1.5"
              >
                <Plus size={12} />
                <span>立即保存预设</span>
              </button>
            </form>

            {customTemplates.length === 0 ? (
              <div className="py-8 text-center flex flex-col items-center justify-center gap-2">
                <Workflow size={24} className="text-zinc-700 animate-pulse" />
                <span className="text-[10px] text-zinc-600">暂无自定义模板，可在上方进行保存</span>
              </div>
            ) : (
              customTemplates.map(tmpl => (
                <div 
                  key={tmpl.id}
                  className="p-3.5 bg-white/5 hover:bg-white/[0.08] border border-white/5 hover:border-white/10 rounded-xl transition-all duration-200 group"
                >
                  <div className="flex items-start justify-between gap-2 mb-1">
                    <span className="text-[9px] text-zinc-500 font-medium">配置: {tmpl.nodes.length}卡片 / {tmpl.connections.length}流向</span>
                    <button
                      type="button"
                      onClick={() => handleDeleteTemplate(tmpl.id, tmpl.name)}
                      className="text-zinc-500 hover:text-rose-400 p-0.5 rounded opacity-0 group-hover:opacity-100 transition-opacity"
                      title="删除此模板"
                    >
                      <Trash2 size={11} />
                    </button>
                  </div>

                  <h4 className="text-xs font-bold text-zinc-200 mb-1">{tmpl.name}</h4>
                  <p className="text-[10px] text-zinc-500 leading-relaxed mb-3">{tmpl.description}</p>

                  <div className="flex gap-2 border-t border-white/5 pt-2.5">
                    <button
                      onClick={() => handleApplyTemplate(tmpl, 'append')}
                      className="flex-1 py-1.5 bg-white/5 hover:bg-white/10 text-zinc-300 hover:text-white rounded-lg text-[10px] font-bold cursor-pointer transition-colors flex items-center justify-center gap-1"
                    >
                      <Plus size={10} />
                      <span>追加载入</span>
                    </button>
                    <button
                      onClick={() => handleApplyTemplate(tmpl, 'replace')}
                      className="flex-1 py-1.5 bg-cyan-500/10 hover:bg-cyan-500 hover:text-black text-cyan-400 rounded-lg text-[10px] font-bold cursor-pointer transition-colors flex items-center justify-center gap-1"
                    >
                      <Play size={10} className="fill-current shrink-0" />
                      <span>覆盖载入</span>
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
};
