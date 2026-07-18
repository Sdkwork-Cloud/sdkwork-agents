import type { LucideIcon } from 'lucide-react';
import {
  Bell,
  Bot,
  Gem,
  SlidersHorizontal,
  Sparkles,
  Terminal,
} from 'lucide-react';
import type { FC } from 'react';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';

export type WorkbenchTab = 'agents';

interface GlobalSidebarProps {
  activeTab: WorkbenchTab;
  avatarBg: string;
  setActiveTab: (tab: WorkbenchTab) => void;
  username: string;
}

interface SidebarItem {
  icon: LucideIcon;
  id: WorkbenchTab;
  label: string;
}

const SIDEBAR_ITEMS: SidebarItem[] = [
  { id: 'agents', icon: Bot, label: 'Agent' },
];

export const GlobalSidebar: FC<GlobalSidebarProps> = ({ activeTab, setActiveTab, avatarBg, username }) => (
  <div className="relative z-50 flex h-full w-[68px] shrink-0 flex-col items-center border-r border-white/5 bg-[#18181A] py-4">
    <div className="mb-6 flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center rounded-lg bg-gradient-to-br from-cyan-400 to-blue-600 shadow-sm">
      <Sparkles className="fill-white text-white" size={16} />
    </div>

    <nav aria-label="工作区" className="flex w-full flex-col gap-1 px-2">
      {SIDEBAR_ITEMS.map(({ id, icon: Icon, label }) => {
        const active = activeTab === id;
        return (
          <button
            aria-current={active ? 'page' : undefined}
            className={cn(
              'group relative flex w-full flex-col items-center justify-center rounded-xl py-3 transition-all duration-300',
              active
                ? 'bg-white/[0.05] text-white shadow-[inset_0_1px_1px_rgba(255,255,255,0.03)]'
                : 'text-zinc-500 hover:bg-white/5 hover:text-zinc-300',
            )}
            key={id}
            onClick={() => setActiveTab(id)}
            type="button"
          >
            <Icon
              className={cn(
                'transition-all duration-300',
                active ? 'scale-105 text-cyan-400' : 'text-zinc-500 group-hover:scale-105',
              )}
              fill={active ? 'currentColor' : 'none'}
              size={20}
            />
            <span className={cn('mt-1.5 text-[10px] font-medium tracking-wide transition-colors duration-300', active ? 'text-zinc-200' : 'text-zinc-500')}>
              {label}
            </span>
            {active && <div className="absolute bottom-[30%] left-0 top-[30%] w-[3px] rounded-r-sm bg-cyan-400" />}
          </button>
        );
      })}
    </nav>

    <div className="mt-auto flex w-full flex-col items-center gap-3 px-2">
      <button className="group flex w-full flex-col items-center justify-center rounded-xl py-2 text-cyan-500 transition-colors hover:bg-white/5" type="button">
        <Gem className="mb-1 group-hover:text-cyan-400" size={16} />
        <span className="text-[10px] font-bold">60</span>
        <span className="text-[9px]">开会员</span>
      </button>

      <button className="group relative my-2" type="button">
        <div className={cn('flex h-8 w-8 cursor-pointer items-center justify-center rounded-full text-xs font-bold shadow-sm ring-2 ring-transparent transition-all group-hover:ring-white/10', avatarBg)}>
          {username.substring(0, 2).toUpperCase()}
        </div>
        <div className="absolute -right-1 -top-1 flex h-3.5 w-3.5 items-center justify-center rounded-full border-[2px] border-[#18181A] bg-cyan-500 text-[8px] font-bold text-black">1</div>
      </button>

      <button className="flex h-10 w-10 items-center justify-center rounded-xl text-zinc-500 transition-colors hover:bg-white/5 hover:text-zinc-300" type="button">
        <Bell size={20} />
      </button>
      <button className="flex h-10 w-10 items-center justify-center rounded-xl text-zinc-500 transition-colors hover:bg-white/5 hover:text-zinc-300" type="button">
        <Terminal size={20} />
      </button>
      <button className="flex h-10 w-10 items-center justify-center rounded-xl text-zinc-500 transition-colors hover:bg-white/5 hover:text-zinc-300" type="button">
        <SlidersHorizontal size={20} />
      </button>
    </div>
  </div>
);
