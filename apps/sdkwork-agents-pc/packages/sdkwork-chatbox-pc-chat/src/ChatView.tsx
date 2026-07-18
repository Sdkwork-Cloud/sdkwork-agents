import React, { useState, useRef, useEffect, useCallback } from "react";
import {
  ChatMessage,
  ChatSession,
} from "@/packages/sdkwork-chatbox-pc-core/src/sdk/types";
import { cn } from "@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer";
import { ChatService } from "@sdkwork/chatbox-pc-core";
import { ProjectService } from "@/packages/sdkwork-chatbox-pc-core/src/services/ProjectService";
import { useTranslation } from "react-i18next";
import {
  agentsDriveUploadService,
  type AgentsDriveMediaResource,
} from "@sdkwork/agents-pc-core/sdk";
import { uuid } from "@sdkwork/utils";

import { Sidebar } from "./components/Sidebar";
import { ChatHeader } from "./components/ChatHeader";
import { MessageList } from "./components/MessageList";
import { ChatInput } from "./components/ChatInput";
import { ArtifactPanel } from "./components/ArtifactPanel";
import { SettingsModal } from "./components/SettingsModal";
import { UserProfileModal } from "./components/UserProfileModal";
import { ProjectHomeView } from "./components/ProjectHomeView";
import { ProjectSettingsModal } from "./components/ProjectSettingsModal";
import { FileLibraryView } from "./components/FileLibraryView";

