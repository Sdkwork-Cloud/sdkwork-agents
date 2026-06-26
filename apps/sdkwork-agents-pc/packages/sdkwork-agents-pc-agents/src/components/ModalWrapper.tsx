import React from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { X } from 'lucide-react';

export interface ModalWrapperProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  height?: string;
  width?: string;
  footer?: React.ReactNode;
}

export const ModalWrapper: React.FC<ModalWrapperProps> = ({ isOpen, onClose, title, children, height, width = 'w-[400px]', footer }) => {
  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          className="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm"
          onClick={onClose}
        >
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            transition={{ duration: 0.2, ease: "easeOut" }}
            onClick={e => e.stopPropagation()}
            className={`bg-[#2b2b2d] border border-white/10 rounded-xl shadow-2xl overflow-hidden flex flex-col ${width} ${height ?? 'max-h-[80vh]'} max-w-[calc(100vw-32px)] max-h-[calc(100vh-40px)]`}
          >
            <div className="flex items-center justify-between px-5 py-4 border-b border-white/5 shrink-0">
              <h3 className="text-gray-200 font-medium">{title}</h3>
              <button onClick={onClose} className="text-gray-400 hover:text-gray-200 transition-colors">
                <X size={20} />
              </button>
            </div>
            <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar p-5">
              {children}
            </div>
            {footer && (
              <div className="px-5 py-4 border-t border-white/5 bg-[#222] shrink-0 flex justify-end gap-3">
                {footer}
              </div>
            )}
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
