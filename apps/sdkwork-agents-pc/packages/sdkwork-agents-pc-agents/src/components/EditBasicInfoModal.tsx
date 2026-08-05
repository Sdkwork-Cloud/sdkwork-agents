import React, { useEffect, useState } from 'react';
import { Bot } from 'lucide-react';
import { motion } from 'motion/react';

import type { AgentsDriveMediaResource } from '@sdkwork/agents-pc-core/sdk/driveUploadService';

import { toast } from './Toast';

export interface EditBasicInfoModalProps {
  isOpen: boolean;
  onClose: () => void;
  initialName: string;
  initialDesc: string;
  initialAvatar: string;
  initialAvatarPreview?: string;
  onUploadAvatar: (file: File) => Promise<AgentsDriveMediaResource>;
  onSave: (name: string, desc: string, avatar: string, avatarPreview?: string) => void;
}

export const EditBasicInfoModal: React.FC<EditBasicInfoModalProps> = ({
  isOpen,
  onClose,
  initialName,
  initialDesc,
  initialAvatar,
  initialAvatarPreview,
  onUploadAvatar,
  onSave,
}) => {
  const [tempName, setTempName] = useState(initialName);
  const [tempDesc, setTempDesc] = useState(initialDesc);
  const [tempAvatar, setTempAvatar] = useState(initialAvatar);
  const [tempAvatarPreview, setTempAvatarPreview] = useState(initialAvatarPreview || initialAvatar);
  const [uploadingAvatar, setUploadingAvatar] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setTempName(initialName);
    setTempDesc(initialDesc);
    setTempAvatar(initialAvatar);
    setTempAvatarPreview(initialAvatarPreview || initialAvatar);
  }, [isOpen, initialName, initialDesc, initialAvatar, initialAvatarPreview]);

  if (!isOpen) return null;

  const handleAvatarChange = (event: React.ChangeEvent<HTMLInputElement>): void => {
    const file = event.target.files?.[0];
    event.target.value = '';
    if (!file) return;

    setUploadingAvatar(true);
    void onUploadAvatar(file)
      .then((media) => {
        setTempAvatar(media.uri ?? '');
        setTempAvatarPreview(media.url ?? '');
        toast('头像已通过 SDKWork Drive 上传', 'success');
      })
      .catch((error) => {
        console.error('Avatar Drive upload failed', error);
        toast('头像上传失败，请检查登录状态和 Drive 服务', 'error');
      })
      .finally(() => setUploadingAvatar(false));
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="flex w-[480px] flex-col overflow-hidden rounded-xl border border-white/10 bg-[#222] shadow-2xl"
      >
        <div className="flex items-center justify-between border-b border-white/5 bg-[#1a1a1a] p-4">
          <h3 className="font-medium text-gray-200">编辑基础信息</h3>
        </div>
        <div className="space-y-4 p-6">
          <div className="mb-2 flex flex-col items-center justify-center">
            <label className="group relative flex h-20 w-20 cursor-pointer items-center justify-center overflow-hidden rounded-full border border-white/10 bg-[#181818] transition-colors hover:bg-white/5">
              {tempAvatarPreview && !tempAvatarPreview.startsWith('drive://') ? (
                <img src={tempAvatarPreview} alt="智能体头像" className="h-full w-full object-cover" />
              ) : (
                <Bot size={32} className="text-gray-500" />
              )}
              <input
                type="file"
                className="hidden"
                accept="image/*"
                disabled={uploadingAvatar}
                onChange={handleAvatarChange}
              />
              <div className="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition-opacity group-hover:opacity-100" />
              <div className="absolute bottom-1 z-10 rounded-full bg-black/60 px-2 py-0.5 text-[10px] text-gray-400 opacity-0 transition-opacity group-hover:opacity-100 group-hover:text-gray-200">
                {uploadingAvatar ? '上传中' : '更换'}
              </div>
            </label>
          </div>
          <div>
            <label className="mb-1.5 block text-sm text-gray-400">智能体名称</label>
            <input
              type="text"
              value={tempName}
              onChange={(event) => setTempName(event.target.value)}
              className="w-full rounded-lg border border-white/5 bg-[#181818] px-3 py-2.5 text-sm text-gray-200 outline-none transition-colors focus:border-white/20"
            />
          </div>
          <div>
            <label className="mb-1.5 block text-sm text-gray-400">简介</label>
            <textarea
              value={tempDesc}
              onChange={(event) => setTempDesc(event.target.value)}
              className="custom-scrollbar h-20 w-full resize-none rounded-lg border border-white/5 bg-[#181818] px-3 py-2.5 text-sm text-gray-200 outline-none transition-colors focus:border-white/20"
            />
          </div>
        </div>
        <div className="flex justify-end gap-2 border-t border-white/5 bg-[#1a1a1a] p-4">
          <button
            type="button"
            onClick={onClose}
            className="rounded bg-white/5 px-4 py-2 text-sm text-gray-300 hover:bg-white/10"
          >
            取消
          </button>
          <button
            type="button"
            disabled={!tempName.trim() || uploadingAvatar}
            onClick={() => onSave(tempName, tempDesc, tempAvatar, tempAvatarPreview)}
            className="rounded bg-[#00b42a] px-4 py-2 text-sm text-white transition-colors hover:bg-[#009a24] disabled:bg-[#00b42a]/50"
          >
            保存
          </button>
        </div>
      </motion.div>
    </div>
  );
};
