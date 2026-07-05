import React, { useState, useEffect, useRef } from 'react';
import { Terminal, Code, PenTool, Sparkles, Image as ImageIcon, Plus, Bot, Zap, Globe, Compass, Edit2, Trash2 } from 'lucide-react';
import { cn } from '@sdkwork/agents-pc-commons';
import { agentService } from '../services/AgentService';
import type { AgentConfig } from '../services/AgentService';
import { toast } from '../components/Toast';

export interface Agent {
  id: string;
  name: string;
  desc: string;
  icon?: React.ReactNode;
  color?: string;
  author?: string;
  users?: string;
  avatar?: string;
  welcomeMessage?: string;
}

interface AgentViewProps {
  onStartChat: (agent: Agent) => void;
  onCreateAgent?: () => void;
  onEditAgent?: (id: string) => void;
}

export const AgentView: React.FC<AgentViewProps> = ({ onStartChat, onCreateAgent, onEditAgent }) => {
  const [activeCategory, setActiveCategory] = useState<string>('market');
  const [marketAgents, setMarketAgents] = useState<AgentConfig[]>([]);
  const [myAgents, setMyAgents] = useState<AgentConfig[]>([]);
  const [marketHasMore, setMarketHasMore] = useState(false);
  const [myHasMore, setMyHasMore] = useState(false);
  const [marketPage, setMarketPage] = useState(1);
  const [myPage, setMyPage] = useState(1);
  const [loading, setLoading] = useState(true);
  const [loadingMoreMarket, setLoadingMoreMarket] = useState(false);
  const [loadingMoreMy, setLoadingMoreMy] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  const categories = [
    { id: 'all', name: '全部智能体' },
    { id: 'tech', name: '技术开发' },
    { id: 'writing', name: '文案创作' },
    { id: 'design', name: 'UI/UX 设计' },
    { id: 'office', name: '效率办公' },
    { id: 'device', name: '硬件管理' }
  ];
  const [selectedMarketCategory, setSelectedMarketCategory] = useState<string>('all');

  const loadMarketPage = async (page: number, append: boolean, q?: string) => {
    if (page === 1 && !append) {
      setLoading(true);
    } else {
      setLoadingMoreMarket(true);
    }
    try {
      const result = await agentService.listAgentsPage({
        page,
        scope: 'market',
        ...(q?.trim() ? { q: q.trim() } : {}),
      });
      setMarketAgents((prev) => (append ? [...prev, ...result.items] : result.items));
      setMarketHasMore(result.pageInfo.hasMore);
      setMarketPage(page);
    } catch {
      toast('加载智能体失败', 'error');
    } finally {
      if (page === 1) {
        setLoading(false);
      } else {
        setLoadingMoreMarket(false);
      }
    }
  };

  const loadMyPage = async (page: number, append: boolean) => {
    if (page === 1) {
      setLoading(true);
    } else {
      setLoadingMoreMy(true);
    }
    try {
      const result = await agentService.listAgentsPage({ page });
      setMyAgents((prev) => (append ? [...prev, ...result.items] : result.items));
      setMyHasMore(result.pageInfo.hasMore);
      setMyPage(page);
    } catch {
      toast('加载智能体失败', 'error');
    } finally {
      if (page === 1) {
        setLoading(false);
      } else {
        setLoadingMoreMy(false);
      }
    }
  };

  useEffect(() => {
    void Promise.all([loadMarketPage(1, false), loadMyPage(1, false)]);
  }, []);

  const skipInitialSearchReload = useRef(true);
  useEffect(() => {
    if (skipInitialSearchReload.current) {
      skipInitialSearchReload.current = false;
      return undefined;
    }
    const timer = window.setTimeout(() => {
      void loadMarketPage(1, false, searchQuery);
    }, 300);
    return () => window.clearTimeout(timer);
  }, [searchQuery]);

  const getIcon = (iconName?: string) => {
    switch (iconName) {
      case 'Terminal': return <Terminal size={24} />;
      case 'PenTool': return <PenTool size={24} />;
      case 'Code': return <Code size={24} />;
      case 'Sparkles': return <Sparkles size={24} />;
      case 'Globe': return <Globe size={24} />;
      case 'Zap': return <Zap size={24} />;
      default: return <Bot size={24} />;
    }
  };

  const mapToAgent = (config: AgentConfig): Agent => ({
    id: config.id || '',
    name: config.name,
    desc: config.description,
    icon: getIcon(config.iconName),
    color: config.color || 'bg-blue-500',
    author: config.author || '我',
    users: config.users || '0',
    avatar: config.avatar,
    welcomeMessage: config.welcomeMessage,
  });

  const filteredMarketAgents = marketAgents.filter((agent) =>
    selectedMarketCategory === 'all' || agent.categoryId === selectedMarketCategory,
  );

  const handleDeleteAgent = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    if (window.confirm('确定要删除这个智能体吗？')) {
      try {
        await agentService.deleteAgent(id);
        setMyAgents(prev => prev.filter(a => a.id !== id));
        toast('删除成功', 'success');
      } catch (error) {
        toast('删除失败', 'error');
      }
    }
  };

  return (
    <div className="flex flex-1 min-h-0">
      {/* Left Category List */}
      <div className="flex w-[280px] shrink-0 flex-col bg-[#202020] border-r border-white/5 min-h-0">
        <div className="flex-1 overflow-y-auto custom-scrollbar py-2">
          <div className="px-4 py-2 text-xs text-gray-500 font-medium tracking-wide">发现</div>
          <div 
            onClick={() => setActiveCategory('market')}
            className={cn(
              "flex items-center px-4 py-3 cursor-pointer transition-all hover:bg-white/5",
              activeCategory === 'market' && "bg-blue-600/10 border-l-2 border-blue-500 text-blue-400"
            )}
          >
            <div className={cn("w-[28px] h-[28px] flex items-center justify-center shrink-0 mr-3", activeCategory === 'market' ? 'text-blue-500' : 'text-gray-400')}>
              <Compass size={18} />
            </div>
            <span className={cn("text-[14px]", activeCategory === 'market' ? 'font-semibold text-blue-400' : 'text-gray-300 font-medium')}>发现智能体</span>
          </div>

          <div className="px-4 py-2 mt-6 text-xs text-gray-500 font-medium tracking-wide">我的智能体</div>
          {loading ? (
            <div className="px-4 py-3 text-sm text-gray-500">加载中...</div>
          ) : myAgents.map(config => {
            const agent = mapToAgent(config);
            return (
              <div 
                key={agent.id}
                onClick={() => onStartChat(agent)}
                className="flex items-center px-4 py-3 cursor-pointer transition-colors hover:bg-white/5 group justify-between"
              >
                <div className="flex items-center flex-1 min-w-0 pr-2">
                  <div className={cn("w-[28px] h-[28px] rounded-lg flex items-center justify-center text-white shrink-0 mr-3 shadow-md shadow-black/20 group-hover:scale-105 transition-transform", agent.color)}>
                    {React.isValidElement(agent.icon) ? React.cloneElement(agent.icon as React.ReactElement<any>, { size: 14 }) : agent.icon}
                  </div>
                  <span className="text-[14px] text-gray-300 font-medium truncate group-hover:text-white transition-colors flex-1">{agent.name}</span>
                </div>
                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button 
                    className="p-1.5 text-gray-500 hover:text-blue-400 rounded transition-colors"
                    onClick={(e) => { e.stopPropagation(); onEditAgent && onEditAgent(agent.id); }}
                    title="编辑"
                  >
                    <Edit2 size={14} />
                  </button>
                  <button 
                    className="p-1.5 text-gray-500 hover:text-red-400 rounded transition-colors"
                    onClick={(e) => handleDeleteAgent(e, agent.id)}
                    title="删除"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            );
          })}
          {myHasMore && !loading && (
            <button
              type="button"
              className="mx-4 my-2 rounded-lg border border-white/10 px-3 py-2 text-xs text-gray-400 hover:text-white hover:bg-white/5"
              disabled={loadingMoreMy}
              onClick={() => void loadMyPage(myPage + 1, true)}
            >
              {loadingMoreMy ? '加载中...' : '加载更多'}
            </button>
          )}
        </div>
      </div>

      {/* Right Panel */}
      <div className="flex-1 flex flex-col bg-[#1e1e1e] min-w-0 p-6 lg:p-8 overflow-y-auto custom-scrollbar">
        <div className="w-full h-full flex flex-col">
          <div className="flex flex-col gap-6 mb-8 shrink-0">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-2xl font-bold text-gray-100 mb-2">探索智能体生态体系</h2>
                <p className="text-gray-500 text-sm">发掘领域专家与效率工具，全方位提升协作能力。</p>
              </div>
              <div className="flex items-center gap-4">
                <div className="relative hidden md:block">
                  <input 
                    type="text" 
                    placeholder="搜索能力、名称或描述..." 
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-72 bg-[#141414] border border-white/10 rounded-xl pl-10 pr-4 py-2.5 text-sm text-gray-200 outline-none focus:border-blue-500 focus:bg-[#181818] transition-all shadow-inner"
                  />
                  <Globe className="absolute left-3.5 top-1/2 -translate-y-1/2 text-gray-500" size={16} />
                </div>
                <button 
                  onClick={onCreateAgent}
                  className="flex items-center gap-2 px-5 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-xl transition-colors text-sm font-semibold shadow-lg shadow-blue-500/20 hover:shadow-blue-500/40"
                >
                  <Plus size={18} />
                  创建新智能体
                </button>
              </div>
            </div>
            
            {/* Category Tabs */}
            <div className="flex items-center gap-2 overflow-x-auto custom-scrollbar pb-2">
              {categories.map(cat => (
                <button
                  key={cat.id}
                  onClick={() => setSelectedMarketCategory(cat.id)}
                  className={cn(
                    "px-4 py-1.5 rounded-full text-sm font-medium transition-colors whitespace-nowrap border",
                    selectedMarketCategory === cat.id 
                      ? "bg-blue-500/10 text-blue-400 border-blue-500/30" 
                      : "bg-[#252528] text-gray-400 border-white/5 hover:bg-white/5 hover:text-gray-200"
                  )}
                >
                  {cat.name}
                </button>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-6 pb-20">
            {loading ? (
              <div className="text-gray-500 text-sm col-span-full py-20 text-center">正在加载智能体核心数据...</div>
            ) : filteredMarketAgents.length === 0 ? (
              <div className="text-gray-500 text-sm col-span-full py-20 text-center">未找到符合条件的智能体</div>
            ) : filteredMarketAgents.map(config => {
              const agent = mapToAgent(config);
              return (
                <div 
                  key={agent.id} 
                  onClick={() => onStartChat(agent)}
                  className="bg-[#242426] rounded-2xl border border-white/5 p-6 hover:border-blue-500/40 transition-all hover:-translate-y-1.5 hover:shadow-2xl hover:shadow-blue-500/10 cursor-pointer flex flex-col group relative overflow-hidden"
                >
                  <div className="absolute top-0 right-0 w-32 h-32 bg-blue-500/5 rounded-full blur-[50px] pointer-events-none group-hover:bg-blue-500/10 transition-colors"></div>
                  
                  <div className="flex items-start justify-between mb-5 relative z-10">
                    <div className={cn("w-14 h-14 rounded-2xl flex items-center justify-center text-white shadow-lg ring-1 ring-white/10 group-hover:scale-105 transition-transform", agent.color)}>
                      {React.isValidElement(agent.icon) ? React.cloneElement(agent.icon as React.ReactElement<any>, { size: 28 }) : agent.icon}
                    </div>
                    <button className="px-4 py-1.5 rounded-lg bg-[#1e1e1e] hover:bg-blue-500 hover:text-white text-gray-300 text-xs font-semibold transition-all opacity-0 group-hover:opacity-100 border border-white/5 shadow-sm">
                      唤醒对话
                    </button>
                  </div>
                  <h3 className="text-lg font-bold text-gray-100 mb-2 group-hover:text-blue-400 transition-colors tracking-wide relative z-10">{agent.name}</h3>
                  <p className="text-sm text-gray-400 line-clamp-3 mb-6 flex-1 leading-relaxed relative z-10">{agent.desc}</p>
                  
                  <div className="flex items-center justify-between text-xs text-gray-500 pt-4 border-t border-white/5 mt-auto relative z-10">
                    <span className="flex items-center gap-1.5 font-medium"><Bot size={14}/> {agent.author}</span>
                    <span className="bg-[#181818] border border-white/5 px-2.5 py-1 rounded-md text-gray-400 tracking-wider font-mono">{agent.users} USERS</span>
                  </div>
                </div>
              );
            })}
          </div>
          {marketHasMore && !loading && (
            <div className="flex justify-center pb-8">
              <button
                type="button"
                className="rounded-xl border border-white/10 px-5 py-2.5 text-sm text-gray-300 hover:text-white hover:bg-white/5"
                disabled={loadingMoreMarket}
                onClick={() => void loadMarketPage(marketPage + 1, true)}
              >
                {loadingMoreMarket ? '加载中...' : '加载更多智能体'}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
