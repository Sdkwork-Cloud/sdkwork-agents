import React, { useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { Smile, Heart, Search, Ghost, Coffee, Zap, Plus } from 'lucide-react';
import { toast } from './Toast';

const UNICODE_EMOJIS = ['😀','😃','😄','😁','😆','😅','😂','🤣','🥲','☺️','😊','😇','🙂','🙃','😉','😌','😍','🥰','😘','😗','😙','😚','😋','😛','😝','😜','🤪','🤨','🧐','🤓','😎','🥸','🤩','🥳','😏','😒','😞','😔','😟','😕','🙁','☹️','😣','😖','😫','😩','🥺','😢','😭','😤','😠','😡','🤬','🤯','😳','🥵','🥶','😱','😨','😰','😥','😓','🤗','🤔','🤭','🤫','🤥','😶','😐','😑','😬','🙄','😯','😦','😧','😮','😲','🥱','😴','🤤','😪','😵','🤐','🥴','🤢','🤮','🤧','😷','🤒','🤕','🤑','🤠','😈','👿','👹','👺','🤡','💩','👻','💀','👽','👾','🤖','🎃','😺','😸','😹','😻','😼','😽','🙀','😿','😾','🙌','👐','🤲','🤝','👍','👎','👊','✊','🤛','🤜','🤞','✌️','🫰','🤟','🤘','👌','🤌','🤏','👈','👉','👆','👇','☝️','✋','🤚','🖐','🖖','👋','🤙','💪','🖕','✍️','🙏'];

const STICKER_PACKS: Record<string, string[]> = {
  favorites: [],
  pack1: [],
  pack2: [],
  pack3: [],
};

const PC_STICKER_PACK_CONTRACT_UNAVAILABLE = 'pc sticker pack contract is not available';

const EmojiContent = React.memo<{
  activeEmojiTab: string;
  onEmojiClick: (e: string) => void;
  onStickerClick: (url: string) => void;
}>(({ activeEmojiTab, onEmojiClick, onStickerClick }) => {
  const stickerPack = STICKER_PACKS[activeEmojiTab] ?? [];

  return (
    <div className="flex-1 overflow-y-auto custom-scrollbar bg-[#1e1e1e] p-4 relative">
      {activeEmojiTab === 'emoji' ? (
        <div className="grid grid-cols-10 gap-x-1 gap-y-3">
          {UNICODE_EMOJIS.map((emoji, i) => (
            <button 
              key={i} 
              className="w-[34px] h-[34px] flex items-center justify-center text-[24px] hover:bg-white/10 rounded-lg transition-all hover:scale-110 active:scale-95"
              onClick={() => onEmojiClick(emoji)}
            >
              {emoji}
            </button>
          ))}
        </div>
      ) : stickerPack.length === 0 ? (
        <div className="h-full flex items-center justify-center px-6 text-center text-sm text-gray-400">
          {PC_STICKER_PACK_CONTRACT_UNAVAILABLE}
        </div>
      ) : (
        <div className="grid grid-cols-4 gap-3">
          {stickerPack.map((url, i) => (
            <div 
              key={i} 
              className="aspect-square bg-[#2b2b2d] rounded-xl overflow-hidden cursor-pointer hover:ring-2 hover:ring-indigo-500/50 transition-all group relative"
              onClick={() => onStickerClick(url)}
            >
              <img src={url} loading="lazy" decoding="async" className="w-full h-full object-cover group-hover:scale-105 transition-transform" />
            </div>
          ))}
        </div>
      )}
    </div>
  );
});

export interface EmojiPickerProps {
  show: boolean;
  onClose: () => void;
  activeEmojiTab: string;
  setActiveEmojiTab: (tab: string) => void;
  onEmojiClick: (emoji: string) => void;
  onStickerClick: (url: string) => void;
}

export const EmojiPicker: React.FC<EmojiPickerProps> = ({
  show,
  onClose,
  activeEmojiTab,
  setActiveEmojiTab,
  onEmojiClick,
  onStickerClick
}) => {
  const emojiPickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (emojiPickerRef.current && !emojiPickerRef.current.contains(event.target as Node)) {
        onClose();
      }
    };
    if (show) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [show, onClose]);

  return (
    <div className="absolute bottom-full left-0 mb-3 z-50 rounded-xl overflow-hidden border border-white/10 bg-[#1e1e1e] shadow-[0_8px_30px_rgb(0,0,0,0.4)] origin-bottom-left flex flex-col w-[420px] h-[460px] " ref={emojiPickerRef}>
        {/* Top Header */}
        <div className="px-5 py-3 text-[13px] text-gray-400 font-medium shrink-0 bg-[#1e1e1e] border-b border-white/5 flex items-center justify-between">
          <span>{activeEmojiTab === 'emoji' ? '所有表情' : activeEmojiTab === 'favorites' ? '我的收藏' : '表情包组合'}</span>
          {activeEmojiTab !== 'emoji' && <button type="button" disabled className="text-gray-500 text-[12px] cursor-not-allowed">管理</button>}
        </div>

        {/* Content Area */}
        <EmojiContent 
          activeEmojiTab={activeEmojiTab} 
          onEmojiClick={onEmojiClick} 
          onStickerClick={onStickerClick} 
        />

        {/* Bottom Toolbar Tabs */}
        <div className="h-12 border-t border-white/5 flex items-center shrink-0 bg-[#252525] relative">
          {/* Left Icon (Search) */}
          <button className="w-12 h-full flex items-center justify-center text-gray-400 hover:text-gray-200 hover:bg-white/5 shrink-0 border-r border-white/5 transition-colors">
            <Search size={20} strokeWidth={1.5} />
          </button>

          {/* Scrollable Tabs */}
          <div className="flex-1 overflow-x-auto flex h-full custom-scrollbar items-center px-2 gap-1">
            <button 
              className={`w-10 h-8 rounded shrink-0 flex items-center justify-center transition-all ${activeEmojiTab === 'emoji' ? 'bg-[#3a3a3a] text-gray-200 shadow-sm' : 'text-gray-400 hover:text-gray-200'}`}
              onClick={() => setActiveEmojiTab('emoji')}
            >
              <Smile size={20} strokeWidth={1.5} />
            </button>
            <button 
              className={`w-10 h-8 rounded shrink-0 flex items-center justify-center transition-all ${activeEmojiTab === 'favorites' ? 'bg-[#3a3a3a] text-red-400 shadow-sm' : 'text-gray-400 hover:text-red-400'}`}
              onClick={() => setActiveEmojiTab('favorites')}
            >
              <Heart size={20} strokeWidth={1.5} className={activeEmojiTab === 'favorites' ? 'fill-red-400/20' : ''} />
            </button>
            <div className="w-px h-4 bg-white/10 mx-1 shrink-0" />
            <button 
              className={`w-10 h-8 rounded shrink-0 flex items-center justify-center transition-all ${activeEmojiTab === 'pack1' ? 'bg-[#3a3a3a] text-indigo-400 shadow-sm' : 'text-gray-400 hover:text-gray-200'}`}
              onClick={() => setActiveEmojiTab('pack1')}
            >
              <Ghost size={20} strokeWidth={1.5} />
            </button>
            <button 
              className={`w-10 h-8 rounded shrink-0 flex items-center justify-center transition-all ${activeEmojiTab === 'pack2' ? 'bg-[#3a3a3a] text-amber-400 shadow-sm' : 'text-gray-400 hover:text-gray-200'}`}
              onClick={() => setActiveEmojiTab('pack2')}
            >
              <Coffee size={20} strokeWidth={1.5} />
            </button>
            <button 
              className={`w-10 h-8 rounded shrink-0 flex items-center justify-center transition-all ${activeEmojiTab === 'pack3' ? 'bg-[#3a3a3a] text-emerald-400 shadow-sm' : 'text-gray-400 hover:text-gray-200'}`}
              onClick={() => setActiveEmojiTab('pack3')}
            >
              <Zap size={20} strokeWidth={1.5} />
            </button>
          </div>

          {/* Right Icon (Add) */}
          <button 
            className="w-12 h-full flex items-center justify-center text-gray-400 hover:text-gray-200 bg-[#3a3a3a]/50 hover:bg-[#3a3a3a] shrink-0 border-l border-white/5 transition-colors group"
            onClick={() => {
              toast(PC_STICKER_PACK_CONTRACT_UNAVAILABLE, 'error');
            }}
          >
            <Plus size={20} strokeWidth={1.5} className="group-hover:rotate-90 transition-transform" />
          </button>
        </div>
    </div>
  );
};
