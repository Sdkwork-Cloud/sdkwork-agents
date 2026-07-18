import React from 'react';
import { createPortal } from 'react-dom';
import { Share, Edit3, Settings, Folder, Pin, Trash2 } from 'lucide-react';

interface SidebarProjectContextMenuProps {
  project: string;
  dropdownPos: { top: number, left: number };
  onClose: () => void;
  onProjectSettings?: (project: string) => void;
  onProjectSelect?: (project: string) => void;
  onProjectRename?: () => void;
  onProjectDelete?: () => void;
}

export const SidebarProjectContextMenu: React.FC<SidebarProjectContextMenuProps> = ({
  project,
  dropdownPos,
  onClose,
  onProjectSettings,
  onProjectSelect,
  onProjectRename,
  onProjectDelete
}) => {
    return createPortal(
      <div 
        className="fixed mt-1 w-[180px] bg-[#2A2A2D] border border-white/10 rounded-xl shadow-2xl py-1.5 z-[9999] animate-in fade-in zoom-in-95 duration-100" 
        style={{ top: dropdownPos.top, left: dropdownPos.left }}
        onClick={e => e.stopPropagation()}
      >
        <button className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left">
          <Share size={14} className="text-zinc-400" />
          分享项目
        </button>
        <button 
          onClick={(e) => {
            onClose();
            onProjectRename?.();
          }}
          className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left"
        >
          <Edit3 size={14} className="text-zinc-400" />
          重命名项目
        </button>
        <button 
          onClick={(e) => {
            onClose();
            onProjectSettings?.(project);
          }}
          className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left"
        >
          <Settings size={14} className="text-zinc-400" />
          项目设置
        </button>
        <button 
          onClick={(e) => {
            onClose();
            onProjectSelect?.(project);
          }}
          className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left border-b border-white/5 pb-2.5 mb-1.5"
        >
          <Folder size={14} className="text-zinc-400" />
          项目主页
        </button>
        
        <button className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left">
          <Pin size={14} className="text-zinc-400" />
          置顶项目
        </button>
        <button 
          onClick={(e) => {
            onClose();
            onProjectDelete?.();
          }}
          className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-red-400 hover:text-red-300 hover:bg-red-400/10 transition-colors text-left mt-0.5"
        >
          <Trash2 size={14} />
          删除项目
        </button>
      </div>,
      document.body
    );
  };
