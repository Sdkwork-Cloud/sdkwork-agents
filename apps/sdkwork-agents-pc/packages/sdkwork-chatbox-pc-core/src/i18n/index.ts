import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import enChat from './en-US/agents/chatbox/chat.json';
import enSettings from './en-US/agents/chatbox/settings.json';
import enCommon from './en-US/agents/chatbox/shell.json';
import enPpt from './en-US/agents/chatbox/presentation.json';
import zhChat from './zh-CN/agents/chatbox/chat.json';
import zhSettings from './zh-CN/agents/chatbox/settings.json';
import zhCommon from './zh-CN/agents/chatbox/shell.json';
import zhPpt from './zh-CN/agents/chatbox/presentation.json';

i18n
  .use(initReactI18next)
  .init({
    resources: {
      'en-US': {
        chat: enChat,
        settings: enSettings,
        common: enCommon,
        ppt: enPpt,
      },
      'zh-CN': {
        chat: zhChat,
        settings: zhSettings,
        common: zhCommon,
        ppt: zhPpt,
      }
    },
    lng: 'en-US',
    fallbackLng: 'en-US',
    interpolation: {
      escapeValue: false // react already safes from xss
    }
  });

export default i18n;
