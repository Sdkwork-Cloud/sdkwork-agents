import React from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { cn } from '@sdkwork/agents-h5-commons';
import { t } from '../i18n/mobileAgentTexts';

/**
 * Mobile-first bottom action sheet and confirm dialog shared by the mobile
 * agent management views. Styled with neutral/semantic tokens so hosts with
 * light or dark themes can embed them.
 */

export interface MobileSheetOption {
  readonly label: string;
  readonly danger?: boolean;
  readonly onClick: () => void;
}

export const MobileActionSheet: React.FC<{
  isOpen: boolean;
  options: readonly MobileSheetOption[];
  onClose: () => void;
}> = ({ isOpen, options, onClose }) => {
  React.useEffect(() => {
    if (!isOpen) return undefined;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [isOpen, onClose]);

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          className="fixed inset-0 z-50 bg-black/50"
          onClick={onClose}
        >
          <motion.div
            initial={{ y: '100%' }}
            animate={{ y: 0 }}
            exit={{ y: '100%' }}
            transition={{ type: 'spring', damping: 28, stiffness: 320 }}
            className="absolute inset-x-0 bottom-0 rounded-t-2xl bg-[var(--color-chat-other-bg,#262626)] pb-[env(safe-area-inset-bottom)]"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="flex justify-center pt-2.5 pb-1">
              <div className="h-1 w-9 rounded-full bg-black/15 dark:bg-white/20" />
            </div>
            <div className="px-3 pb-4 pt-1">
              {options.map((option) => (
                <button
                  key={option.label}
                  type="button"
                  onClick={() => {
                    onClose();
                    option.onClick();
                  }}
                  className={cn(
                    'w-full rounded-xl px-4 py-3.5 text-center text-[16px] font-medium transition-colors',
                    'active:bg-black/5 dark:active:bg-white/10',
                    option.danger
                      ? 'text-red-500'
                      : 'text-gray-900 dark:text-gray-100',
                  )}
                >
                  {option.label}
                </button>
              ))}
              <button
                type="button"
                onClick={onClose}
                className="mt-1.5 w-full rounded-xl bg-black/5 dark:bg-white/10 px-4 py-3.5 text-center text-[16px] font-medium text-gray-600 dark:text-gray-300 transition-colors active:bg-black/10 dark:active:bg-white/15"
              >
                {t('agents.mobile.confirm.cancel')}
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

export const MobileConfirmDialog: React.FC<{
  isOpen: boolean;
  title: string;
  description: string;
  confirmText: string;
  cancelText: string;
  danger?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}> = ({ isOpen, title, description, confirmText, cancelText, danger, onConfirm, onCancel }) => {
  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 px-8"
          onClick={onCancel}
        >
          <motion.div
            initial={{ scale: 0.92, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.92, opacity: 0 }}
            transition={{ type: 'spring', damping: 26, stiffness: 340 }}
            className="w-full max-w-[300px] rounded-2xl bg-[var(--color-chat-other-bg,#262626)] p-5"
            onClick={(event) => event.stopPropagation()}
          >
            <h3 className="text-center text-[17px] font-semibold text-gray-900 dark:text-gray-100">
              {title}
            </h3>
            <p className="mt-2 text-center text-[14px] leading-relaxed text-gray-500 dark:text-gray-400">
              {description}
            </p>
            <div className="mt-5 flex gap-2.5">
              <button
                type="button"
                onClick={onCancel}
                className="flex-1 rounded-lg bg-black/5 dark:bg-white/10 py-2.5 text-[15px] font-medium text-gray-600 dark:text-gray-300 active:bg-black/10 dark:active:bg-white/15"
              >
                {cancelText}
              </button>
              <button
                type="button"
                onClick={onConfirm}
                className={cn(
                  'flex-1 rounded-lg py-2.5 text-[15px] font-medium text-white active:opacity-80',
                  danger ? 'bg-red-500' : 'bg-[#2b5ce7]',
                )}
              >
                {confirmText}
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
