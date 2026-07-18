import React from 'react';
import { Copy, ImageIcon, Heart, Users } from 'lucide-react';
import { cn } from '@/packages/sdkwork-chatbox-pc-commons/src/components/MarkdownRenderer';

interface DiscoverTabProps {
  discoverData: {
    banner: any;
    cols: any[][];
  };
  onSelectImage: (item: any) => void;
}

const BannerItem = ({ item, onClick }: { item: any, onClick: (item: any) => void }) => (
  <div 
    className="relative group cursor-pointer overflow-hidden w-full aspect-[21/9] bg-[#141414] rounded-xl border border-white/5 shadow-md"
    onClick={() => onClick(item)}
  >
    <img src={item.src} alt={item.alt} className="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" />
    <div className="absolute inset-0 bg-gradient-to-r from-black/80 via-black/40 to-transparent" />
    <div className="absolute inset-0 flex flex-col justify-center px-10">
       <div className="text-cyan-400 text-[13px] font-bold tracking-wider mb-1.5 uppercase">AI影像创作单元</div>
       <div className="text-white text-3xl font-extrabold tracking-tight mb-2 leading-tight">2026大学生AI艺术季</div>
       <div className="text-white/60 text-[12px] flex items-center gap-1.5">
         <Users size={12} />
         <span>已有 {item.likes} 人参与创作</span>
       </div>
    </div>
  </div>
);

const GalleryItem = ({ item, className, onClick }: { key?: React.Key, item: any, className?: string, onClick: (item: any) => void }) => (
  <div 
    className={cn("relative group cursor-pointer overflow-hidden bg-[#141414] border border-white/5 rounded-xl transition-all hover:border-white/10", className)}
    onClick={() => onClick(item)}
  >
    <img src={item.src} alt={item.alt} className="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" />
    <div className="absolute inset-0 bg-black/30 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
    
    <div className="absolute bottom-0 left-0 right-0 p-4 bg-gradient-to-t from-black/90 via-black/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end">
      {item.title && (
        <div className="text-[13px] text-white font-medium line-clamp-2 mb-3 leading-snug">{item.title}</div>
      )}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <img src={item.avatar} className="w-5 h-5 rounded-full object-cover" />
          <span className="text-white/80 text-[11px] truncate">{item.author}</span>
        </div>
        <div className="flex items-center gap-2.5 text-white/70 shrink-0">
           <div title="复制提示词" className="cursor-pointer" onClick={(e) => { e.stopPropagation(); navigator.clipboard.writeText(item.prompt || ''); }}><Copy size={13} className="hover:text-white transition-colors" /></div>
           <div title="查看图片" className="cursor-pointer"><ImageIcon size={13} className="hover:text-white transition-colors" /></div>
           <div className="flex items-center gap-1 hover:text-white cursor-pointer transition-colors">
             <Heart size={13} />
             <span className="text-[11px] font-mono">{item.likes}</span>
           </div>
        </div>
      </div>
    </div>
  </div>
);

export const DiscoverTab: React.FC<DiscoverTabProps> = ({ discoverData, onSelectImage }) => {
  return (
    <div className="flex w-full items-start gap-4 bg-[#141414] overflow-hidden rounded-2xl">
       {/* Left Banner + 2 Cols */}
       <div className="w-1/3 flex flex-col gap-4">
          <BannerItem item={discoverData.banner} onClick={(item) => onSelectImage({
            ...item,
            prompt: item.prompt || item.title
          })} />
          <div className="flex w-full items-start gap-4">
             <div className="w-1/2 flex flex-col gap-4">
                {discoverData.cols[0].map(item => (
                  <GalleryItem key={item.id} item={item} onClick={onSelectImage} className="w-full h-[320px]" />
                ))}
             </div>
             <div className="w-1/2 flex flex-col gap-4">
                {discoverData.cols[1].map(item => (
                  <GalleryItem key={item.id} item={item} onClick={onSelectImage} className="w-full h-[320px]" />
                ))}
             </div>
          </div>
       </div>
       
       {/* Remaining 4 Cols */}
       {discoverData.cols.slice(2).map((col, colIdx) => (
          <div key={colIdx} className="w-1/6 flex flex-col gap-4">
             {col.map(item => (
               <GalleryItem key={item.id} item={item} onClick={onSelectImage} className="w-full h-[410px]" />
             ))}
          </div>
       ))}
    </div>
  );
};
