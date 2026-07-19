import React, { useState } from "react";
import { ChevronDown, Plus, MoreHorizontal, Folder } from "lucide-react";
import { cn } from "@sdkwork/agents-pc-commons";
import { SidebarProjectContextMenu } from "./SidebarProjectContextMenu";
import type { ChatProject } from "../services/ProjectService";

interface SidebarProjectsSectionProps {
  projects: ChatProject[];
  activeProject?: string | null;
  onProjectSelect?: (project: ChatProject) => void;
  onProjectSettings?: (project: ChatProject) => void;
  onProjectCreate?: (title: string) => void;
  onProjectRename?: (project: ChatProject, newTitle: string) => void;
  onProjectDelete?: (project: ChatProject) => void;
}

export const SidebarProjectsSection: React.FC<SidebarProjectsSectionProps> = ({
  projects,
  activeProject,
  onProjectSelect,
  onProjectSettings,
  onProjectCreate,
  onProjectRename,
  onProjectDelete,
}) => {
  const [activeProjectDropdown, setActiveProjectDropdown] = useState<string | null>(null);
  const [dropdownPos, setDropdownPos] = useState({ top: 0, left: 0 });
  const [editingProject, setEditingProject] = useState<string | null>(null);

  const handleProjectDropdownClick = (e: React.MouseEvent, projectId: string) => {
    e.stopPropagation();
    if (activeProjectDropdown === projectId) {
      setActiveProjectDropdown(null);
    } else {
      const rect = e.currentTarget.getBoundingClientRect();
      setDropdownPos({ top: rect.bottom, left: rect.right - 180 });
      setActiveProjectDropdown(projectId);
    }
  };

  // Close dropdown on click outside or scroll
  React.useEffect(() => {
    const handleClose = () => {
      setActiveProjectDropdown(null);
    };
    document.addEventListener("click", handleClose);
    document.addEventListener("scroll", handleClose, true);
    return () => {
      document.removeEventListener("click", handleClose);
      document.removeEventListener("scroll", handleClose, true);
    };
  }, []);

  return (
    <div className="px-2" id="sidebar-projects-section">
      <div className="flex items-center justify-between px-2.5 mb-2 group cursor-pointer">
        <div className="flex items-center gap-1.5 text-zinc-100 text-[15px] font-bold">
          项目 <ChevronDown size={14} className="text-zinc-400" />
        </div>
        <div className="flex items-center gap-2 text-zinc-500">
          <Plus
            size={16}
            className="cursor-pointer hover:text-zinc-100 transition-colors"
            onClick={(e) => {
              e.stopPropagation();
              const newTitle = `新项目 ${projects.length + 1}`;
              onProjectCreate?.(newTitle);
            }}
          />
          <MoreHorizontal
            size={16}
            className="cursor-pointer hover:text-zinc-100 transition-colors"
          />
        </div>
      </div>
      <div className="space-y-0.5">
        {projects.slice(0, 5).map((project) => (
          <div key={project.projectId} className="relative group">
            {editingProject === project.projectId ? (
              <div className="flex items-center justify-between text-zinc-300 font-medium rounded-lg w-full py-1.5 px-2.5 bg-indigo-500/10 border border-indigo-500/30">
                <div className="flex items-center gap-3 w-full">
                  <Folder
                    size={18}
                    className="text-indigo-400 shrink-0"
                  />
                  <input
                    type="text"
                    className="bg-transparent border-none outline-none text-[14px] text-white w-full h-full"
                    defaultValue={project.name}
                    autoFocus
                    onBlur={(e) => {
                      const newVal = e.target.value.trim();
                      if (newVal && newVal !== project.name) {
                        onProjectRename?.(project, newVal);
                      }
                      setEditingProject(null);
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.currentTarget.blur();
                      } else if (e.key === "Escape") {
                        setEditingProject(null);
                      }
                    }}
                  />
                </div>
              </div>
            ) : (
              <button
                onClick={() => onProjectSelect?.(project)}
                className={cn(
                  "flex items-center justify-between font-medium rounded-lg transition-all w-full py-2 px-2.5",
                  activeProject === project.projectId
                    ? "bg-indigo-500/15 text-white"
                    : "text-zinc-300 hover:bg-indigo-500/10 hover:text-white"
                )}
              >
                <div className="flex items-center gap-3 truncate">
                  <Folder
                    size={18}
                    className={cn(
                      "transition-colors shrink-0",
                      activeProject === project.projectId
                        ? "text-indigo-400"
                        : "text-zinc-400 group-hover:text-indigo-400"
                    )}
                  />
                  <span className="text-[14px] truncate">{project.name}</span>
                </div>
                <div
                  className="p-1 rounded text-zinc-500 opacity-0 group-hover:opacity-100 hover:text-zinc-300 hover:bg-zinc-700/50 transition-all shrink-0 cursor-pointer"
                  onClick={(e) => handleProjectDropdownClick(e, project.projectId)}
                >
                  <MoreHorizontal size={14} />
                </div>
              </button>
            )}
            {activeProjectDropdown === project.projectId && (
              <SidebarProjectContextMenu
                project={project.name}
                dropdownPos={dropdownPos}
                onClose={() => setActiveProjectDropdown(null)}
                onProjectSettings={() => onProjectSettings?.(project)}
                onProjectSelect={() => onProjectSelect?.(project)}
                onProjectRename={() => setEditingProject(project.projectId)}
                onProjectDelete={() => onProjectDelete?.(project)}
              />
            )}
          </div>
        ))}
        <button className="flex items-center gap-3 text-zinc-500 font-medium rounded-lg transition-all w-full py-1.5 px-2.5 hover:bg-white/5 hover:text-zinc-300 mt-1">
          <span className="text-[13px]">查看更多</span>
        </button>
      </div>
    </div>
  );
};
