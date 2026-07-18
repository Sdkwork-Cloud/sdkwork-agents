import React from "react";
import { createPortal } from "react-dom";
import {
  Share,
  UserPlus,
  Edit3,
  FolderInput,
  ChevronRight,
  FolderPlus,
  Pin,
  Archive,
  Trash2,
  Folder,
} from "lucide-react";
import { ChatSession } from "@/packages/sdkwork-chatbox-pc-core/src/sdk/types";

interface SidebarSessionContextMenuProps {
  session: ChatSession;
  isPinned: boolean;
  dropdownPos: { top: number; left: number };
  onClose: () => void;
  onTogglePin: (e: React.MouseEvent, id: string) => void;
  onDeleteSession: (e: React.MouseEvent, id: string) => void;
  onRename?: () => void;
  projectsList: string[];
  onMoveToProject: (project: string) => void;
  canDelete: boolean;
}

export const SidebarSessionContextMenu: React.FC<
  SidebarSessionContextMenuProps
> = ({
  session,
  isPinned,
  dropdownPos,
  onClose,
  onTogglePin,
  onDeleteSession,
  onRename,
  projectsList,
  onMoveToProject,
  canDelete,
}) => {
  return createPortal(
    <div
      className="fixed mt-1 w-[180px] bg-[#2A2A2D] border border-white/10 rounded-xl shadow-2xl py-1.5 z-[9999] animate-in fade-in zoom-in-95 duration-100"
      style={{ top: dropdownPos.top, left: dropdownPos.left }}
      onClick={(e) => e.stopPropagation()}
    >
      <button className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left">
        <Share size={14} className="text-zinc-400" />
        分享
      </button>
      <button className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left">
        <UserPlus size={14} className="text-zinc-400" />
        开始群聊
      </button>
      <button 
        onClick={(e) => {
          e.stopPropagation();
          onRename?.();
        }}
        className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left"
      >
        <Edit3 size={14} className="text-zinc-400" />
        重命名
      </button>

      <div className="relative group/project">
        <button className="w-full flex items-center justify-between px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left">
          <div className="flex items-center gap-3">
            <FolderInput size={14} className="text-zinc-400" />
            移至项目
          </div>
          <ChevronRight
            size={14}
            className="text-zinc-500 group-hover/project:text-zinc-400"
          />
        </button>

        {/* Project Submenu */}
        <div className="absolute left-full top-0 ml-1 w-[200px] bg-[#2A2A2D] border border-white/10 rounded-xl shadow-2xl py-1.5 opacity-0 invisible group-hover/project:opacity-100 group-hover/project:visible transition-all duration-100 z-50">
          <div className="px-1.5 mb-1 pb-1 border-b border-white/5">
            <button className="w-full flex items-center gap-2.5 px-2.5 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 rounded-lg transition-colors text-left">
              <FolderPlus size={14} className="text-zinc-400" />
              新项目
            </button>
          </div>
          <div className="max-h-[220px] overflow-y-auto custom-scrollbar px-1.5 space-y-0.5">
            {projectsList.map((project) => (
              <button
                key={project}
                className="w-full flex items-center gap-2.5 px-2.5 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-indigo-500/10 hover:border-indigo-500/20 border border-transparent rounded-lg transition-colors text-left group/item"
                onClick={() => {
                  onClose();
                  onMoveToProject(project);
                }}
              >
                <Folder
                  size={14}
                  className="text-zinc-400 group-hover/item:text-indigo-400 transition-colors"
                />
                <span className="truncate">{project}</span>
              </button>
            ))}
          </div>
        </div>
      </div>
      <div className="h-px bg-white/10 my-1.5 mx-2" />
      <button
        onClick={(e) => {
          onTogglePin(e, session.id);
          onClose();
        }}
        className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left"
      >
        <Pin size={14} className="text-zinc-400" />
        {isPinned ? "取消置顶" : "置顶聊天"}
      </button>
      <button className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-zinc-300 hover:text-white hover:bg-white/5 transition-colors text-left">
        <Archive size={14} className="text-zinc-400" />
        归档
      </button>
      {canDelete && (
        <button
          onClick={(e) => {
            onDeleteSession(e, session.id);
            onClose();
          }}
          className="w-full flex items-center gap-3 px-3 py-2 text-[13px] text-red-400 hover:text-red-300 hover:bg-red-400/10 transition-colors text-left mt-0.5"
        >
          <Trash2 size={14} />
          删除
        </button>
      )}
    </div>,
    document.body,
  );
};
