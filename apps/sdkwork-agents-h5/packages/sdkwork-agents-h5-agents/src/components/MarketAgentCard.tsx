import React, { useRef } from 'react';
import { ArrowRight } from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import type { AgentConfig } from '../services/AgentService';
import { createDefaultAvatar } from '../services/DefaultAvatarService';

export interface MarketAgentCardProps {
  agent: AgentConfig;
  /** Tap target: host-navigated "start a chat with this agent". */
  onStartChat?: (agent: AgentConfig) => void;
}

/**
 * Agent marketplace row card (shared by the market tab and the search view).
 *
 * Layout follows the original IM H5 agent tab design: circular avatar,
 * name / two-line description, optional `users · author` meta and a trailing
 * chat affordance. Every field renders only when the real catalog record
 * provides it — no mock placeholders.
 */
export const MarketAgentCard: React.FC<MarketAgentCardProps> = ({
  agent,
  onStartChat,
}) => {
  const avatar = agent.avatar || createDefaultAvatar('agent');
  const longPressTimer = useRef<number | null>(null);
  const longPressTriggered = useRef(false);

  const beginLongPress = () => {
    longPressTimer.current = window.setTimeout(() => {
      longPressTriggered.current = true;
    }, 500);
  };
  const cancelLongPress = () => {
    if (longPressTimer.current !== null) {
      window.clearTimeout(longPressTimer.current);
      longPressTimer.current = null;
    }
  };

  const meta = [agent.users, agent.author].filter(Boolean).join(' · ');

  return (
    <div
      role="button"
      tabIndex={0}
      className="flex items-start gap-3 px-4 py-3.5 select-none touch-callout-none transition-colors active:bg-black/5 dark:active:bg-white/5 cursor-pointer"
      onClick={() => {
        if (longPressTriggered.current) {
          longPressTriggered.current = false;
          return;
        }
        onStartChat?.(agent);
      }}
      onPointerDown={beginLongPress}
      onPointerUp={cancelLongPress}
      onPointerLeave={cancelLongPress}
      onKeyDown={(event) => {
        if (event.key === 'Enter') onStartChat?.(agent);
      }}
    >
      <div className="relative shrink-0">
        <div className="w-[52px] h-[52px] rounded-full overflow-hidden bg-black/5 dark:bg-white/10 border border-black/5 dark:border-white/10">
          <img src={avatar} alt={agent.name} className="w-full h-full object-cover" draggable={false} />
        </div>
      </div>

      <div className="flex-1 min-w-0 border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] pb-3.5 pt-1">
        <div className="flex items-center mb-1">
          <h3 className="text-[16px] font-semibold text-[var(--color-text-main,#111827)] truncate">
            {agent.name}
          </h3>
        </div>
        <p className="text-[14px] text-[var(--color-text-main,#4b5563)] leading-[1.4] line-clamp-2 mb-1.5">
          {agent.description}
        </p>
        {meta && (
          <p className="text-[12px] text-[var(--color-text-sub,#9ca3af)] truncate">{meta}</p>
        )}
      </div>

      <div className="shrink-0 ml-2 self-center flex items-center h-[52px]">
        <div className="w-7 h-7 rounded-full bg-[var(--color-primary-blue,#2b5ce7)] flex items-center justify-center text-white">
          <ArrowRight className="w-4 h-4" strokeWidth={2.5} />
        </div>
      </div>
    </div>
  );
};
