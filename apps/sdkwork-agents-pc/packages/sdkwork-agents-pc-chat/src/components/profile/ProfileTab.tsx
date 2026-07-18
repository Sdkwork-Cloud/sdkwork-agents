import React from 'react';
import { Check, RefreshCw } from 'lucide-react';

interface ProfileTabProps {
  t: (key: string) => string;
  tCommon: (key: string) => string;
  username: string;
  setUsername: (val: string) => void;
  email: string;
  setEmail: (val: string) => void;
  bio: string;
  setBio: (val: string) => void;
  avatarIndex: number;
  setAvatarIndex: (val: number) => void;
  isSaving: boolean;
  handleSaveProfile: () => void;
  AVATAR_COLOR_TEMPLATES: { name: string; class: string; colorHex: string; }[];
}

export const ProfileTab: React.FC<ProfileTabProps> = ({
  t,
  tCommon,
  username,
  setUsername,
  email,
  setEmail,
  bio,
  setBio,
  avatarIndex,
  setAvatarIndex,
  isSaving,
  handleSaveProfile,
  AVATAR_COLOR_TEMPLATES
}) => {
  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-bold text-gray-900 dark:text-white tracking-tight">{t('profile')}</h3>
        <p className="text-xs text-gray-500 mt-1">{t('profileSubtitle')}</p>
      </div>

      <div className="flex items-center gap-5 p-5 rounded-2xl bg-gray-50/50 dark:bg-zinc-900/40 border border-gray-100 dark:border-zinc-850">
        <div className={`w-16 h-16 rounded-2xl ${AVATAR_COLOR_TEMPLATES[avatarIndex].class} flex items-center justify-center font-extrabold text-2xl shadow-md cursor-pointer relative hover:scale-105 transition-transform`}>
          {username.substring(0, 2).toUpperCase()}
        </div>
        <div>
          <h4 className="font-semibold text-sm text-gray-900 dark:text-zinc-100">{username}</h4>
          <p className="text-xs text-gray-500 mt-0.5">{email}</p>
          <div className="mt-2.5 flex items-center gap-2">
            <span className="px-2.5 py-0.5 rounded-full text-[10px] font-semibold tracking-wider bg-emerald-500/10 text-emerald-500 dark:bg-emerald-500/20">{t('verifiedEmail')}</span>
            <span className="px-2.5 py-0.5 rounded-full text-[10px] font-semibold tracking-wider bg-[#1890ff]/10 text-[#1890ff] dark:bg-[#1890ff]/20">{t('proWorkspace')}</span>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-1.5">
          <label className="block text-xs font-semibold text-gray-500 tracking-wide">{t('username')}</label>
          <input 
            type="text" 
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="w-full bg-gray-50/50 dark:bg-zinc-900/50 border border-gray-200 dark:border-zinc-800 rounded-xl px-4 py-2.5 text-xs text-gray-900 dark:text-white focus:ring-1 focus:ring-[#1890ff] outline-none transition-all dark:focus:border-[#1890ff]"
          />
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-semibold text-gray-500 tracking-wide">{t('email')}</label>
          <input 
            type="email" 
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="w-full bg-gray-50/50 dark:bg-zinc-900/50 border border-gray-200 dark:border-zinc-800 rounded-xl px-4 py-2.5 text-xs text-gray-900 dark:text-white focus:ring-1 focus:ring-[#1890ff] outline-none transition-all dark:focus:border-[#1890ff]"
          />
        </div>

        <div className="col-span-2 space-y-1.5">
          <label className="block text-xs font-semibold text-gray-500 tracking-wide">{t('bio')}</label>
          <textarea 
            rows={2}
            value={bio}
            onChange={(e) => setBio(e.target.value)}
            className="w-full bg-gray-50/50 dark:bg-zinc-900/50 border border-gray-200 dark:border-zinc-800 rounded-xl px-4 py-2.5 text-xs text-gray-900 dark:text-white focus:ring-1 focus:ring-[#1890ff] outline-none transition-all dark:focus:border-[#1890ff]"
          />
        </div>
      </div>

      <div className="space-y-2">
        <label className="block text-xs font-semibold text-gray-500 tracking-wide">{t('changeAvatar')}</label>
        <div className="flex gap-2.5">
          {AVATAR_COLOR_TEMPLATES.map((color, idx) => (
            <button
              key={color.name}
              onClick={() => setAvatarIndex(idx)}
              className={`w-9 h-9 rounded-xl flex items-center justify-center transition-all ${color.class} hover:scale-110 active:scale-95 ${
                avatarIndex === idx ? 'ring-2 ring-violet-500 ring-offset-2 dark:ring-offset-[#1d1d1d]' : 'opacity-85'
              }`}
              title={color.name}
            >
              {avatarIndex === idx && <Check size={14} className="text-white" />}
            </button>
          ))}
        </div>
      </div>

      <div className="pt-4 flex justify-end">
        <button
          onClick={handleSaveProfile}
          disabled={isSaving}
          className="px-6 py-2.5 bg-[#1890ff] hover:bg-[#1890ff]/90 text-white rounded-xl text-xs font-bold shadow-md transition-colors flex items-center gap-2"
        >
          {isSaving && <RefreshCw size={13} className="animate-spin" />}
          {t('saveProfile')}
        </button>
      </div>
    </div>
  );
};
