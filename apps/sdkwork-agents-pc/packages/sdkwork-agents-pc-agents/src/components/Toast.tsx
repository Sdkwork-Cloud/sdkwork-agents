import React, { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'motion/react';

type ToastType = 'info' | 'success' | 'error';

interface ToastOptions {
  placement?: 'top' | 'bottom-right';
}

type ToastPlacement = NonNullable<ToastOptions['placement']>;

interface ToastItem {
  id: string;
  message: string;
  placement: ToastPlacement;
  type: ToastType;
}

let addToast: (message: string, type?: ToastType, options?: ToastOptions) => void;

function toastClassName(type: ToastType): string {
  return `px-4 py-2 rounded-full shadow-lg text-sm font-medium text-white flex items-center gap-2 ${
    type === 'success' ? 'bg-[#00b42a]' :
    type === 'error' ? 'bg-red-500' :
    'bg-[#2b2b2d] border border-white/10'
  }`;
}

function renderToast(toast: ToastItem, placement: ToastPlacement): React.ReactNode {
  return (
    <motion.div
      key={toast.id}
      initial={{ opacity: 0, y: placement === 'bottom-right' ? 20 : -20, scale: 0.9 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: placement === 'bottom-right' ? 20 : -20, scale: 0.9 }}
      className={toastClassName(toast.type)}
    >
      {toast.message}
    </motion.div>
  );
}

export const ToastContainer: React.FC = () => {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  useEffect(() => {
    addToast = (message, type = 'info', options = {}) => {
      const id = Date.now().toString();
      setToasts(prev => [...prev, { id, message, placement: options.placement ?? 'top', type }]);
      setTimeout(() => {
        setToasts(prev => prev.filter(t => t.id !== id));
      }, 3000);
    };
  }, []);

  const topToasts = toasts.filter((toast) => toast.placement === 'top');
  const bottomRightToasts = toasts.filter((toast) => toast.placement === 'bottom-right');

  return (
    <>
      <div className="fixed top-10 left-1/2 -translate-x-1/2 z-[9999] flex flex-col gap-2 pointer-events-none">
        <AnimatePresence>
          {topToasts.map((toast) => renderToast(toast, 'top'))}
        </AnimatePresence>
      </div>
      <div className="fixed bottom-6 right-6 z-[9999] flex max-w-[320px] flex-col items-end gap-2 pointer-events-none">
        <AnimatePresence>
          {bottomRightToasts.map((toast) => renderToast(toast, 'bottom-right'))}
        </AnimatePresence>
      </div>
    </>
  );
};

export const toast = (message: string, type: ToastType = 'info', options?: ToastOptions) => {
  if (addToast) {
    addToast(message, type, options);
  } else {
    console.log(`[Toast] ${type}: ${message}`);
  }
};
