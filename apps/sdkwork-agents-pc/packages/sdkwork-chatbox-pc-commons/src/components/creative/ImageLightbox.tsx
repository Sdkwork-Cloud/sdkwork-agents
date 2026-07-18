import React, { useState, useEffect } from 'react';
import { Download, X } from 'lucide-react';
import { cn } from '../MarkdownRenderer';

interface ImageLightboxProps {
  imageUrls: string[];
  initialIndex?: number;
  onClose: () => void;
  onDownload?: (url: string, index: number) => void;
}

export const ImageLightbox: React.FC<ImageLightboxProps> = ({
  imageUrls,
  initialIndex = 0,
  onClose,
  onDownload,
}) => {
  const [activeIndex, setActiveIndex] = useState<number>(initialIndex);

  // Sync state if initialIndex changes
  useEffect(() => {
    setActiveIndex(initialIndex);
  }, [initialIndex]);

  // Handle keyboard shortcuts (ESC, Left, Right)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === 'ArrowLeft' && imageUrls.length > 1) {
        setActiveIndex((prev) => (prev - 1 + imageUrls.length) % imageUrls.length);
      } else if (e.key === 'ArrowRight' && imageUrls.length > 1) {
        setActiveIndex((prev) => (prev + 1) % imageUrls.length);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [imageUrls.length, onClose]);

  const currentUrl = imageUrls[activeIndex];

  if (!currentUrl) return null;

  const handlePrev = (e: React.MouseEvent) => {
    e.stopPropagation();
    setActiveIndex((prev) => (prev - 1 + imageUrls.length) % imageUrls.length);
  };

  const handleNext = (e: React.MouseEvent) => {
    e.stopPropagation();
    setActiveIndex((prev) => (prev + 1) % imageUrls.length);
  };

  const handleDownloadClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onDownload) {
      onDownload(currentUrl, activeIndex);
    } else {
      // Default browser fallback download if no custom handler is provided
      const link = document.createElement('a');
      link.href = currentUrl;
      link.download = `creative-image-${activeIndex + 1}.png`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
  };

  return (
    <div 
      className="fixed inset-0 z-[999] bg-black/95 flex flex-col items-center justify-center animate-in fade-in duration-200 select-none"
      onClick={onClose}
    >
      {/* Top Panel Actions */}
      <div className="absolute top-0 inset-x-0 h-16 flex items-center justify-between px-6 bg-gradient-to-b from-black/60 to-transparent z-10">
        <div className="text-zinc-400 text-sm font-medium font-mono">
          {activeIndex + 1} / {imageUrls.length || 1}
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={handleDownloadClick}
            className="p-2 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-300 hover:text-white transition-colors cursor-pointer"
            title="下载图片"
          >
            <Download size={18} />
          </button>
          <button
            onClick={onClose}
            className="p-2 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-300 hover:text-white text-sm transition-colors cursor-pointer font-medium flex items-center gap-1"
          >
            <X size={16} />
            <span>关闭 ESC</span>
          </button>
        </div>
      </div>

      {/* Main Large Image Display */}
      <div className="relative max-w-[90vw] max-h-[80vh] flex items-center justify-center group/img">
        {imageUrls.length > 1 && (
          <button
            onClick={handlePrev}
            className="absolute -left-16 p-3 rounded-full bg-white/5 hover:bg-white/10 text-white transition-all hover:scale-105 active:scale-95 cursor-pointer hidden md:flex items-center justify-center border border-white/5"
            title="上一张"
          >
            <span className="text-xl">←</span>
          </button>
        )}

        <div className="relative rounded-2xl overflow-hidden border border-white/5 shadow-2xl group">
          <img
            src={currentUrl}
            className="max-w-full max-h-[80vh] object-contain animate-in zoom-in-95 duration-200"
            alt=""
            onClick={(e) => e.stopPropagation()}
            referrerPolicy="no-referrer"
          />
          {/* Download Floating Action Button on hover of the image */}
          <button
            onClick={handleDownloadClick}
            className="absolute bottom-4 right-4 flex items-center gap-2 px-4 py-2 rounded-xl bg-black/75 hover:bg-black text-white hover:text-cyan-400 border border-white/10 shadow-lg cursor-pointer transition-all opacity-0 group-hover/img:opacity-100 hover:scale-105 backdrop-blur-md z-30"
            title="下载当前图片"
          >
            <Download size={16} />
            <span className="text-xs font-semibold">保存到设备</span>
          </button>
        </div>

        {imageUrls.length > 1 && (
          <button
            onClick={handleNext}
            className="absolute -right-16 p-3 rounded-full bg-white/5 hover:bg-white/10 text-white transition-all hover:scale-105 active:scale-95 cursor-pointer hidden md:flex items-center justify-center border border-white/5"
            title="下一张"
          >
            <span className="text-xl">→</span>
          </button>
        )}
      </div>

      {/* Image Thumbnails Strip inside Lightbox */}
      {imageUrls.length > 1 && (
        <div className="absolute bottom-6 flex gap-2 overflow-x-auto max-w-[90vw] p-1.5 bg-black/40 backdrop-blur-md rounded-2xl border border-white/5">
          {imageUrls.map((url, idx) => (
            <button
              key={idx}
              onClick={(e) => {
                e.stopPropagation();
                setActiveIndex(idx);
              }}
              className={cn(
                "relative w-16 h-12 rounded-lg overflow-hidden border transition-all shrink-0 cursor-pointer",
                activeIndex === idx ? "border-cyan-400 ring-2 ring-cyan-400/20 scale-95" : "border-white/10 opacity-60 hover:opacity-100"
              )}
            >
              <img src={url} className="absolute inset-0 w-full h-full object-cover" alt="" referrerPolicy="no-referrer" />
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
