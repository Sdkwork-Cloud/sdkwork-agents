import React, { useState } from 'react';
import { Camera, Bot, Server } from 'lucide-react';
import { toast } from './Toast';
import { agentService } from '../services/AgentService';
import { ModalWrapper } from './ModalWrapper';
import { DEFAULT_AGENT_CONFIG } from './AgentDefaults';
import { createDefaultAvatar } from '../services/DefaultAvatarService';

const DEFAULT_AGENT_AVATAR = createDefaultAvatar('agent');

export const CreateAgentModal: React.FC<{ isOpen: boolean; onClose: () => void; onSuccess: (agentId?: string) => void }> = ({ isOpen, onClose, onSuccess }) => {
  const [name, setName] = useState('');
  const [desc, setDesc] = useState('');
  const [type, setType] = useState<'normal' | 'independent'>('normal');
  const [creating, setCreating] = useState(false);

  // Reset state when modal opens/closes
  React.useEffect(() => {
    if (!isOpen) {
      setName('');
      setDesc('');
      setType('normal');
    }
  }, [isOpen]);

  return (
    <ModalWrapper 
      isOpen={isOpen} 
      onClose={onClose} 
      title="创建智能体"
      width="w-[520px]"
      footer={
        <>
          <button onClick={onClose} className="px-4 py-2 rounded bg-white/5 text-gray-300 hover:bg-white/10 transition-colors text-sm">取消</button>
          <button 
            disabled={!name.trim() || creating}
            className={`px-4 py-2 rounded text-white transition-colors text-sm flex items-center gap-1 ${name.trim() && !creating ? 'bg-[#00b42a] hover:bg-[#009a24]' : 'bg-[#00b42a]/50 cursor-not-allowed'}`}
            onClick={async () => {
              setCreating(true);
              try {
                const createdAgent = await agentService.createAgent({
                  ...DEFAULT_AGENT_CONFIG,
                  name,
                  description: desc,
                  type,
                  avatar: DEFAULT_AGENT_AVATAR
                });
                toast(`智能体 "${name}" 创建成功`, 'success');
                onSuccess(createdAgent.id);
              } catch (error) {
                toast('创建智能体失败', 'error');
              } finally {
                setCreating(false);
              }
            }}
          >
            {creating ? '创建中...' : '下一步'}
          </button>
        </>
      }
    >
      <div className="space-y-6">
        {/* Avatar Upload */}
        <div className="flex flex-col items-center justify-center">
          <label className="w-20 h-20 rounded-full bg-[#181818] border border-white/10 flex items-center justify-center cursor-pointer hover:bg-white/5 transition-colors group relative overflow-hidden mb-2">
            <input type="file" className="hidden" accept="image/*" onChange={(e) => {
               if (e.target.files && e.target.files.length > 0) {
                 toast('头像已更新', 'success');
               }
            }} />
            <Camera size={24} className="text-gray-400 group-hover:text-gray-200 transition-colors" />
            <div className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
              <span className="text-[11px] text-white">上传头像</span>
            </div>
          </label>
        </div>

        {/* Name */}
        <div>
          <label className="block text-sm text-gray-400 mb-1.5">智能体名称 <span className="text-red-500">*</span></label>
          <input 
            type="text" 
            placeholder="例如：代码助手、翻译专家" 
            value={name}
            onChange={e => setName(e.target.value)}
            className="w-full bg-[#181818] border border-white/5 rounded-lg py-2.5 px-3 text-sm text-gray-200 outline-none focus:border-white/20 transition-colors" 
          />
        </div>

        {/* Type Selection */}
        <div>
          <label className="block text-sm text-gray-400 mb-1.5">智能体类型</label>
          <div className="grid grid-cols-2 gap-3">
            <div 
              onClick={() => setType('normal')}
              className={`p-3 rounded-lg border cursor-pointer transition-all flex items-start gap-3 ${
                type === 'normal' ? 'bg-[#00b42a]/10 border-[#00b42a]/50' : 'bg-[#181818] border-white/5 hover:border-white/10'
              }`}
            >
              <div className={`mt-0.5 ${type === 'normal' ? 'text-[#00b42a]' : 'text-gray-500'}`}>
                <Bot size={18} />
              </div>
              <div>
                <div className={`text-sm font-medium mb-0.5 ${type === 'normal' ? 'text-[#00b42a]' : 'text-gray-200'}`}>普通智能体</div>
                <div className="text-[11px] text-gray-500 leading-tight">在当前工作空间内运行，共享资源</div>
              </div>
            </div>
            
            <div 
              onClick={() => setType('independent')}
              className={`p-3 rounded-lg border cursor-pointer transition-all flex items-start gap-3 ${
                type === 'independent' ? 'bg-[#00b42a]/10 border-[#00b42a]/50' : 'bg-[#181818] border-white/5 hover:border-white/10'
              }`}
            >
              <div className={`mt-0.5 ${type === 'independent' ? 'text-[#00b42a]' : 'text-gray-500'}`}>
                <Server size={18} />
              </div>
              <div>
                <div className={`text-sm font-medium mb-0.5 ${type === 'independent' ? 'text-[#00b42a]' : 'text-gray-200'}`}>独立部署智能体</div>
                <div className="text-[11px] text-gray-500 leading-tight">拥有独立的运行环境和专属资源</div>
              </div>
            </div>
          </div>
        </div>

        {/* Description */}
        <div>
          <label className="block text-sm text-gray-400 mb-1.5">简介</label>
          <textarea 
            placeholder="一句话描述它的作用" 
            value={desc}
            onChange={e => setDesc(e.target.value)}
            className="w-full bg-[#181818] border border-white/5 rounded-lg py-2.5 px-3 text-sm text-gray-200 outline-none focus:border-white/20 h-20 resize-none custom-scrollbar transition-colors" 
          />
        </div>
      </div>
    </ModalWrapper>
  );
};
