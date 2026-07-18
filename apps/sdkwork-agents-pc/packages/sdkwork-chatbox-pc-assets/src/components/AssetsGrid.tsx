import React from 'react';
import { Play } from 'lucide-react';
import { AssetItem } from './AssetDetailModal';

interface AssetGroup {
  date: string;
  items: AssetItem[];
}

interface AssetsGridProps {
  groups: AssetGroup[];
  activeFilter: 'image' | 'video' | 'audio' | 'document';
  onItemClick: (item: AssetItem) => void;
}

export const AssetsGrid: React.FC<AssetsGridProps> = ({ groups, activeFilter, onItemClick }) => {
  return (
    <div className="flex-1 overflow-y-auto px-6 pb-6 scrollbar-thin scrollbar-thumb-zinc-700 scrollbar-track-transparent">
      <div className="max-w-[1600px] mx-auto space-y-8">
        {groups.map((group, index) => {
          const filteredItems = group.items.filter(item => {
            if (activeFilter === 'image') return item.type === 'image';
            if (activeFilter === 'video') return item.type === 'video';
            return false;
          });

          if (filteredItems.length === 0) return null;

          return (
            <div key={index} className="space-y-4">
              <h3 className="text-sm font-semibold text-white tracking-wide">{group.date}</h3>
              <div className="grid grid-cols-[repeat(auto-fill,minmax(120px,1fr))] gap-2 2xl:grid-cols-[repeat(12,1fr)] xl:grid-cols-[repeat(10,1fr)] lg:grid-cols-[repeat(8,1fr)]">
                {filteredItems.map((item) => (
                  <div 
                    key={item.id} 
                    onClick={() => onItemClick(item)}
                    className="aspect-square relative group rounded-xl overflow-hidden bg-zinc-800 cursor-pointer border border-transparent hover:border-white/20 hover:scale-[1.02] active:scale-[0.98] transition-all duration-200"
                  >
                    <img 
                      src={item.imageUrl} 
                      alt={item.prompt} 
                      className="w-full h-full object-cover"
                      loading="lazy"
                      referrerPolicy="no-referrer"
                    />
                    
                    {item.type === 'video' && (
                      <div className="absolute inset-0 bg-black/20 flex items-center justify-center">
                        <div className="w-8 h-8 rounded-full bg-black/60 backdrop-blur-sm border border-white/10 flex items-center justify-center text-white shadow-lg group-hover:scale-110 transition-transform">
                          <Play size={12} fill="currentColor" className="ml-0.5" />
                        </div>
                      </div>
                    )}

                    <div className="absolute top-2 left-2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <div className="w-4 h-4 rounded border border-white/40 bg-black/20 flex items-center justify-center"></div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
