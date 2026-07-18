import React from 'react';

interface ToastData {
  message: string;
  type: 'success' | 'error' | 'info' | 'loading';
}

interface CanvasToastBannerProps {
  toast: ToastData | null;
  onClear: () => void;
}

export const CanvasToastBanner: React.FC<CanvasToastBannerProps> = ({ toast, onClear }) => {
  if (!toast) return null;

  return (
    <div className="absolute top-24 left-1/2 -translate-x-1/2 px-4 py-2 bg-[#121214]/95 border border-cyan-500/30 text-zinc-100 backdrop-blur-md rounded-xl flex items-center gap-2.5 text-xs font-semibold shadow-2xl z-[9999] select-none animate-in fade-in slide-in-from-top-2 duration-200">
      {toast.type === 'loading' ? (
        <span className="w-3.5 h-3.5 rounded-full border-2 border-cyan-500 border-t-transparent animate-spin" />
      ) : toast.type === 'error' ? (
        <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
      ) : (
        <span className="w-2 h-2 rounded-full bg-cyan-400 animate-pulse" />
      )}
      <span>{toast.message}</span>
      {toast.type !== 'loading' && (
        <button onClick={onClear} className="ml-1 text-zinc-500 hover:text-white transition-colors cursor-pointer text-[10px]">
          ✕
        </button>
      )}
    </div>
  );
};
