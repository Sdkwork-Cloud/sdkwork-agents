import React, { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowLeft, Plus, UserRound } from 'lucide-react';
import { characterService, type Character } from '../services/CharacterService';
import { CharacterCard } from '../components/CharacterCard';
import { MobileActionSheet, MobileConfirmDialog } from '../components/MobileSheets';
import { toast } from '../components/Toast';
import { t } from '../copy/mobileAgentTexts';

export interface MyCharactersViewProps {
  /** Host-navigated back; when omitted the back chevron is hidden. */
  onBack?: () => void;
  /** Host-navigated create entry; when omitted the header "+" is hidden. */
  onCreateCharacter?: () => void;
  /** Host-navigated edit entry (route with character id). */
  onEditCharacter?: (id: string) => void;
  /** Host-navigated detail entry (route with character id). */
  onViewDetail?: (id: string) => void;
  /** Host toast port; defaults to the built-in agents toast. */
  notify?: (message: string, type?: 'info' | 'success' | 'error') => void;
}

export const MyCharactersView: React.FC<MyCharactersViewProps> = ({
  onBack,
  onCreateCharacter,
  onEditCharacter,
  onViewDetail,
  notify = toast,
}) => {
  const [characters, setCharacters] = useState<Character[]>([]);
  const [actionSheetItem, setActionSheetItem] = useState<Character | null>(null);
  const [confirmItem, setConfirmItem] = useState<Character | null>(null);
  const [isLongPressed, setIsLongPressed] = useState(false);

  const loadCharacters = useCallback(async () => {
    setCharacters(await characterService.getCharacters());
  }, []);

  useEffect(() => {
    void loadCharacters();
  }, [loadCharacters]);

  // Long-press to open the action sheet (touch + pointer + context menu).
  const longPressTimer = useRef<number | null>(null);
  const startLongPress = (char: Character) => {
    const handlePressStart = () => {
      setIsLongPressed(false);
      longPressTimer.current = window.setTimeout(() => {
        setIsLongPressed(true);
        setActionSheetItem(char);
      }, 500);
    };

    const handlePressEnd = () => {
      if (longPressTimer.current !== null) {
        window.clearTimeout(longPressTimer.current);
        longPressTimer.current = null;
      }
    };

    return {
      onPointerDown: handlePressStart,
      onPointerUp: handlePressEnd,
      onPointerLeave: () => {
        handlePressEnd();
        setIsLongPressed(false);
      },
      onContextMenu: (e: React.MouseEvent) => {
        e.preventDefault();
        handlePressStart();
        setIsLongPressed(true);
        setActionSheetItem(char);
        handlePressEnd();
      },
    };
  };

  const handleDelete = async () => {
    if (!confirmItem) return;
    await characterService.deleteCharacter(confirmItem.id);
    notify(t('agents.mobile.characters.toast.deleted'), 'success');
    setConfirmItem(null);
    void loadCharacters();
  };

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
          {t('agents.mobile.characters.title')}
        </h1>
        <div className="flex items-center flex-1 justify-end">
          {onCreateCharacter && (
            <button
              type="button"
              aria-label={t('agents.mobile.characters.create')}
              onClick={onCreateCharacter}
              className="w-10 h-10 flex items-center justify-center rounded-full active:bg-black/5 dark:active:bg-white/10 transition-colors"
            >
              <Plus className="w-6 h-6 text-[var(--color-text-main,#111827)]" />
            </button>
          )}
        </div>
      </header>

      {/* Content */}
      <div className="flex-1 overflow-y-auto overscroll-contain">
        {characters.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full px-8">
            <div className="w-20 h-20 rounded-[24px] bg-[var(--color-chat-other-bg,#262626)] flex items-center justify-center mb-5 shadow-sm">
              <UserRound className="w-10 h-10 text-[var(--color-primary-blue,#2b5ce7)]" />
            </div>
            <h3 className="text-[17px] font-semibold text-[var(--color-text-main,#111827)] mb-2">
              {t('agents.mobile.characters.title')}
            </h3>
            <p className="text-[13px] text-[var(--color-text-sub,#6b7280)] mb-8 max-w-[240px] text-center leading-relaxed">
              {t('agents.mobile.characters.empty')}
            </p>
            {onCreateCharacter && (
              <button
                type="button"
                onClick={onCreateCharacter}
                className="px-8 h-12 rounded-full bg-[var(--color-primary-blue,#2b5ce7)] text-white text-[15px] font-medium active:scale-95 transition-transform shadow-lg shadow-[var(--color-primary-blue,#2b5ce7)]/25"
              >
                {t('agents.mobile.characters.create')}
              </button>
            )}
          </div>
        ) : (
          <div className="mt-2 w-full">
            <div className="flex flex-col border-y border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))]">
              {characters.map((char) => (
                <CharacterCard
                  key={char.id}
                  character={char}
                  onClick={() => {
                    if (isLongPressed) {
                      setIsLongPressed(false);
                      return;
                    }
                    onViewDetail?.(char.id);
                  }}
                  onLongPressProps={startLongPress(char)}
                />
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Action sheet */}
      <MobileActionSheet
        isOpen={actionSheetItem !== null}
        onClose={() => setActionSheetItem(null)}
        options={
          actionSheetItem
            ? [
                {
                  label: t('agents.mobile.characters.menu.edit'),
                  onClick: () => {
                    if (actionSheetItem.id && onEditCharacter) {
                      onEditCharacter(actionSheetItem.id);
                    }
                  },
                },
                {
                  label: t('agents.mobile.characters.menu.delete'),
                  danger: true,
                  onClick: () => setConfirmItem(actionSheetItem),
                },
              ]
            : []
        }
      />

      {/* Delete confirm */}
      <MobileConfirmDialog
        isOpen={confirmItem !== null}
        title={t('agents.mobile.characters.confirm.delete.title')}
        description={t('agents.mobile.characters.confirm.delete.desc', {
          name: confirmItem?.name ?? '',
        })}
        confirmText={t('agents.mobile.characters.confirm.ok')}
        cancelText={t('agents.mobile.characters.confirm.cancel')}
        danger
        onConfirm={() => void handleDelete()}
        onCancel={() => setConfirmItem(null)}
      />
    </div>
  );
};