export const ChatView = () => {
  const { t } = useTranslation("chat");

  const [sessions, setSessions] = useState<ChatSession[]>([
    { id: "1", title: t("newChat"), messages: [], updatedAt: Date.now() },
  ]);
  const [currentSessionId, setCurrentSessionId] = useState<string>("1");
  const [activeView, setActiveView] = useState<'chat' | 'library'>('chat');
  const [projects, setProjects] = useState<string[]>([]);
  const [activeProject, setActiveProject] = useState<string | null>(null);
  const [activeProjectSettings, setActiveProjectSettings] = useState<
    string | null
  >(null);
  const [input, setInput] = useState(() => {
    try {
      return localStorage.getItem("chat_input_draft") || "";
    } catch {
      return "";
    }
  });

  useEffect(() => {
    ProjectService.getProjects().then(setProjects);
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem("chat_input_draft", input);
    } catch {
      // ignore
    }
  }, [input]);
  const [selectedImages, setSelectedImages] = useState<string[]>([]);
  const [selectedMediaResources, setSelectedMediaResources] = useState<AgentsDriveMediaResource[]>([]);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);
  const [isGenerating, setIsGenerating] = useState(false);

  const [selectedModel, setSelectedModel] = useState("gemini-2.5-flash");
  const [selectedVendor, setSelectedVendor] = useState("Google");
  const [isModelSelectorOpen, setIsModelSelectorOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [profileModalTab, setProfileModalTab] = useState<
    "profile" | "billing" | null
  >(null);
  const [shouldAutoScroll, setShouldAutoScroll] = useState(true);

  const [artifact, setArtifact] = useState<{
    language: string;
    code: string;
    mode: "preview" | "code";
  } | null>(null);
  const [isArtifactOpen, setIsArtifactOpen] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  const currentSession = sessions.find((s) => s.id === currentSessionId)!;

  const handleScroll = () => {
    if (!scrollContainerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } =
      scrollContainerRef.current;
    const isAtBottom = scrollHeight - scrollTop - clientHeight <= 100;
    setShouldAutoScroll(isAtBottom);
  };

  useEffect(() => {
    if (sessions.length === 1 && sessions[0].messages.length === 0 && sessions[0].title !== t("newChat")) {
      setSessions((prev) => [{ ...prev[0], title: t("newChat") }]);
    }
  }, [t, sessions]);

  useEffect(() => {
    const scrollContainer = scrollContainerRef.current;
    if (!scrollContainer) return;

    const observer = new MutationObserver(() => {
      if (shouldAutoScroll) {
        messagesEndRef.current?.scrollIntoView({ behavior: "auto" });
      }
    });

    observer.observe(scrollContainer, {
      childList: true,
      subtree: true,
      characterData: true
    });

    return () => observer.disconnect();
  }, [shouldAutoScroll]);

  useEffect(() => {
    if (shouldAutoScroll) {
      messagesEndRef.current?.scrollIntoView({ behavior: "auto" });
    }
  }, [currentSession?.messages, shouldAutoScroll, isGenerating]);

  // Parse for artifacts when generation completes
  useEffect(() => {
    if (!isGenerating && currentSession?.messages?.length > 0) {
      const lastMessage =
        currentSession.messages[currentSession.messages.length - 1];
      if (lastMessage.role === "model" && lastMessage.text) {
        const regex =
          /```(html|js|javascript|jsx|ts|typescript|tsx|css|json|markdown|md|svg|xml)\n([\s\S]*?)```/g;
        let lastMatch;
        let match;
        while ((match = regex.exec(lastMessage.text)) !== null) {
          lastMatch = match;
        }
        if (lastMatch) {
          const lang = lastMatch[1].toLowerCase();
          const code = lastMatch[2];
          setArtifact({
            language: lang,
            code,
            mode: ["html", "svg", "xml"].includes(lang) ? "preview" : "code",
          });
          setIsArtifactOpen(true);
        }
      }
    }
  }, [isGenerating, currentSession?.id]);

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  const handleNewChat = useCallback(() => {
    const newSession: ChatSession = {
      id: uuid(),
      title: t("newChat"),
      messages: [],
      updatedAt: Date.now(),
    };
    setSessions((prev) => [newSession, ...prev]);
    setCurrentSessionId(newSession.id);
    setActiveProject(null);
    setActiveView('chat');
    setIsArtifactOpen(false);
    setShouldAutoScroll(true);
  }, [t]);

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        if (e.key.toLowerCase() === 'n') {
          e.preventDefault();
          handleNewChat();
        } else if (e.key.toLowerCase() === 'b') {
          e.preventDefault();
          setIsSidebarOpen(prev => !prev);
        } else if (e.key.toLowerCase() === 'f') {
          e.preventDefault();
          setIsSidebarOpen(true);
          setTimeout(() => {
            document.getElementById('sidebar-search-input')?.focus();
          }, 100);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleNewChat]);

  const handleRenameSession = (id: string, newTitle: string) => {
    setSessions((prev) =>
      prev.map((s) => (s.id === id ? { ...s, title: newTitle } : s))
    );
  };

  const handleCreateProject = (title: string) => {
    setProjects(prev => [title, ...prev]);
  };

  const handleRenameProject = (oldTitle: string, newTitle: string) => {
    setProjects(prev => prev.map(p => p === oldTitle ? newTitle : p));
    if (activeProject === oldTitle) setActiveProject(newTitle);
  };

  const handleDeleteProject = (title: string) => {
    setProjects(prev => prev.filter(p => p !== title));
    if (activeProject === title) setActiveProject(null);
  };

  const handleSelectSession = (id: string) => {
    setCurrentSessionId(id);
    setActiveProject(null);
    setActiveView('chat');
    setIsArtifactOpen(false);
    setShouldAutoScroll(true);
  };

  const handleDeleteSession = (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    setSessions((prev) => prev.filter((s) => s.id !== id));
    if (currentSessionId === id) {
      setCurrentSessionId(sessions.find((s) => s.id !== id)?.id || "");
    }
  };

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const files: File[] = [];
    for (let index = 0; index < (event.target.files?.length ?? 0); index += 1) {
      const file = event.target.files?.item(index);
      if (file?.type.startsWith('image/')) files.push(file);
    }
    event.target.value = '';
    if (files.length === 0) return;

    void Promise.all(files.map((file) => agentsDriveUploadService.upload({
      file,
      purpose: 'agent-chat-image',
      resourceId: `chatbox:${currentSessionId}`,
    })))
      .then((uploaded) => {
        setSelectedMediaResources((previous) => [...previous, ...uploaded]);
        setSelectedImages((previous) => [...previous, ...uploaded.map((media) => media.url)]);
      })
      .catch((error) => {
        console.error('Chat image Drive upload failed', error);
      });
  };

  const handleRemoveImage = (index: number) => {
    setSelectedImages((prev) => prev.filter((_, i) => i !== index));
    setSelectedMediaResources((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSend = async () => {
    if ((!input.trim() && selectedImages.length === 0) || isGenerating) return;

    setShouldAutoScroll(true);

    const userMessage: ChatMessage = {
      id: uuid(),
      role: "user",
      text: input.trim(),
      images: selectedImages.length > 0 ? [...selectedImages] : undefined,
      mediaResources: selectedMediaResources.length > 0 ? [...selectedMediaResources] : undefined,
    };

    setShouldAutoScroll(true);
    setInput("");
    setSelectedImages([]);
    setSelectedMediaResources([]);
    setIsGenerating(true);

    // Add user message
    setSessions((prev) =>
      prev.map((s) => {
        if (s.id === currentSessionId) {
          // Auto-generate title for first message
          const titleText = userMessage.text || t("imageChat");
          const title =
            s.messages.length === 0
              ? titleText.slice(0, 30) + (titleText.length > 30 ? "..." : "")
              : s.title;
          return {
            ...s,
            title,
            updatedAt: Date.now(),
            messages: [...s.messages, userMessage],
          };
        }
        return s;
      }),
    );

    // Add empty model message placeholder
    const modelMessageId = uuid();
    setSessions((prev) =>
      prev.map((s) =>
        s.id === currentSessionId
          ? {
              ...s,
              messages: [
                ...s.messages,
                { id: modelMessageId, role: "model", text: "" },
              ],
            }
          : s,
      ),
    );

    const updatedSession = sessions.find((s) => s.id === currentSessionId)!;
    const requestMessages = updatedSession.messages.concat(userMessage);

    // Abort previous generation if exist
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    const abortController = new AbortController();
    abortControllerRef.current = abortController;

    try {
      await ChatService.streamChat({
        model: selectedModel,
        messages: requestMessages,
        signal: abortController.signal,
        onMessageUpdate: (text) => {
          setSessions((prev) =>
            prev.map((s) => {
              if (s.id === currentSessionId) {
                return {
                  ...s,
                  messages: s.messages.map((m) =>
                    m.id === modelMessageId ? { ...m, text: m.text + text } : m,
                  ),
                };
              }
              return s;
            }),
          );
        },
        onComplete: () => {
          setIsGenerating(false);
          abortControllerRef.current = null;
        },
        onError: (err) => {
          if (err !== "AbortError") {
            setSessions((prev) =>
              prev.map((s) => {
                if (s.id === currentSessionId) {
                  return {
                    ...s,
                    messages: s.messages.map((m) =>
                      m.id === modelMessageId
                        ? {
                            ...m,
                            text: m.text + `\n\n**${t("generatingError")}**`,
                          }
                        : m,
                    ),
                  };
                }
                return s;
              }),
            );
          }
          setIsGenerating(false);
          abortControllerRef.current = null;
        },
      });
    } catch (e: any) {
      if (e.name !== "AbortError") {
        setIsGenerating(false);
        abortControllerRef.current = null;
      }
    }
  };

  const handleStop = () => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
      setIsGenerating(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex h-full w-full bg-[#f5f5f5] dark:bg-[#191919] font-sans text-gray-900 dark:text-gray-100 overflow-hidden">
      <Sidebar
        isOpen={isSidebarOpen}
        toggleSidebar={() => setIsSidebarOpen(!isSidebarOpen)}
        sessions={sessions}
        currentSessionId={currentSessionId}
        onNewChat={handleNewChat}
        onSelectSession={handleSelectSession}
        onDeleteSession={handleDeleteSession}
        onRenameSession={handleRenameSession}
        onOpenUserProfile={(tab) => setProfileModalTab(tab)}
        onOpenFileLibrary={() => setActiveView('library')}
        projects={projects}
        activeProject={activeProject}
        onProjectSelect={(project) => setActiveProject(project)}
        onProjectSettings={(project) => setActiveProjectSettings(project)}
        onProjectCreate={handleCreateProject}
        onProjectRename={handleRenameProject}
        onProjectDelete={handleDeleteProject}
      />

      <div
        className={cn(
          "flex flex-col h-full relative bg-[#f5f5f5] dark:bg-[#191919] transition-all duration-300",
          isArtifactOpen
            ? "flex-1 min-w-[300px] border-r border-[#d9d9d9] dark:border-[#1a1a1a]"
            : "flex-1 min-w-0",
        )}
      >
        {activeView === 'library' ? (
          <FileLibraryView />
        ) : !activeProject ? (
          <>
            <ChatHeader
              isSidebarOpen={isSidebarOpen}
              toggleSidebar={() => setIsSidebarOpen(!isSidebarOpen)}
              selectedModel={selectedModel}
              selectedVendor={selectedVendor}
              isModelSelectorOpen={isModelSelectorOpen}
              setIsModelSelectorOpen={setIsModelSelectorOpen}
              setSelectedVendor={setSelectedVendor}
              setSelectedModel={setSelectedModel}
              onOpenSettings={() => setIsSettingsOpen(true)}
            />

            <div
              className="flex-1 overflow-y-auto w-full"
              ref={scrollContainerRef}
              onScroll={handleScroll}
            >
              <MessageList
                messages={currentSession?.messages || []}
                messagesEndRef={messagesEndRef}
                onOpenArtifact={(lang, code, mode) => {
                  const finalMode =
                    mode ||
                    (["html", "svg", "xml", "md", "markdown"].includes(
                      lang.toLowerCase(),
                    )
                      ? "preview"
                      : "code");
                  setArtifact({
                    language: lang.toLowerCase(),
                    code,
                    mode: finalMode,
                  });
                  setIsArtifactOpen(true);
                }}
              />
            </div>

            <ChatInput
              input={input}
              setInput={setInput}
              selectedImages={selectedImages}
              isGenerating={isGenerating}
              textareaRef={textareaRef}
              fileInputRef={fileInputRef}
              handleFileChange={handleFileChange}
              handleRemoveImage={handleRemoveImage}
              handleSend={handleSend}
              handleStop={handleStop}
              handleKeyDown={handleKeyDown}
            />
          </>
        ) : (
          <ProjectHomeView projectName={activeProject} />
        )}
      </div>

      {isArtifactOpen && (
        <ArtifactPanel
          artifact={artifact}
          onClose={() => setIsArtifactOpen(false)}
          onModeChange={(mode) =>
            setArtifact((prev) => (prev ? { ...prev, mode } : null))
          }
          onCodeChange={(code) =>
            setArtifact((prev) => (prev ? { ...prev, code } : null))
          }
        />
      )}

      {isSettingsOpen && (
        <SettingsModal onClose={() => setIsSettingsOpen(false)} />
      )}
      {activeProjectSettings && (
        <ProjectSettingsModal
          projectName={activeProjectSettings}
          onClose={() => setActiveProjectSettings(null)}
        />
      )}
      {profileModalTab !== null && (
        <UserProfileModal
          initialTab={profileModalTab}
          onClose={() => setProfileModalTab(null)}
        />
      )}
    </div>
  );
};
