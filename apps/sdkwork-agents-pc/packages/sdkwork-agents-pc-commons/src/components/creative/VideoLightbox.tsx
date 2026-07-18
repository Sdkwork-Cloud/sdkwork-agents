import React, { useState, useEffect, useRef } from 'react';
import { Download, X, Play, Pause, Volume2, VolumeX } from 'lucide-react';
import { cn } from '../MarkdownRenderer';

interface VideoLightboxProps {
  videoUrls: string[];
  initialIndex?: number;
  onClose: () => void;
  onDownload?: (url: string, index: number) => void;
}

export const VideoLightbox: React.FC<VideoLightboxProps> = ({
  videoUrls,
  initialIndex = 0,
  onClose,
  onDownload,
}) => {
  const [activeIndex, setActiveIndex] = useState<number>(initialIndex);
  const [isPlaying, setIsPlaying] = useState<boolean>(true);
  const [isMuted, setIsMuted] = useState<boolean>(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  // Sync state if initialIndex changes
  useEffect(() => {
    setActiveIndex(initialIndex);
    setIsPlaying(true);
  }, [initialIndex]);

  // Handle keyboard shortcuts (ESC, Left, Right, Space to Play/Pause)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      } else if (e.key === ' ' || e.code === 'Space') {
        e.preventDefault();
        togglePlay();
      } else if (e.key === 'ArrowLeft' && videoUrls.length > 1) {
        setActiveIndex((prev) => (prev - 1 + videoUrls.length) % videoUrls.length);
        setIsPlaying(true);
      } else if (e.key === 'ArrowRight' && videoUrls.length > 1) {
        setActiveIndex((prev) => (prev + 1) % videoUrls.length);
        setIsPlaying(true);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [videoUrls.length, onClose]);

  const currentUrl = videoUrls[activeIndex];

  useEffect(() => {
    if (videoRef.current) {
      if (isPlaying) {
        videoRef.current.play().catch(() => setIsPlaying(false));
      } else {
        videoRef.current.pause();
      }
    }
  }, [isPlaying, currentUrl]);

  if (!currentUrl) return null;

  const togglePlay = () => {
    setIsPlaying(!isPlaying);
  };

  const toggleMute = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsMuted(!isMuted);
    if (videoRef.current) {
      videoRef.current.muted = !isMuted;
    }
  };

  const handlePrev = (e: React.MouseEvent) => {
    e.stopPropagation();
    setActiveIndex((prev) => (prev - 1 + videoUrls.length) % videoUrls.length);
    setIsPlaying(true);
  };

  const handleNext = (e: React.MouseEvent) => {
    e.stopPropagation();
    setActiveIndex((prev) => (prev + 1) % videoUrls.length);
    setIsPlaying(true);
  };

  const handleDownloadClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (onDownload) {
      onDownload(currentUrl, activeIndex);
    } else {
      const link = document.createElement('a');
      link.href = currentUrl;
      link.download = `creative-video-${activeIndex + 1}.mp4`;
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
          视频 {activeIndex + 1} / {videoUrls.length || 1}
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={toggleMute}
            className="p-2 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-300 hover:text-white transition-colors cursor-pointer"
            title={isMuted ? "取消静音" : "静音"}
          >
            {isMuted ? <VolumeX size={18} /> : <Volume2 size={18} />}
          </button>
          <button
            onClick={handleDownloadClick}
            className="p-2 rounded-lg bg-white/5 hover:bg-white/10 text-zinc-300 hover:text-white transition-colors cursor-pointer"
            title="下载视频"
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

      {/* Main Large Video Display */}
      <div className="relative max-w-[90vw] max-h-[80vh] flex items-center justify-center" onClick={(e) => e.stopPropagation()}>
        {videoUrls.length > 1 && (
          <button
            onClick={handlePrev}
            className="absolute -left-16 p-3 rounded-full bg-white/5 hover:bg-white/10 text-white transition-all hover:scale-105 active:scale-95 cursor-pointer hidden md:flex items-center justify-center border border-white/5"
            title="上一个"
          >
            <span className="text-xl">←</span>
          </button>
        )}

        <div className="relative group/player rounded-2xl overflow-hidden shadow-2xl border border-white/5 max-h-[80vh]">
          <video
            ref={videoRef}
            src={currentUrl}
            className="max-w-full max-h-[80vh] object-contain rounded-2xl"
            autoPlay
            loop
            muted={isMuted}
            onClick={togglePlay}
          />
          {/* Custom Overlay Play Button indicator when paused */}
          {!isPlaying && (
            <div 
              className="absolute inset-0 bg-black/40 flex items-center justify-center cursor-pointer transition-opacity"
              onClick={togglePlay}
            >
              <div className="w-16 h-16 rounded-full bg-white/10 backdrop-blur-md border border-white/20 flex items-center justify-center text-white scale-110 transition-transform">
                <Play size={30} fill="currentColor" className="ml-1 text-white" />
              </div>
            </div>
          )}
        </div>

        {videoUrls.length > 1 && (
          <button
            onClick={handleNext}
            className="absolute -right-16 p-3 rounded-full bg-white/5 hover:bg-white/10 text-white transition-all hover:scale-105 active:scale-95 cursor-pointer hidden md:flex items-center justify-center border border-white/5"
            title="下一个"
          >
            <span className="text-xl">→</span>
          </button>
        )}
      </div>

      {/* Thumbnails strip */}
      {videoUrls.length > 1 && (
        <div className="absolute bottom-6 flex gap-2 overflow-x-auto max-w-[90vw] p-1.5 bg-black/40 backdrop-blur-md rounded-2xl border border-white/5">
          {videoUrls.map((url, idx) => (
            <button
              key={idx}
              onClick={(e) => {
                e.stopPropagation();
                setActiveIndex(idx);
                setIsPlaying(true);
              }}
              className={cn(
                "relative w-16 h-12 rounded-lg overflow-hidden border transition-all shrink-0 cursor-pointer",
                activeIndex === idx ? "border-cyan-400 ring-2 ring-cyan-400/20 scale-95" : "border-white/10 opacity-60 hover:opacity-100"
              )}
            >
              {/* Fallback container with play indicator badge since we can't easily extract thumbnail from video on client side without loadedmetadata */}
              <div className="absolute inset-0 bg-[#161618] flex items-center justify-center text-zinc-500 hover:text-zinc-300">
                <Play size={14} fill="currentColor" />
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
