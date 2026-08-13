import React, { useCallback, useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import {
  ArrowLeft,
  Camera,
  ChevronRight,
  X,
  Plus,
  Brain,
  SlidersHorizontal,
  Zap,
  Check,
} from 'lucide-react';
import { cn } from '@sdkwork/agents-h5-commons';
import {
  agentService,
  loadMobileModelCatalog,
  type AgentConfig,
  type ModelCatalogItem,
} from '../services/AgentService';
import { createDefaultAvatar } from '../services/DefaultAvatarService';
import { toast } from '../components/Toast';
import { t } from '../copy/mobileAgentTexts';

export interface CreateAgentMobileViewProps {
  /** Agent id to edit; when omitted the view creates a new agent. */
  initialAgentId?: string;
  onBack?: () => void;
  onCreated?: (agent: AgentConfig) => void;
  onUpdated?: (agent: AgentConfig) => void;
  notify?: (message: string, type?: 'info' | 'success' | 'error') => void;
}

const MAX_PROMPT_LENGTH = 32768;
const MAX_DESCRIPTION_LENGTH = 4096;
const MAX_NAME_LENGTH = 255;

const PRESET_TEMPERATURE = 0.7;

export const CreateAgentMobileView: React.FC<CreateAgentMobileViewProps> = ({
  initialAgentId,
  onBack,
  onCreated,
  onUpdated,
  notify = toast,
}) => {
  const isEdit = Boolean(initialAgentId);

  const [loading, setLoading] = useState(isEdit);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [persona, setPersona] = useState('');
  const [avatar, setAvatar] = useState('');
  const [model, setModel] = useState('');
  const [modelLabel, setModelLabel] = useState('');
  const [temperature, setTemperature] = useState(PRESET_TEMPERATURE);
  const [memoryEnabled, setMemoryEnabled] = useState(true);
  const [welcomeMessage, setWelcomeMessage] = useState('');
  const [suggestedPrompts, setSuggestedPrompts] = useState<string[]>([]);
  const [promptDraft, setPromptDraft] = useState('');

  const [modelCatalog, setModelCatalog] = useState<ModelCatalogItem[]>([]);
  const [showModelSheet, setShowModelSheet] = useState(false);
  const [customModel, setCustomModel] = useState('');

  const [saving, setSaving] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadEditAgent = useCallback(async () => {
    if (!initialAgentId) return;
    try {
      const agent = await agentService.getAgent(initialAgentId);
      if (!agent) {
        notify(t('agents.mobile.form.toast.loadFailed'), 'error');
        onBack?.();
        return;
      }
      setName(agent.name);
      setDescription(agent.description ?? '');
      setPersona(agent.systemPrompt ?? '');
      setAvatar(agent.avatar ?? '');
      setModel(agent.model ?? '');
      setModelLabel(agent.model ?? '');
      setTemperature(agent.temperature ?? PRESET_TEMPERATURE);
      setMemoryEnabled(agent.memoryEnabled ?? true);
      setWelcomeMessage(agent.welcomeMessage ?? '');
      setSuggestedPrompts(agent.suggestedPrompts ?? []);
    } catch (error) {
      console.error('Failed to load agent for edit', error);
      notify(t('agents.mobile.form.toast.loadFailed'), 'error');
      onBack?.();
    } finally {
      setLoading(false);
    }
  }, [initialAgentId, notify, onBack]);

  useEffect(() => {
    if (isEdit) {
      void loadEditAgent();
    }
  }, [isEdit, loadEditAgent]);

  useEffect(() => {
    let cancelled = false;
    void loadMobileModelCatalog()
      .then((catalog) => {
        if (cancelled) return;
        setModelCatalog(catalog);
        if (!modelLabel && catalog.length > 0) {
          const preferred =
            catalog.find((item) => item.defaultForEngine) ??
            catalog.find((item) => item.id === 'gpt-4') ??
            catalog[0];
          setModel(preferred.id);
          setModelLabel(preferred.label);
        }
      })
      .catch((error) => {
        console.error('Failed to load model catalog', error);
        if (!cancelled) {
          notify(t('agents.mobile.form.model.loadFailed'), 'error');
        }
      });
    return () => {
      cancelled = true;
    };
    // `modelLabel` seeding must run once; eslint-disable is intentional.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [notify]);

  const handleAvatarSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const url = URL.createObjectURL(file);
    setAvatar(url);
    notify(t('agents.mobile.form.avatar.uploaded'), 'success');
  };

  const addSuggestedPrompt = () => {
    const value = promptDraft.trim();
    if (!value) return;
    setSuggestedPrompts((prev) => (prev.includes(value) ? prev : [...prev, value]));
    setPromptDraft('');
  };

  const removeSuggestedPrompt = (index: number) => {
    setSuggestedPrompts((prev) => prev.filter((_, itemIndex) => itemIndex !== index));
  };

  const buildConfig = (): AgentConfig => ({
    name: name.trim(),
    description: description.trim(),
    avatar: avatar || undefined,
    type: 'normal',
    systemPrompt: persona.trim() || undefined,
    model: model || undefined,
    temperature,
    memoryEnabled,
    welcomeMessage: welcomeMessage.trim() || undefined,
    suggestedPrompts: suggestedPrompts.length > 0 ? suggestedPrompts : undefined,
  });

  const validate = (): boolean => {
    if (!name.trim()) {
      notify(t('agents.mobile.form.name.required'), 'error');
      return false;
    }
    if (persona.length > MAX_PROMPT_LENGTH) {
      notify(t('agents.mobile.form.toast.promptTooLong'), 'error');
      return false;
    }
    return true;
  };

  const handleSave = async (publish: boolean) => {
    if (saving || !validate()) return;
    setSaving(true);
    try {
      const config = buildConfig();
      let agent: AgentConfig;
      if (isEdit && initialAgentId) {
        agent = await agentService.updateAgent(initialAgentId, config);
      } else {
        agent = await agentService.createAgent(config);
      }
      if (publish && agent.id) {
        try {
          await agentService.publishAgent(agent.id);
          notify(t('agents.mobile.form.toast.published'), 'success');
        } catch (error) {
          console.error('Failed to publish agent', error);
          notify(t('agents.mobile.form.toast.publishFailed'), 'error');
        }
      } else {
        notify(
          isEdit
            ? t('agents.mobile.form.toast.updated')
            : t('agents.mobile.form.toast.created'),
          'success',
        );
      }
      if (isEdit) {
        onUpdated?.(agent);
      } else {
        onCreated?.(agent);
      }
      onBack?.();
    } catch (error) {
      console.error('Failed to save agent', error);
      notify(
        isEdit
          ? t('agents.mobile.form.toast.updateFailed')
          : t('agents.mobile.form.toast.createFailed'),
        'error',
      );
      setSaving(false);
    }
  };

  return (
    <div className="flex flex-col h-full bg-[var(--color-bg-color,#f5f5f7)] overflow-hidden">
      {/* Header */}
      <header className="h-[56px] shrink-0 flex items-center justify-between px-2 bg-[var(--color-glass-bg,#f5f5f7)] backdrop-blur-xl border-b border-[var(--color-border-color,rgba(0,0,0,0.05))] dark:border-[var(--color-border-color,rgba(255,255,255,0.05))] z-10">
        <div className="flex items-center flex-1">
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
          {isEdit ? t('agents.mobile.form.title.edit') : t('agents.mobile.form.title.create')}
        </h1>
        <div className="flex items-center flex-1 justify-end gap-2 pr-1">
          <button
            type="button"
            disabled={saving}
            onClick={() => void handleSave(false)}
            className={cn(
              'px-3.5 py-1.5 rounded-lg text-[14px] font-medium transition-colors',
              saving
                ? 'bg-black/5 dark:bg-white/10 text-gray-400 cursor-not-allowed'
                : 'bg-[var(--color-primary-blue,#2b5ce7)] text-white active:bg-[#2452cc]',
            )}
          >
            {saving ? t('agents.mobile.form.saving') : t('agents.mobile.form.save')}
          </button>
          <button
            type="button"
            disabled={saving}
            onClick={() => void handleSave(true)}
            className={cn(
              'px-3.5 py-1.5 rounded-lg text-[14px] font-medium transition-colors border',
              saving
                ? 'border-black/10 dark:border-white/10 text-gray-400 cursor-not-allowed'
                : 'border-[#2b5ce7]/30 text-[var(--color-primary-blue,#2b5ce7)] dark:text-[#6f9bff] active:bg-[var(--color-primary-blue,#2b5ce7)]/10',
            )}
          >
            {saving ? t('agents.mobile.form.publishing') : t('agents.mobile.form.publish')}
          </button>
        </div>
      </header>

      {/* Content */}
      {loading ? (
        <div className="flex-1 px-4 pt-4 space-y-3 animate-pulse" aria-busy="true">
          <div className="h-28 rounded-2xl bg-[var(--color-chat-other-bg,#262626)]" />
          <div className="h-40 rounded-2xl bg-[var(--color-chat-other-bg,#262626)]" />
          <div className="h-48 rounded-2xl bg-[var(--color-chat-other-bg,#262626)]" />
        </div>
      ) : (
        <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4 pb-8">
          {/* Basic info */}
          <section className="rounded-2xl bg-[var(--color-chat-other-bg,#262626)] px-4 py-4 space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-[13px] font-medium text-[var(--color-text-sub,#6b7280)]">
                {t('agents.mobile.form.avatar')}
              </span>
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="relative w-16 h-16 rounded-2xl overflow-hidden bg-black/5 dark:bg-white/10 border border-black/5 dark:border-white/10"
              >
                <img
                  src={avatar || createDefaultAvatar('agent')}
                  alt={t('agents.mobile.form.avatar')}
                  className="w-full h-full object-cover"
                />
                <span className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 active:opacity-100 transition-opacity">
                  <Camera className="w-5 h-5 text-white" />
                </span>
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept="image/*"
                className="hidden"
                onChange={handleAvatarSelect}
              />
            </div>

            <FormField label={t('agents.mobile.form.name')}>
              <input
                value={name}
                onChange={(event) => setName(event.target.value)}
                maxLength={MAX_NAME_LENGTH}
                placeholder={t('agents.mobile.form.name.placeholder')}
                className={inputClassName}
              />
            </FormField>

            <FormField label={t('agents.mobile.form.description')}>
              <textarea
                value={description}
                onChange={(event) => setDescription(event.target.value)}
                maxLength={MAX_DESCRIPTION_LENGTH}
                rows={2}
                placeholder={t('agents.mobile.form.description.placeholder')}
                className={cn(inputClassName, 'resize-none leading-relaxed')}
              />
            </FormField>
          </section>

          {/* Persona */}
          <section className="rounded-2xl bg-[var(--color-chat-other-bg,#262626)] px-4 py-4">
            <FormField label={t('agents.mobile.form.persona')}>
              <textarea
                value={persona}
                onChange={(event) => setPersona(event.target.value)}
                maxLength={MAX_PROMPT_LENGTH}
                rows={5}
                placeholder={t('agents.mobile.form.persona.placeholder')}
                className={cn(inputClassName, 'resize-none leading-relaxed')}
              />
              <span className="mt-1 block text-right text-[11px] text-[var(--color-text-sub,#9ca3af)]">
                {persona.length}/{MAX_PROMPT_LENGTH}
              </span>
            </FormField>
          </section>

          {/* Config */}
          <section className="rounded-2xl bg-[var(--color-chat-other-bg,#262626)] px-4 py-1 divide-y divide-[var(--color-border-color,rgba(0,0,0,0.05))] dark:divide-[var(--color-border-color,rgba(255,255,255,0.05))]">
            <SettingRow
              icon={<SlidersHorizontal className="w-[18px] h-[18px] text-[var(--color-primary-blue,#2b5ce7)]" />}
              label={t('agents.mobile.form.model')}
              value={modelLabel || t('agents.mobile.form.model.placeholder')}
              onClick={() => setShowModelSheet(true)}
            />

            <div className="py-3.5 flex items-center justify-between gap-3">
              <div className="flex items-center gap-3 min-w-0">
                <SlidersHorizontal className="w-[18px] h-[18px] text-[var(--color-primary-blue,#2b5ce7)] shrink-0" />
                <div className="min-w-0">
                  <div className="text-[15px] text-[var(--color-text-main,#111827)]">
                    {t('agents.mobile.form.temperature')}
                    <span className="ml-1.5 text-[13px] text-[var(--color-text-sub,#9ca3af)]">
                      {temperature.toFixed(1)}
                    </span>
                  </div>
                  <p className="text-[11px] text-[var(--color-text-sub,#9ca3af)]">
                    {t('agents.mobile.form.temperature.desc')}
                  </p>
                </div>
              </div>
              <input
                type="range"
                min={0}
                max={2}
                step={0.1}
                value={temperature}
                onChange={(event) => setTemperature(Number(event.target.value))}
                className="w-28 accent-[#2b5ce7]"
                aria-label={t('agents.mobile.form.temperature')}
              />
            </div>

            <SettingRow
              icon={<Brain className="w-[18px] h-[18px] text-[var(--color-primary-blue,#2b5ce7)]" />}
              label={t('agents.mobile.form.memory')}
              description={t('agents.mobile.form.memory.desc')}
              trailing={
                <button
                  type="button"
                  role="switch"
                  aria-checked={memoryEnabled}
                  onClick={() => setMemoryEnabled((prev) => !prev)}
                  className={cn(
                    'relative w-11 h-6 rounded-full transition-colors shrink-0',
                    memoryEnabled ? 'bg-[var(--color-primary-blue,#2b5ce7)]' : 'bg-black/15 dark:bg-white/20',
                  )}
                >
                  <span
                    className={cn(
                      'absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-all',
                      memoryEnabled ? 'left-[22px]' : 'left-0.5',
                    )}
                  />
                </button>
              }
            />
          </section>

          {/* Experience */}
          <section className="rounded-2xl bg-[var(--color-chat-other-bg,#262626)] px-4 py-4 space-y-4">
            <FormField label={t('agents.mobile.form.welcome')}>
              <input
                value={welcomeMessage}
                onChange={(event) => setWelcomeMessage(event.target.value)}
                maxLength={200}
                placeholder={t('agents.mobile.form.welcome.placeholder')}
                className={inputClassName}
              />
            </FormField>

            <FormField label={t('agents.mobile.form.suggestedPrompts')}>
              {suggestedPrompts.length > 0 && (
                <div className="flex flex-wrap gap-2 mb-2.5">
                  {suggestedPrompts.map((prompt, index) => (
                    <span
                      key={`${prompt}-${index}`}
                      className="inline-flex items-center gap-1 rounded-full bg-[var(--color-primary-blue,#2b5ce7)]/10 text-[var(--color-primary-blue,#2b5ce7)] dark:text-[#6f9bff] px-3 py-1 text-[13px]"
                    >
                      {prompt}
                      <button
                        type="button"
                        aria-label="remove"
                        onClick={() => removeSuggestedPrompt(index)}
                        className="opacity-60 active:opacity-100"
                      >
                        <X className="w-3.5 h-3.5" />
                      </button>
                    </span>
                  ))}
                </div>
              )}
              <div className="flex items-center gap-2">
                <input
                  value={promptDraft}
                  onChange={(event) => setPromptDraft(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      event.preventDefault();
                      addSuggestedPrompt();
                    }
                  }}
                  maxLength={80}
                  placeholder={t('agents.mobile.form.suggestedPrompts.placeholder')}
                  className={cn(inputClassName, 'flex-1')}
                />
                <button
                  type="button"
                  onClick={addSuggestedPrompt}
                  aria-label={t('agents.mobile.form.suggestedPrompts.add')}
                  className="shrink-0 w-9 h-9 flex items-center justify-center rounded-full bg-[var(--color-primary-blue,#2b5ce7)]/10 text-[var(--color-primary-blue,#2b5ce7)] dark:text-[#6f9bff] active:bg-[var(--color-primary-blue,#2b5ce7)]/20 transition-colors"
                >
                  <Plus className="w-4.5 h-4.5" />
                </button>
              </div>
            </FormField>
          </section>
        </div>
      )}

      {/* Model picker bottom sheet */}
      <AnimatePresence>
        {showModelSheet && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15 }}
            className="fixed inset-0 z-50 bg-black/50 flex items-end justify-center"
            onClick={() => setShowModelSheet(false)}
          >
            <motion.div
              initial={{ y: '100%' }}
              animate={{ y: 0 }}
              exit={{ y: '100%' }}
              transition={{ type: 'spring', damping: 28, stiffness: 320 }}
              className="w-full max-w-[420px] rounded-t-2xl bg-[var(--color-chat-other-bg,#262626)] max-h-[70vh] flex flex-col pb-[env(safe-area-inset-bottom)]"
              onClick={(event) => event.stopPropagation()}
            >
              <div className="flex justify-center pt-2.5 pb-1 shrink-0">
                <div className="h-1 w-9 rounded-full bg-black/15 dark:bg-white/20" />
              </div>
              <h3 className="px-5 pb-3 pt-1 text-[16px] font-semibold text-[var(--color-text-main,#111827)] shrink-0">
                {t('agents.mobile.form.model')}
              </h3>
              <div className="flex-1 overflow-y-auto px-3 pb-3 space-y-1">
                {modelCatalog.map((item) => {
                  const selected = item.id === model;
                  return (
                    <button
                      key={item.id}
                      type="button"
                      onClick={() => {
                        setModel(item.id);
                        setModelLabel(item.label);
                        setShowModelSheet(false);
                      }}
                      className={cn(
                        'w-full flex items-center gap-3 rounded-xl px-3 py-3 text-left transition-colors',
                        selected
                          ? 'bg-[var(--color-primary-blue,#2b5ce7)]/10'
                          : 'active:bg-black/5 dark:active:bg-white/10',
                      )}
                    >
                      <div className="flex-1 min-w-0">
                        <div className="text-[15px] font-medium text-[var(--color-text-main,#111827)] truncate">
                          {item.label}
                        </div>
                        {item.description && (
                          <p className="text-[12px] text-[var(--color-text-sub,#6b7280)] truncate">
                            {item.description}
                          </p>
                        )}
                      </div>
                      {selected && <Check className="w-4.5 h-4.5 text-[var(--color-primary-blue,#2b5ce7)] shrink-0" />}
                    </button>
                  );
                })}
                {modelCatalog.length === 0 && (
                  <p className="px-3 py-4 text-center text-[13px] text-[var(--color-text-sub,#9ca3af)]">
                    {t('agents.mobile.form.model.loadFailed')}
                  </p>
                )}
                <div className="flex items-center gap-2 px-1 pt-2">
                  <Zap className="w-4 h-4 text-gray-400 shrink-0" />
                  <input
                    value={customModel}
                    onChange={(event) => setCustomModel(event.target.value)}
                    onKeyDown={(event) => {
                      if (event.key === 'Enter' && customModel.trim()) {
                        setModel(customModel.trim());
                        setModelLabel(customModel.trim());
                        setShowModelSheet(false);
                      }
                    }}
                    placeholder={t('agents.mobile.form.model.custom')}
                    className={cn(inputClassName, 'flex-1')}
                  />
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

const inputClassName = cn(
  'w-full rounded-xl bg-black/5 dark:bg-white/10 px-3.5 py-2.5 text-[15px] text-[var(--color-text-main,#111827)] placeholder:text-gray-400 dark:placeholder:text-gray-500 outline-none focus:ring-2 focus:ring-[#2b5ce7]/30 transition-shadow',
);

const FormField: React.FC<{
  label: string;
  children: React.ReactNode;
}> = ({ label, children }) => (
  <label className="block">
    <span className="mb-1.5 block text-[13px] font-medium text-[var(--color-text-sub,#6b7280)]">
      {label}
    </span>
    {children}
  </label>
);

const SettingRow: React.FC<{
  icon: React.ReactNode;
  label: string;
  description?: string;
  value?: string;
  trailing?: React.ReactNode;
  onClick?: () => void;
}> = ({ icon, label, description, value, trailing, onClick }) => (
  <div
    role={onClick ? 'button' : undefined}
    tabIndex={onClick ? 0 : undefined}
    onClick={onClick}
    onKeyDown={(event) => {
      if (onClick && event.key === 'Enter') onClick();
    }}
    className="py-3.5 flex items-center gap-3 min-w-0"
  >
    <span className="shrink-0">{icon}</span>
    <div className="flex-1 min-w-0">
      <div className="text-[15px] text-[var(--color-text-main,#111827)]">{label}</div>
      {description && (
        <p className="text-[11px] text-[var(--color-text-sub,#9ca3af)]">{description}</p>
      )}
    </div>
    {value !== undefined && (
      <span className="shrink-0 text-[14px] text-[var(--color-text-sub,#9ca3af)] truncate max-w-[140px]">
        {value}
      </span>
    )}
    {trailing}
    {onClick && <ChevronRight className="w-4 h-4 text-gray-300 dark:text-gray-600 shrink-0" />}
  </div>
);
