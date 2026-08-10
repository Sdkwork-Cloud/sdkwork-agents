import React from 'react';
import type { Character } from '../services/CharacterService';

export interface CharacterCardProps {
  character: Character;
  onClick?: () => void;
  /** Gesture handlers spread onto the row (long-press, context menu). */
  onLongPressProps?: React.DOMAttributes<HTMLDivElement>;
}

export const CharacterCard: React.FC<CharacterCardProps> = ({
  character,
  onClick,
  onLongPressProps,
}) => {
  return (
    <div
      className="bg-[var(--color-chat-other-bg,#262626)] px-4 py-3.5 flex items-center gap-4 border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] last:border-b-0 active:bg-black/5 dark:active:bg-white/5 transition-colors cursor-pointer select-none touch-callout-none"
      onClick={onClick}
      {...onLongPressProps}
    >
      <img
        src={character.avatar}
        className="w-12 h-12 rounded-full object-cover shrink-0 border border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] pointer-events-none"
        alt={character.name}
      />
      <div className="flex-1 min-w-0 pointer-events-none">
        <h3 className="text-[16px] font-medium text-[var(--color-text-main,#111827)] truncate">
          {character.name}
        </h3>
        <p className="text-[13px] text-[var(--color-text-sub,#6b7280)] truncate mt-0.5">
          {character.desc}
        </p>
      </div>
    </div>
  );
};
