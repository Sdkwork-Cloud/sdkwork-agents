import React from 'react';
import { useTranslation } from 'react-i18next';

export const TypingIndicator: React.FC = () => {
  const { t } = useTranslation('common');
  
  return (
    <div className="flex items-center space-x-1.5 py-3 px-4 bg-[#f4f4f4] dark:bg-[#2a2a2a] rounded-2xl rounded-tl-sm w-fit border border-[#ebebeb] dark:border-[#333] shadow-sm mt-1">
      <div className="flex space-x-1 items-center justify-center">
        <div className="w-1.5 h-1.5 bg-gray-400 dark:bg-gray-500 rounded-full animate-bounce [animation-delay:-0.3s]"></div>
        <div className="w-1.5 h-1.5 bg-gray-400 dark:bg-gray-500 rounded-full animate-bounce [animation-delay:-0.15s]"></div>
        <div className="w-1.5 h-1.5 bg-gray-400 dark:bg-gray-500 rounded-full animate-bounce"></div>
      </div>
      <span className="ml-3 text-[13px] text-gray-500 dark:text-gray-400 font-medium ml-2">
        {t('typing') || 'Typing...'}
      </span>
    </div>
  );
};
