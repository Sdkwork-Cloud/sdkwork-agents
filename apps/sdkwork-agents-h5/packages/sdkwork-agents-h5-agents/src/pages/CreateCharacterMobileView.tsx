import React, { useEffect, useState } from 'react';
import { ArrowLeft, Mic, ChevronRight } from 'lucide-react';
import { characterService, type Character } from '../services/CharacterService';
import { SelectVoiceModal } from '../components/SelectVoiceModal';
import { toast } from '../components/Toast';
import { t } from '../copy/mobileAgentTexts';

export interface CreateCharacterMobileViewProps {
  /** Character id to edit; when omitted the view creates a new character. */
  initialCharacterId?: string;
  onBack?: () => void;
  /** Host-navigated callback after a successful create/update. */
  onSaved?: (character: Character) => void;
  /** Host toast port; defaults to the built-in agents toast. */
  notify?: (message: string, type?: 'info' | 'success' | 'error') => void;
}

export const CreateCharacterMobileView: React.FC<CreateCharacterMobileViewProps> = ({
  initialCharacterId,
  onBack,
  onSaved,
  notify = toast,
}) => {
  const isEdit = Boolean(initialCharacterId);

  const [name, setName] = useState('');
  const [desc, setDesc] = useState('');
  const [voice, setVoice] = useState('');
  const [voiceLabel, setVoiceLabel] = useState('');
  const [showVoiceSelector, setShowVoiceSelector] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!initialCharacterId) return;
    void characterService.getCharacters().then((chars) => {
      const char = chars.find((c) => c.id === initialCharacterId);
      if (!char) return;
      setName(char.name);
      setDesc(char.desc);
      if (char.voice) setVoice(char.voice);
    });
  }, [initialCharacterId]);

  const handleSave = async () => {
    if (!name.trim()) {
      notify(t('agents.mobile.characters.form.name.required'), 'error');
      return;
    }
    setSaving(true);
    try {
      const baseData = { name, desc, voice: voice || undefined };
      if (isEdit && initialCharacterId) {
        const updated = await characterService.editCharacter(initialCharacterId, baseData);
        notify(t('agents.mobile.characters.form.toast.updated'), 'success');
        onSaved?.(updated);
      } else {
        const created = await characterService.addCharacter(baseData);
        notify(t('agents.mobile.characters.form.toast.created'), 'success');
        onSaved?.(created);
      }
    } catch (error) {
      console.error('Failed to save character', error);
      notify(t('agents.mobile.form.toast.createFailed'), 'error');
    } finally {
      setSaving(false);
    }
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
          {isEdit
            ? t('agents.mobile.characters.form.title.edit')
            : t('agents.mobile.characters.form.title.create')}
        </h1>
        <div className="flex-1" />
      </header>

      <div className="flex-1 overflow-y-auto p-4 flex flex-col gap-6">
        <div className="bg-[var(--color-chat-other-bg,#262626)] rounded-2xl border border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] px-4 py-2 flex flex-col">
          <div className="flex items-center gap-4 py-3 border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))]">
            <span className="w-16 whitespace-nowrap text-[16px] text-[var(--color-text-main,#111827)]">
              {t('agents.mobile.characters.form.name')}
            </span>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t('agents.mobile.characters.form.name.placeholder')}
              className="flex-1 bg-transparent text-[16px] text-[var(--color-text-main,#111827)] outline-none placeholder:text-gray-400 dark:placeholder:text-gray-500"
            />
          </div>

          <div
            className="flex items-center gap-4 py-3 cursor-pointer active:opacity-70 transition-opacity"
            onClick={() => setShowVoiceSelector(true)}
          >
            <span className="w-16 whitespace-nowrap text-[16px] text-[var(--color-text-main,#111827)]">
              {t('agents.mobile.characters.form.voice')}
            </span>
            <div className="flex-1 flex justify-between items-center text-[16px]">
              <span
                className={
                  voice ? 'text-[var(--color-primary-blue,#2b5ce7)] font-medium' : 'text-[var(--color-text-sub,#9ca3af)]'
                }
              >
                {voiceLabel || voice || t('agents.mobile.characters.form.voice.placeholder')}
              </span>
              <div className="flex items-center gap-1 text-[var(--color-text-sub,#9ca3af)]">
                <Mic className="w-4 h-4" />
                <ChevronRight className="w-4 h-4 ml-1 opacity-50" />
              </div>
            </div>
          </div>
        </div>

        <div className="flex flex-col gap-2">
          <label className="text-[14px] font-medium text-[var(--color-text-main,#111827)] ml-2">
            {t('agents.mobile.characters.form.desc.label')}
          </label>
          <div className="bg-[var(--color-chat-other-bg,#262626)] border border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] rounded-2xl p-4">
            <textarea
              value={desc}
              onChange={(e) => setDesc(e.target.value)}
              placeholder={t('agents.mobile.characters.form.desc.placeholder')}
              rows={4}
              className="w-full bg-transparent text-[15px] text-[var(--color-text-main,#111827)] outline-none resize-none placeholder:text-gray-400 dark:placeholder:text-gray-500"
            />
          </div>
        </div>
      </div>

      <div className="px-4 pt-2 pb-[calc(env(safe-area-inset-bottom,0px)+1rem)] shrink-0">
        <button
          type="button"
          onClick={() => void handleSave()}
          disabled={saving}
          className="w-full py-3.5 bg-[var(--color-primary-blue,#2b5ce7)] text-white rounded-full font-bold text-[16px] shadow-lg shadow-[#2b5ce7]/20 active:opacity-80 transition-opacity disabled:opacity-50"
        >
          {t('agents.mobile.characters.form.save')}
        </button>
      </div>

      <SelectVoiceModal
        isOpen={showVoiceSelector}
        onClose={() => setShowVoiceSelector(false)}
        selectedVoices={voice ? [voice] : []}
        isMulti={false}
        onSave={(voiceIds, selectedItems) => {
          setVoice(voiceIds[0] ?? '');
          setVoiceLabel(selectedItems[0]?.name ?? '');
          setShowVoiceSelector(false);
        }}
      />
    </div>
  );
};
