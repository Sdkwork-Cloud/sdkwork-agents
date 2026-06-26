import React, { useState } from 'react';
import { motion } from 'motion/react';
import { Bot } from 'lucide-react';
import { toast } from './Toast';

export const EditBasicInfoModal: React.FC<{
  isOpen: boolean;
  onClose: () => void;
  initialName: string;
  initialDesc: string;
  initialAvatar: string;
  onSave: (name: string, desc: string, avatar: string) => void;
}> = ({ isOpen, onClose, initialName, initialDesc, initialAvatar, onSave }) => {
  const [tempName, setTempName] = useState(initialName);
  const [tempDesc, setTempDesc] = useState(initialDesc);
  const [tempAvatar, setTempAvatar] = useState(initialAvatar);

  React.useEffect(() => {
    if (isOpen) {
      setTempName(initialName);
      setTempDesc(initialDesc);
      setTempAvatar(initialAvatar);
    }
  }, [isOpen, initialName, initialDesc, initialAvatar]);

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <motion.div 
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        className="w-[480px] bg-[#222] border border-white/10 rounded-xl shadow-2xl flex flex-col overflow-hidden"
      >
        <div className="flex items-center justify-between p-4 border-b border-white/5 bg-[#1a1a1a]">
          <h3 className="font-medium text-gray-200">编辑基础信息</h3>
        </div>
        <div className="p-6 space-y-4">
          <div className="flex flex-col items-center justify-center mb-2">
            <label className="w-20 h-20 rounded-full bg-[#181818] border border-white/10 flex items-center justify-center cursor-pointer hover:bg-white/5 transition-colors group relative overflow-hidden">
              {tempAvatar ? (
                <img src={tempAvatar} alt="Avatar" className="w-full h-full object-cover" />
              ) : (
                <Bot size={32} className="text-gray-500" />
              )}
              <input type="file" className="hidden" accept="image/*" onChange={(e) => {
                 if (e.target.files && e.target.files.length > 0) {
                   const file = e.target.files[0];
                   const url = URL.createObjectURL(file);
                   setTempAvatar(url);
                   toast('头像上传成功', 'success');
                 }
              }} />
              <div className="text-gray-400 group-hover:text-gray-200 transition-colors absolute z-10 bottom-1 text-[10px] bg-black/60 px-2 py-0.5 rounded-full opacity-0 group-hover:opacity-100">更换</div>
              <div className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
              </div>
            </label>
          </div>
          <div>
             <label className="block text-sm text-gray-400 mb-1.5">智能体名称</label>
             <input type="text" value={tempName} onChange={e => setTempName(e.target.value)} className="w-full bg-[#181818] border border-white/5 rounded-lg py-2.5 px-3 text-sm text-gray-200 outline-none focus:border-white/20 transition-colors" />
          </div>
          <div>
             <label className="block text-sm text-gray-400 mb-1.5">简介</label>
             <textarea value={tempDesc} onChange={e => setTempDesc(e.target.value)} className="w-full bg-[#181818] border border-white/5 rounded-lg py-2.5 px-3 text-sm text-gray-200 outline-none focus:border-white/20 h-20 resize-none transition-colors custom-scrollbar" />
          </div>
        </div>
        <div className="flex justify-end gap-2 p-4 border-t border-white/5 bg-[#1a1a1a]">
          <button onClick={onClose} className="px-4 py-2 rounded bg-white/5 text-gray-300 hover:bg-white/10 transition-colors text-sm">取消</button>
          <button 
            disabled={!tempName.trim()}
            onClick={() => {
              onSave(tempName, tempDesc, tempAvatar);
            }} 
            className="px-4 py-2 rounded bg-[#00b42a] hover:bg-[#009a24] disabled:bg-[#00b42a]/50 text-white transition-colors text-sm"
          >保存</button>
        </div>
      </motion.div>
    </div>
  );
};
