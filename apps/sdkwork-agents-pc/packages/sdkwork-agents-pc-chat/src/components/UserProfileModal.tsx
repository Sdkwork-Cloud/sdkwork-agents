import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { 
  X, Check, Shield, Cpu, RefreshCw, Key, LogOut, Code, AlertCircle,
  Laptop, Smartphone, CreditCard, ChevronRight, Activity, Copy, Sparkles 
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ProfileTab } from './profile/ProfileTab';
import { BillingTab } from './profile/BillingTab';
import { UsageTab } from './profile/UsageTab';
import { SecurityTab } from './profile/SecurityTab';
import { DeveloperTab } from './profile/DeveloperTab';

interface UserProfileModalProps {
  onClose: () => void;
  initialTab?: 'profile' | 'billing' | 'usage' | 'security' | 'developer';
}

interface ActiveSession {
  id: string;
  device: string;
  browser: string;
  ip: string;
  location: string;
  activeNow: boolean;
  type: 'desktop' | 'mobile';
}

const AVATAR_COLOR_TEMPLATES = [
  { name: 'Sky Blue', class: 'bg-[#1890ff] text-white', colorHex: '#1890ff' },
  { name: 'Emerald', class: 'bg-emerald-500 text-white', colorHex: '#10b981' },
  { name: 'Violet', class: 'bg-violet-500 text-white', colorHex: '#8b5cf6' },
  { name: 'Sunset Orange', class: 'bg-orange-500 text-white', colorHex: '#f97316' },
  { name: 'Rose', class: 'bg-rose-500 text-white', colorHex: '#f43f5e' },
  { name: 'Midnight', class: 'bg-[#1f1f1f] border border-gray-700 text-white', colorHex: '#1f1f1f' }
];

