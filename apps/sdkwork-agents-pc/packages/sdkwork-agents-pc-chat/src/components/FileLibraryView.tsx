import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Search, List as ListIcon, LayoutGrid, FileText, Image as ImageIcon, Presentation, RefreshCw } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import {
  chatFileLibraryService,
  type ChatLibraryFile,
} from '../services/chatFileLibraryService';

const PAGE_SIZE = 100;
const MAX_PAGES = 10;

type FileKind = 'image' | 'ppt' | 'md' | 'other';

const getFileKind = (file: ChatLibraryFile): FileKind => {
  if (file.mimeType?.startsWith('image/')) return 'image';
  const extension = file.name.split('.').pop()?.toLowerCase();
  if (extension === 'ppt' || extension === 'pptx') return 'ppt';
  if (extension === 'md' || extension === 'markdown') return 'md';
  return 'other';
};

const formatDate = (iso: string): string => {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return '-';
  return `${date.getMonth() + 1}月${date.getDate()}日`;
};

const formatSize = (bytes?: string): string => {
  if (!bytes) return '-';
  const value = Number(bytes);
  if (!Number.isFinite(value) || value <= 0) return '-';
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 * 1024 * 1024) return `${(value / (1024 * 1024)).toFixed(2)} MB`;
  return `${(value / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

export const FileLibraryView = () => {
  const [activeTab, setActiveTab] = useState<'all' | 'image' | 'file'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [files, setFiles] = useState<ChatLibraryFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [openingId, setOpeningId] = useState<string | null>(null);

  const loadFiles = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const collected: ChatLibraryFile[] = [];
      let cursor: string | undefined;
      for (let page = 0; page < MAX_PAGES; page += 1) {
        const result = await chatFileLibraryService.listFiles(PAGE_SIZE, cursor);
        collected.push(...result.items);
        if (!result.nextCursor) break;
        cursor = result.nextCursor;
      }
      setFiles(collected);
    } catch (cause) {
      console.error('Failed to load chat file library', cause);
      setError('文件库加载失败，请稍后重试');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadFiles();
  }, [loadFiles]);

  const handleOpenFile = useCallback(async (file: ChatLibraryFile) => {
    // Open the tab synchronously inside the user gesture so popup blockers
    // don't discard it, then navigate once the short-lived download URL is
    // resolved.
    const popup = window.open('', '_blank');
    setOpeningId(file.id);
    try {
      const url = await chatFileLibraryService.resolvePreviewUrl(file.id);
      if (!popup) {
        // Popup blocked: fall back to a synthetic anchor click.
        const anchor = document.createElement('a');
        anchor.href = url;
        anchor.target = '_blank';
        anchor.rel = 'noopener noreferrer';
        document.body.appendChild(anchor);
        anchor.click();
        anchor.remove();
        return;
      }
      popup.opener = null;
      popup.location.href = url;
    } catch (cause) {
      popup?.close();
      console.error('Failed to resolve file preview url', cause);
    } finally {
      setOpeningId(null);
    }
  }, []);

  const getFileIcon = (kind: FileKind) => {
    switch (kind) {
      case 'image':
        return (
          <div className="w-7 h-7 rounded bg-blue-500/20 flex items-center justify-center shrink-0">
            <ImageIcon size={14} className="text-blue-400" />
          </div>
        );
      case 'ppt':
        return (
          <div className="w-7 h-7 rounded bg-red-500/20 flex items-center justify-center shrink-0">
            <Presentation size={14} className="text-red-400" />
          </div>
        );
      case 'md':
        return (
          <div className="w-7 h-7 rounded bg-blue-500/20 flex items-center justify-center shrink-0">
            <FileText size={14} className="text-blue-400" />
          </div>
        );
      default:
        return (
          <div className="w-7 h-7 rounded bg-zinc-500/20 flex items-center justify-center shrink-0">
            <FileText size={14} className="text-zinc-400" />
          </div>
        );
    }
  };

  const filteredFiles = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    return files.filter((file) => {
      if (query && !file.name.toLowerCase().includes(query)) return false;
      const kind = getFileKind(file);
      if (activeTab === 'image') return kind === 'image';
      if (activeTab === 'file') return kind !== 'image';
      return true;
    });
  }, [files, searchQuery, activeTab]);

  return (
    <div className="flex flex-col h-full w-full bg-[#000000] text-gray-200 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-10 pt-10 pb-6 shrink-0">
        <h1 className="text-3xl font-bold text-white">资料库</h1>
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <Search size={16} className="text-zinc-400" />
            </div>
            <input
              type="text"
              placeholder="搜索"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="bg-[#1C1C1E] border border-white/5 rounded-full pl-9 pr-4 py-1.5 text-[14px] text-white placeholder:text-zinc-500 focus:outline-none focus:border-white/20 transition-colors w-[240px]"
            />
          </div>
          <button
            onClick={() => void loadFiles()}
            disabled={loading}
            className="flex items-center gap-2 bg-white/5 text-white hover:bg-white/10 rounded-full px-3 py-1.5 text-[14px] font-medium transition-colors disabled:opacity-50"
            title="刷新"
          >
            <RefreshCw size={14} className={cn(loading && 'animate-spin')} />
            刷新
          </button>
        </div>
      </div>

      {/* Tabs & Controls */}
      <div className="flex items-center justify-between px-10 pb-4 shrink-0">
        <div className="flex items-center gap-2">
          {(['全部', '图片', '文件'] as const).map((label, idx) => {
            const tabKey = idx === 0 ? 'all' : idx === 1 ? 'image' : 'file';
            return (
              <button
                key={tabKey}
                onClick={() => setActiveTab(tabKey)}
                className={cn(
                  "px-4 py-1.5 rounded-full text-[14px] font-medium transition-colors",
                  activeTab === tabKey
                    ? "bg-[#2A2A2D] text-white"
                    : "text-zinc-400 hover:text-white"
                )}
              >
                {label}
              </button>
            );
          })}
        </div>

        <div className="flex items-center gap-4">
          <button className="text-zinc-400 hover:text-white transition-colors">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M3 6h18M6 12h12M10 18h4" />
            </svg>
          </button>
          <div className="w-[1px] h-4 bg-white/10" />
          <button className="text-zinc-400 hover:text-white transition-colors">
            <LayoutGrid size={18} />
          </button>
          <button className="w-8 h-8 rounded-full bg-[#2A2A2D] flex items-center justify-center text-white transition-colors">
            <ListIcon size={18} />
          </button>
        </div>
      </div>

      {/* Table Header */}
      <div className="flex items-center px-10 py-3 border-b border-white/5 text-[13px] text-zinc-400 font-medium shrink-0">
        <div className="flex-1 pr-4">名称</div>
        <div className="w-[180px] shrink-0">修改时间 &darr;</div>
        <div className="w-[120px] shrink-0">大小</div>
      </div>

      {/* File List */}
      <div className="flex-1 overflow-y-auto px-10 py-2 custom-scrollbar">
        {loading ? (
          <div className="py-20 text-center text-zinc-500 text-[14px]">加载中...</div>
        ) : error ? (
          <div className="py-20 text-center">
            <div className="text-zinc-500 text-[14px] mb-4">{error}</div>
            <button
              onClick={() => void loadFiles()}
              className="bg-white/10 hover:bg-white/15 text-white rounded-full px-5 py-1.5 text-[14px] font-medium transition-colors"
            >
              重试
            </button>
          </div>
        ) : (
          <div className="flex flex-col space-y-1 pb-10">
            {filteredFiles.map((file) => (
              <div
                key={file.id}
                onClick={() => void handleOpenFile(file)}
                className="flex items-center py-4 border-b border-white/5 hover:bg-white/[0.02] transition-colors cursor-pointer group"
              >
                <div className="flex-1 flex items-center gap-3 min-w-0 pr-4">
                  {getFileIcon(getFileKind(file))}
                  <span className="text-[14px] text-white truncate font-medium group-hover:text-blue-400 transition-colors">
                    {file.name}
                  </span>
                </div>
                <div className="w-[180px] shrink-0 text-[14px] text-zinc-400">
                  {formatDate(file.updatedAt)}
                </div>
                <div className="w-[120px] shrink-0 text-[14px] text-zinc-400">
                  {openingId === file.id ? '打开中...' : formatSize(file.sizeBytes)}
                </div>
              </div>
            ))}
            {filteredFiles.length === 0 && (
              <div className="py-20 text-center text-zinc-500 text-[14px]">
                无文件
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
