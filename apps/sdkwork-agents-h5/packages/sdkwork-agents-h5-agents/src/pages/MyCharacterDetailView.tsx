import React, { useEffect, useState } from 'react';
import { ArrowLeft, MessageSquare, Settings2, ChevronRight } from 'lucide-react';
import { characterService, type Character } from '../services/CharacterService';
import { toast } from '../components/Toast';
import { t } from '../i18n/mobileAgentTexts';

export interface MyCharacterDetailViewProps {
  /** Character id resolved from the host route params. */
  characterId?: string;
  onBack?: () => void;
  /** Host-navigated chat entry with the character. */
  onStartChat?: (character: Character) => void;
  /** Host-navigated edit entry (route with character id). */
  onEdit?: (character: Character) => void;
  /** Host toast port; defaults to the built-in agents toast. */
  notify?: (message: string, type?: 'info' | 'success' | 'error') => void;
}

export const MyCharacterDetailView: React.FC<MyCharacterDetailViewProps> = ({
  characterId,
  onBack,
  onStartChat,
  onEdit,
  notify = toast,
}) => {
  const [character, setCharacter] = useState<Character | null>(null);

  useEffect(() => {
    if (!characterId) return;
    void characterService.getCharacters().then((chars) => {
      const found = chars.find((c) => c.id === characterId);
      if (found) {
        setCharacter(found);
      } else {
        notify(t('agents.mobile.form.toast.loadFailed'), 'error');
        onBack?.();
      }
    });
  }, [characterId, notify, onBack]);

  if (!character) return null;

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg-color,#f5f5f7)] overflow-hidden">
      {/* Header */}
      <header className="h-[56px] shrink-0 flex items-center justify-between px-2 bg-[var(--color-glass-bg,#f5f5f7)] backdrop-blur-xl border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] z-10">
        <div className="flex items-center flex-1 gap-1">
          {onBack && (
            <button
              type="button"
              aria-label={t('agents.mobile.back')}
              onClick={onBack}
              className="w-10 h-10 flex items-center justify-center rounded-full active:bg-black/5 dark:active:bg-white/10 transition-colors"
            >
              <ArrowLeft className="w-6 h-6 text-[var(--color-text-main,#111827)]" />
            </button>
          )}
        </div>
        <h1 className="absolute left-1/2 -translate-x-1/2 text-[17px] font-semibold text-[var(--color-text-main,#111827)]">
          {t('agents.mobile.characters.detail.title')}
        </h1>
        <div className="flex items-center flex-1 justify-end">
          <button
            type="button"
            aria-label={t('agents.mobile.characters.menu.edit')}
            className="w-10 h-10 flex items-center justify-center rounded-full text-[var(--color-text-sub,#9ca3af)] active:bg-black/5 dark:active:bg-white/10"
          >
            <Settings2 className="w-6 h-6" />
          </button>
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-y-auto w-full flex flex-col items-center">
        {/* Profile Card */}
        <div className="w-full flex flex-col items-center justify-center py-10 px-6 bg-[var(--color-chat-other-bg,#262626)] border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))]">
          <div className="w-24 h-24 mb-4 rounded-full overflow-hidden shadow-sm border border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))]">
            <img src={character.avatar} alt={character.name} className="w-full h-full object-cover" />
          </div>

          <h2 className="text-[20px] font-bold text-[var(--color-text-main,#111827)] mb-1">
            {character.name}
          </h2>
          <p className="text-[14px] text-[var(--color-text-sub,#6b7280)] mb-6">
            {t('agents.mobile.characters.detail.visibility.private')}
          </p>

          <p className="text-[15px] leading-relaxed text-gray-900/90 dark:text-gray-100/90 text-center px-4 line-clamp-3">
            {character.desc}
          </p>
        </div>

        {/* Info */}
        <div className="w-full mt-2 bg-[var(--color-chat-other-bg,#262626)] py-2">
          <div className="flex flex-col px-4 py-3 border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] last:border-0 transition-colors">
            <span className="text-[16px] text-[var(--color-text-main,#111827)] mb-1">
              {t('agents.mobile.characters.detail.systemPrompt')}
            </span>
            <span className="text-[14px] text-[var(--color-text-sub,#6b7280)] line-clamp-2">
              {character.prompt || t('agents.mobile.characters.detail.systemPrompt.empty')}
            </span>
          </div>
          <div className="flex items-center px-4 py-4 border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] last:border-0 transition-colors">
            <span className="text-[16px] text-[var(--color-text-main,#111827)] flex-1">
              {t('agents.mobile.characters.detail.voice')}
            </span>
            <span className="text-[15px] text-[var(--color-text-sub,#6b7280)] flex items-center gap-2">
              {t('agents.mobile.characters.detail.voice.default')}
              <ChevronRight className="w-4 h-4" />
            </span>
          </div>
        </div>

        <div className="w-full mt-6 px-4 pb-8 flex flex-col gap-3">
          <button
            type="button"
            onClick={() => onStartChat?.(character)}
            className="w-full flex items-center justify-center gap-2 py-3.5 bg-[var(--color-primary-blue,#2b5ce7)] text-white rounded-full font-bold active:opacity-80 transition-opacity shadow-md"
          >
            <MessageSquare className="w-5 h-5 text-current" />
            <span>{t('agents.mobile.characters.detail.chat')}</span>
          </button>
          <button
            type="button"
            onClick={() => onEdit?.(character)}
            className="w-full py-3 bg-[var(--color-chat-other-bg,#262626)] border border-black/10 dark:border-white/10 text-[var(--color-text-main,#111827)] rounded-full font-medium active:bg-black/5 dark:active:bg-white/5 transition-colors"
          >
            {t('agents.mobile.characters.detail.edit')}
          </button>
        </div>
      </div>
    </div>
  );
};
