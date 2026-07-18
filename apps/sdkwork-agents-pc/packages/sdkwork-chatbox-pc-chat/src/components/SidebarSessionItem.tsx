import React, { useState, useRef, useEffect } from 'react';
import { Pin, MoreHorizontal, Check, X } from 'lucide-react';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';
import { ChatSession } from '@/packages/sdkwork-chatbox-pc-core/src/sdk/types';
import { SidebarSessionContextMenu } from './SidebarSessionContextMenu';

interface SidebarSessionItemProps {
  session: ChatSession;
  currentSessionId: string;
  isPinned: boolean;
  activeDropdown: string | null;
  dropdownPos: { top: number, left: number };
  onSelectSession: (id: string) => void;
  togglePin: (e: React.MouseEvent, id: string) => void;
  onDeleteSession: (e: React.MouseEvent, id: string) => void;
  onRenameSession?: (id: string, newTitle: string) => void;
  handleDropdownClick: (e: React.MouseEvent, id: string) => void;
  setActiveDropdown: (id: string | null) => void;
  t: (key: string) => string;
  projectsList: string[];
  onMoveToProject: (project: string) => void;
  canDelete: boolean;
}

export const SidebarSessionItem: React.FC<SidebarSessionItemProps> = ({
  session,
  currentSessionId,
  isPinned,
  activeDropdown,
  dropdownPos,
  onSelectSession,
  togglePin,
  onDeleteSession,
  onRenameSession,
  handleDropdownClick,
  setActiveDropdown,
  t,
  projectsList,
  onMoveToProject,
  canDelete
}) => {
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const [editedTitle, setEditedTitle] = useState(session.title);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isEditingTitle) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [isEditingTitle]);

  const handleSaveRename = (e?: React.MouseEvent | React.KeyboardEvent) => {
    e?.stopPropagation();
    if (editedTitle.trim() && editedTitle !== session.title && onRenameSession) {
      onRenameSession(session.id, editedTitle.trim());
    }
    setIsEditingTitle(false);
  };

  const handleCancelRename = (e?: React.MouseEvent | React.KeyboardEvent) => {
    e?.stopPropagation();
    setEditedTitle(session.title);
    setIsEditingTitle(false);
  };

  return (
    <div
      onClick={() => {
        if (!isEditingTitle) {
          onSelectSession(session.id);
        }
      }}
      className={cn(
        "group flex items-center justify-between gap-2 rounded-[8px] cursor-pointer transition-all px-2.5 py-2.5 text-[13px] w-full relative",
        currentSessionId === session.id 
          ? "bg-[#27272A] text-white font-medium shadow-sm ring-1 ring-white/5" 
          : "hover:bg-[#27272A]/50 text-zinc-400 hover:text-zinc-100"
      )}
    >
      <div className="flex items-center gap-2.5 truncate flex-1 min-w-0 pr-10 relative">
        <div className={cn(
          "w-6 h-6 rounded-md flex items-center justify-center shrink-0 text-[10px] font-bold shadow-sm transition-colors", 
          currentSessionId === session.id ? "bg-[#3f3f46] text-white" : "bg-[#27272A] text-zinc-400 group-hover:bg-[#3f3f46] group-hover:text-white"
        )}>
          {session.title.substring(0, 1).toUpperCase()}
        </div>
        
        {isEditingTitle ? (
          <input
            ref={inputRef}
            type="text"
            value={editedTitle}
            onChange={e => setEditedTitle(e.target.value)}
            onKeyDown={e => {
              if (e.key === 'Enter') handleSaveRename(e);
              if (e.key === 'Escape') handleCancelRename(e);
            }}
            onClick={e => e.stopPropagation()}
            className="flex-1 bg-[#1a1a1c] border border-[#1890ff] text-white text-[13px] rounded px-1.5 py-0.5 outline-none w-full"
          />
        ) : (
          <>
            <span className="truncate flex-1 z-10">{session.title}</span>
            <div className={cn(
              "absolute right-0 top-0 bottom-0 w-8 z-20 transition-all",
              currentSessionId === session.id ? "bg-gradient-to-l from-[#27272A] to-transparent" : "opacity-0 group-hover:opacity-100 bg-gradient-to-l from-[#1C1C1E] group-hover:from-[#212124] to-transparent border-none"
            )} />
          </>
        )}
      </div>
      
      {!isEditingTitle && (
        <div className="absolute right-2 top-2.5 flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity z-30">
          <button
            onClick={(e) => {
              e.stopPropagation();
              togglePin(e, session.id);
            }}
            className="p-1 rounded text-zinc-400 hover:text-amber-400 hover:bg-zinc-700/50 transition-colors"
            title={isPinned ? t('unpinChat') : t('pinChat')}
          >
            {isPinned ? (
              <Pin size={12} className="fill-zinc-400 text-zinc-400 hover:fill-amber-400 hover:text-amber-400" />
            ) : (
              <Pin size={12} />
            )}
          </button>
          <div className="relative">
            <button
              onClick={(e) => handleDropdownClick(e, session.id)}
              className="p-1 rounded text-zinc-400 hover:text-zinc-200 hover:bg-zinc-700/50 transition-colors"
            >
              <MoreHorizontal size={14} />
            </button>
            {activeDropdown === session.id && (
              <SidebarSessionContextMenu
                session={session}
                isPinned={isPinned}
                dropdownPos={dropdownPos}
                onClose={() => setActiveDropdown(null)}
                onTogglePin={togglePin}
                onDeleteSession={onDeleteSession}
                onRename={() => {
                  setActiveDropdown(null);
                  setIsEditingTitle(true);
                }}
                projectsList={projectsList}
                onMoveToProject={onMoveToProject}
                canDelete={canDelete}
              />
            )}
          </div>
        </div>
      )}
    </div>
  );
};
