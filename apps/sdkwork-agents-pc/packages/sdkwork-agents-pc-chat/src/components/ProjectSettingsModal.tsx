import React, { useEffect, useState } from 'react';
import { FolderCog, Loader2, Save, Trash2, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import {
  ProjectService,
  type ChatProject,
  type ProjectSettingsData,
} from '../services/ProjectService';

interface ProjectSettingsModalProps {
  project: ChatProject;
  onClose: () => void;
  onSaved: (project: ChatProject) => void;
  onDeleted: (projectId: string) => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error && error.message.trim() ? error.message : String(error);
}

export const ProjectSettingsModal: React.FC<ProjectSettingsModalProps> = ({
  project,
  onClose,
  onSaved,
  onDeleted,
}) => {
  const { t } = useTranslation('chat');
  const [settings, setSettings] = useState<ProjectSettingsData | null>(null);
  const [name, setName] = useState(project.name);
  const [description, setDescription] = useState(project.description ?? '');
  const [visibility, setVisibility] = useState(project.visibility);
  const [driveAccessMode, setDriveAccessMode] = useState(project.driveAccessMode);
  const [instructions, setInstructions] = useState('');
  const [memorySpaceId, setMemorySpaceId] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError(null);
    ProjectService.getProjectSettings(project.projectId)
      .then((loaded) => {
        if (!active) return;
        setSettings(loaded);
        setName(loaded.project.name);
        setDescription(loaded.project.description ?? '');
        setVisibility(loaded.project.visibility);
        setDriveAccessMode(loaded.project.driveAccessMode);
        setInstructions(loaded.instructions);
        setMemorySpaceId(loaded.memorySpaceId ?? '');
      })
      .catch((cause) => active && setError(errorMessage(cause)))
      .finally(() => active && setLoading(false));
    return () => {
      active = false;
    };
  }, [project.projectId]);

  const handleVisibilityChange = (next: ChatProject['visibility']) => {
    setVisibility(next);
    if (next === 'shared' && driveAccessMode === 'owner_library') {
      setDriveAccessMode('explicit_resources');
    }
  };

  const handleSave = async () => {
    if (!settings || !name.trim()) return;
    setSaving(true);
    setError(null);
    try {
      const updatedProject = await ProjectService.updateProject(settings.project, {
        name: name.trim(),
        description: description.trim() || undefined,
        visibility,
        driveAccessMode,
      });
      let updatedSlots = await ProjectService.saveProjectInstructions(
        updatedProject,
        settings.slots,
        instructions,
      );
      updatedSlots = await ProjectService.saveProjectMemorySpace(
        updatedProject.projectId,
        updatedSlots,
        memorySpaceId || undefined,
      );
      setSettings({
        ...settings,
        project: updatedProject,
        slots: updatedSlots,
        instructions,
        memorySpaceId: memorySpaceId || undefined,
      });
      onSaved(updatedProject);
      onClose();
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!window.confirm(t('projectDeleteConfirm'))) return;
    setDeleting(true);
    setError(null);
    try {
      await ProjectService.deleteProject(project.projectId);
      onDeleted(project.projectId);
      onClose();
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setDeleting(false);
    }
  };

  const busy = loading || saving || deleting;

  return (
    <div
      className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/65 p-4"
      onMouseDown={(event) => event.target === event.currentTarget && !busy && onClose()}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="project-settings-title"
        className="flex max-h-[min(760px,calc(100vh-32px))] w-full max-w-[620px] flex-col overflow-hidden rounded-lg border border-white/10 bg-[#18181a] shadow-2xl"
      >
        <header className="flex min-h-16 items-center justify-between border-b border-white/10 px-5">
          <div className="flex min-w-0 items-center gap-3">
            <FolderCog size={20} className="shrink-0 text-zinc-400" />
            <h2 id="project-settings-title" className="truncate text-base font-semibold text-white">
              {t('projectSettings')}
            </h2>
          </div>
          <button
            type="button"
            onClick={onClose}
            disabled={busy}
            className="grid size-9 place-items-center rounded-md text-zinc-400 transition-colors hover:bg-white/10 hover:text-white disabled:opacity-40"
            aria-label={t('close')}
            title={t('close')}
          >
            <X size={19} />
          </button>
        </header>

        <div className="flex-1 overflow-y-auto px-5 py-5">
          {loading ? (
            <div className="grid min-h-64 place-items-center" aria-label={t('loading')}>
              <Loader2 size={24} className="animate-spin text-zinc-400" />
            </div>
          ) : (
            <div className="space-y-5">
              {error && (
                <div role="alert" className="rounded-md border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-300">
                  {error}
                </div>
              )}

              <label className="block space-y-2">
                <span className="text-sm font-medium text-zinc-300">{t('projectName')}</span>
                <input
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  maxLength={255}
                  disabled={busy}
                  className="h-10 w-full rounded-md border border-white/10 bg-[#101012] px-3 text-sm text-white outline-none transition-colors focus:border-emerald-500/60 disabled:opacity-50"
                />
              </label>

              <label className="block space-y-2">
                <span className="text-sm font-medium text-zinc-300">{t('projectDescription')}</span>
                <textarea
                  value={description}
                  onChange={(event) => setDescription(event.target.value)}
                  disabled={busy}
                  rows={3}
                  className="w-full resize-y rounded-md border border-white/10 bg-[#101012] px-3 py-2 text-sm text-white outline-none transition-colors focus:border-emerald-500/60 disabled:opacity-50"
                />
              </label>

              <div className="grid gap-4 sm:grid-cols-2">
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-zinc-300">{t('projectVisibility')}</span>
                  <select
                    value={visibility}
                    onChange={(event) => handleVisibilityChange(event.target.value as ChatProject['visibility'])}
                    disabled={busy}
                    className="h-10 w-full rounded-md border border-white/10 bg-[#101012] px-3 text-sm text-white outline-none focus:border-emerald-500/60 disabled:opacity-50"
                  >
                    <option value="private">{t('projectVisibilityPrivate')}</option>
                    <option value="organization">{t('projectVisibilityOrganization')}</option>
                    <option value="shared">{t('projectVisibilityShared')}</option>
                  </select>
                </label>

                <label className="block space-y-2">
                  <span className="text-sm font-medium text-zinc-300">{t('projectDriveAccess')}</span>
                  <select
                    value={driveAccessMode}
                    onChange={(event) => setDriveAccessMode(event.target.value as ChatProject['driveAccessMode'])}
                    disabled={busy}
                    className="h-10 w-full rounded-md border border-white/10 bg-[#101012] px-3 text-sm text-white outline-none focus:border-emerald-500/60 disabled:opacity-50"
                  >
                    <option value="disabled">{t('projectDriveDisabled')}</option>
                    <option value="owner_library" disabled={visibility === 'shared'}>
                      {t('projectDriveOwnerLibrary')}
                    </option>
                    <option value="explicit_resources">{t('projectDriveExplicitResources')}</option>
                  </select>
                </label>
              </div>

              <label className="block space-y-2">
                <span className="text-sm font-medium text-zinc-300">{t('projectInstructions')}</span>
                <textarea
                  value={instructions}
                  onChange={(event) => setInstructions(event.target.value)}
                  disabled={busy}
                  rows={6}
                  className="w-full resize-y rounded-md border border-white/10 bg-[#101012] px-3 py-2 text-sm leading-6 text-white outline-none transition-colors focus:border-emerald-500/60 disabled:opacity-50"
                />
              </label>

              <label className="block space-y-2">
                <span className="text-sm font-medium text-zinc-300">{t('projectMemory')}</span>
                <select
                  value={memorySpaceId}
                  onChange={(event) => setMemorySpaceId(event.target.value)}
                  disabled={busy}
                  className="h-10 w-full rounded-md border border-white/10 bg-[#101012] px-3 text-sm text-white outline-none focus:border-emerald-500/60 disabled:opacity-50"
                >
                  <option value="">{t('projectMemoryDefault')}</option>
                  {settings?.memorySpaces.map((space) => (
                    <option key={space.spaceId} value={space.spaceId}>{space.displayName}</option>
                  ))}
                </select>
              </label>
            </div>
          )}
        </div>

        <footer className="flex flex-wrap items-center justify-between gap-3 border-t border-white/10 px-5 py-4">
          <button
            type="button"
            onClick={handleDelete}
            disabled={busy}
            className="inline-flex h-9 items-center gap-2 rounded-md px-3 text-sm font-medium text-red-400 transition-colors hover:bg-red-500/10 disabled:opacity-40"
          >
            {deleting ? <Loader2 size={16} className="animate-spin" /> : <Trash2 size={16} />}
            {t('deleteProject')}
          </button>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onClose}
              disabled={busy}
              className="h-9 rounded-md px-4 text-sm font-medium text-zinc-300 transition-colors hover:bg-white/10 disabled:opacity-40"
            >
              {t('cancel')}
            </button>
            <button
              type="button"
              onClick={handleSave}
              disabled={busy || !settings || !name.trim()}
              className="inline-flex h-9 items-center gap-2 rounded-md bg-emerald-600 px-4 text-sm font-semibold text-white transition-colors hover:bg-emerald-500 disabled:cursor-not-allowed disabled:opacity-40"
            >
              {saving ? <Loader2 size={16} className="animate-spin" /> : <Save size={16} />}
              {t('save')}
            </button>
          </div>
        </footer>
      </div>
    </div>
  );
};
