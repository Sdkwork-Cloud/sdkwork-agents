import React from 'react';
import { cn } from '@sdkwork/agents-pc-commons';

interface ImagePreviewListProps {
  images: string[];
  isUser: boolean;
}

export const ImagePreviewList: React.FC<ImagePreviewListProps> = ({ images, isUser }) => {
  if (!images || images.length === 0) return null;

  return (
    <div className={cn(
      "flex flex-wrap gap-2 mb-2 w-full",
      isUser ? "justify-end" : "justify-start"
    )}>
      {images.map((img, i) => (
        <div key={i} className="relative rounded-2xl overflow-hidden border border-[#d9d9d9] dark:border-[#333333] bg-[#f7f7f7] dark:bg-[#282828]">
          <img src={img} alt="Uploaded content" className="max-h-64 object-contain" />
        </div>
      ))}
    </div>
  );
};