export const UserProfileModal: React.FC<UserProfileModalProps> = ({ onClose, initialTab = 'profile' }) => {
  const { t } = useTranslation('settings');
  const { t: tCommon } = useTranslation('common');

  // Load from locale-storage or default
  const [username, setUsername] = useState(() => localStorage.getItem('profile_username') || tCommon('mockUserName'));
  const [email, setEmail] = useState(() => localStorage.getItem('profile_email') || 'buptrolex@gmail.com');
  const [bio, setBio] = useState(() => localStorage.getItem('profile_bio') || t('defaultBio'));
  const [avatarIndex, setAvatarIndex] = useState(() => parseInt(localStorage.getItem('profile_avatar_index') || '0', 10));

  // Visual/Functional Quota metrics (reactive state so users can click "Simulate API Call" to preview updates)
  const [tokenUsage, setTokenUsage] = useState(1284520); // 1.28M / 5M max
  const [messageUsage, setMessageUsage] = useState(342); // 342 / 1000 max
  const [imageUsage, setImageUsage] = useState(48); // 48 / 200 max

  const [activeTab, setActiveTab] = useState<'profile' | 'billing' | 'usage' | 'security' | 'developer'>(initialTab);
  const [isToastVisible, setIsToastVisible] = useState(false);
  const [toastMessage, setToastMessage] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  // Active devices state
  const [sessions, setSessions] = useState<ActiveSession[]>([
    { id: '1', device: 'MacBook Pro 16"', browser: 'Chrome Browser', ip: '198.51.100.41', location: 'Tokyo, Japan', activeNow: true, type: 'desktop' },
    { id: '2', device: 'iPhone 15 Pro Max', browser: 'Safari Mobile', ip: '198.51.100.82', location: 'Tokyo, Japan', activeNow: false, type: 'mobile' },
    { id: '3', device: 'iPad Pro M4', browser: 'Safari Mobile', ip: '203.0.113.12', location: 'London, UK', activeNow: false, type: 'mobile' },
    { id: '4', device: 'Work PC Linux x86', browser: 'Firefox Developer Edition', ip: '192.0.2.14', location: 'San Francisco, USA', activeNow: false, type: 'desktop' }
  ]);

  const triggerToast = (msg: string) => {
    setToastMessage(msg);
    setIsToastVisible(true);
  };

  useEffect(() => {
    if (isToastVisible) {
      const timer = setTimeout(() => setIsToastVisible(false), 2500);
      return () => clearTimeout(timer);
    }
  }, [isToastVisible]);

  const handleSaveProfile = () => {
    setIsSaving(true);
    setTimeout(() => {
      localStorage.setItem('profile_username', username);
      localStorage.setItem('profile_email', email);
      localStorage.setItem('profile_bio', bio);
      localStorage.setItem('profile_avatar_index', avatarIndex.toString());
      setIsSaving(false);
      triggerToast(t('saveProfile') + ' - ' + tCommon('copied').replace('!', ''));
    }, 600);
  };

  // Simulation handler: makes metrics go up! Fun & interactive closed-loop test
  const handleSimulateCall = () => {
    setTokenUsage(prev => Math.min(prev + Math.floor(Math.random() * 45000) + 12000, 5000000));
    setMessageUsage(prev => Math.min(prev + 1, 1000));
    if (Math.random() > 0.6) {
      setImageUsage(prev => Math.min(prev + 1, 200));
    }
    triggerToast(t('apiSimulated'));
  };

  const handleRevokeSessions = () => {
    setSessions(prev => prev.filter(s => s.activeNow));
    triggerToast(t('revokeAll'));
  };

  const handleCopyCode = () => {
    const codeText = `import { ChatService } from '@sdkwork/agents-pc-chat';

// 1. Configure standard streams
await ChatService.streamChat({
  model: 'gemini-2.5-flash',
  vendor: 'Google',
  messages: [
    { role: 'user', text: 'Hello, World!' }
  ],
  onMessageUpdate: (text) => {
    console.log("Chunk received:", text);
  },
  onComplete: () => {
    console.log("Stream delivery finished.");
  }
});`;

    navigator.clipboard.writeText(codeText);
    triggerToast(t('copiedSnippet'));
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      {/* Toast Alert */}
      <AnimatePresence>
        {isToastVisible && (
          <motion.div 
            initial={{ opacity: 0, y: -20, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -20, scale: 0.95 }}
            className="fixed top-6 left-1/2 -translate-x-1/2 bg-gray-900 text-white py-3 px-6 rounded-xl shadow-2xl flex items-center gap-2.5 text-sm font-medium z-999 border border-gray-800"
          >
            <Sparkles size={16} className="text-[#1890ff] animate-pulse" />
            <span>{toastMessage}</span>
          </motion.div>
        )}
      </AnimatePresence>

      <motion.div 
        id="user-profile-modal-container"
        initial={{ opacity: 0, scale: 0.97, y: 15 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.97, y: 15 }}
        className="bg-white dark:bg-[#1e1e1e] border border-gray-100 dark:border-zinc-800 rounded-3xl w-full max-w-4xl h-[700px] shadow-2xl overflow-hidden flex flex-col max-h-[92vh]"
      >
        {/* Header bar */}
        <div className="px-6 py-5 border-b border-gray-100 dark:border-zinc-800 flex justify-between items-center bg-gray-50/50 dark:bg-zinc-900/45 shrink-0">
          <div className="flex items-center gap-3">
            <div className={`w-8 h-8 rounded-xl ${AVATAR_COLOR_TEMPLATES[avatarIndex].class} flex items-center justify-center font-bold text-sm shadow-sm transition-all duration-300`}>
              {username.substring(0, 2).toUpperCase()}
            </div>
            <div>
              <h2 className="font-semibold text-base text-gray-900 dark:text-white leading-none mb-0.5">{t('profile')}</h2>
              <p className="text-[11px] text-gray-500 dark:text-zinc-400 font-mono">{t('profileDetail')}</p>
            </div>
          </div>
          <button 
            onClick={onClose} 
            className="text-gray-400 dark:text-zinc-500 hover:text-gray-800 dark:hover:text-zinc-200 p-1.5 rounded-xl hover:bg-gray-100 dark:hover:bg-zinc-800/80 transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        {/* Modal body (Sidebar/Content layout) */}
        <div className="flex flex-1 min-h-0 overflow-hidden">
          {/* Left Navigation tabs */}
          <div className="w-56 bg-gray-50/50 dark:bg-zinc-900/10 border-r border-gray-100 dark:border-zinc-800 p-4 shrink-0 flex flex-col justify-between">
            <div className="space-y-1">
              {[
                { id: 'profile', label: t('profile'), icon: <Cpu size={16} /> },
                { id: 'billing', label: t('plan'), icon: <CreditCard size={16} /> },
                { id: 'usage', label: t('usage'), icon: <Activity size={16} /> },
                { id: 'security', label: t('sessions'), icon: <Shield size={16} /> },
                { id: 'developer', label: t('developer'), icon: <Code size={16} /> },
              ].map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  className={`w-full flex items-center gap-3 px-3 py-3 rounded-xl text-xs font-semibold tracking-wide transition-all ${
                    activeTab === tab.id
                      ? 'bg-[#1890ff] text-white shadow-md shadow-[#1890ff]/15'
                      : 'text-gray-600 dark:text-zinc-300 hover:bg-gray-100 dark:hover:bg-zinc-800/60'
                  }`}
                >
                  <span className="shrink-0">{tab.icon}</span>
                  <span className="truncate flex-1 text-left">{tab.label}</span>
                </button>
              ))}
            </div>

            {/* Simulated Live Action widget */}
            <div className="p-3.5 rounded-2xl bg-gradient-to-br from-[#1890ff]/5 via-transparent to-pink-500/5 border border-gray-100 dark:border-zinc-800">
              <span className="text-[10px] uppercase tracking-wider text-gray-400 font-bold block mb-1">{t('sandboxLive')}</span>
              <p className="text-[11px] text-gray-500 dark:text-zinc-400 mb-2 leading-relaxed">{t('sandboxDesc')}</p>
              <button 
                onClick={handleSimulateCall}
                className="w-full flex items-center justify-center gap-1.5 py-2 bg-gray-100 dark:bg-zinc-800 hover:bg-gray-200 dark:hover:bg-zinc-700 text-gray-800 dark:text-zinc-200 text-[10px] font-bold rounded-xl transition-all shadow-sm"
              >
                <RefreshCw size={11} className="animate-spin-slow text-[#1890ff]" />
                {t('simulateUsage')}
              </button>
            </div>
          </div>

          {/* Right workspace panels */}
          <div className="flex-1 overflow-y-auto p-8 relative bg-white dark:bg-[#1d1d1d]">
            
            {activeTab === 'profile' && (
              <ProfileTab 
                t={t}
                tCommon={tCommon}
                username={username}
                setUsername={setUsername}
                email={email}
                setEmail={setEmail}
                bio={bio}
                setBio={setBio}
                avatarIndex={avatarIndex}
                setAvatarIndex={setAvatarIndex}
                isSaving={isSaving}
                handleSaveProfile={handleSaveProfile}
                AVATAR_COLOR_TEMPLATES={AVATAR_COLOR_TEMPLATES}
              />
            )}

            {activeTab === 'billing' && (
              <BillingTab 
                t={t} 
                tCommon={tCommon} 
              />
            )}

            {activeTab === 'usage' && (
              <UsageTab 
                t={t}
                tokenUsage={tokenUsage}
                messageUsage={messageUsage}
                imageUsage={imageUsage}
                handleSimulateCall={handleSimulateCall}
              />
            )}

            {activeTab === 'security' && (
              <SecurityTab 
                t={t}
                sessions={sessions}
                setSessions={setSessions}
                handleRevokeSessions={handleRevokeSessions}
                triggerToast={triggerToast}
              />
            )}

            {activeTab === 'developer' && (
              <DeveloperTab 
                t={t}
                handleCopyCode={handleCopyCode}
              />
            )}

          </div>
        </div>
      </motion.div>
    </div>
  );
};
