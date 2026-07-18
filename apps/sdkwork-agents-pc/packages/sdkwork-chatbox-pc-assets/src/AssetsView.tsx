import React, { useState, useEffect } from 'react';
import { HelpCircle } from 'lucide-react';
import { AssetDetailModal, AssetItem } from './components/AssetDetailModal';
import { AssetsHeader } from './components/AssetsHeader';
import { AssetsFilter } from './components/AssetsFilter';
import { AssetsGrid } from './components/AssetsGrid';
import { AssetsService } from '@/packages/sdkwork-chatbox-pc-core/src/services/AssetsService';

export const AssetsView = () => {
  const [activeTab, setActiveTab] = useState<'history' | 'subject' | 'canvas'>('history');
  const [activeFilter, setActiveFilter] = useState<'image' | 'video' | 'audio' | 'document'>('image');
  
  const [groups, setGroups] = useState<{ date: string; items: AssetItem[] }[]>([]);

  useEffect(() => {
    AssetsService.getAssetGroups().then(setGroups);
  }, []);

  // Preview states
  const [selectedItem, setSelectedItem] = useState<AssetItem | null>(null);
  const [isDetailOpen, setIsDetailOpen] = useState<boolean>(false);

  // Flattened items for seamless navigation
  const allItems = groups.flatMap(g => g.items);
  const currentIndex = selectedItem ? allItems.findIndex(i => i.id === selectedItem.id) : -1;
  const hasPrev = currentIndex > 0;
  const hasNext = currentIndex < allItems.length - 1;

  const handlePrev = () => {
    if (hasPrev) {
      setSelectedItem(allItems[currentIndex - 1]);
    }
  };

  const handleNext = () => {
    if (hasNext) {
      setSelectedItem(allItems[currentIndex + 1]);
    }
  };

  const handleItemClick = (item: AssetItem) => {
    setSelectedItem(item);
    setIsDetailOpen(true);
  };

  return (
    <div className="flex flex-col h-full w-full bg-[#18181A] text-gray-200">
      <AssetsHeader activeTab={activeTab} setActiveTab={setActiveTab} />
      <AssetsFilter activeFilter={activeFilter} setActiveFilter={setActiveFilter} />
      
      <AssetsGrid 
        groups={groups} 
        activeFilter={activeFilter} 
        onItemClick={handleItemClick} 
      />

      {/* Floating Help Button */}
      <div className="absolute right-6 bottom-6">
        <button className="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center text-zinc-400 hover:text-white transition-colors backdrop-blur-sm">
          <HelpCircle size={16} />
        </button>
      </div>

      {/* High-Fidelity Asset Detail Modal */}
      <AssetDetailModal
        isOpen={isDetailOpen}
        onClose={() => setIsDetailOpen(false)}
        currentItem={selectedItem}
        onPrev={handlePrev}
        onNext={handleNext}
        hasPrev={hasPrev}
        hasNext={hasNext}
      />
    </div>
  );
};
