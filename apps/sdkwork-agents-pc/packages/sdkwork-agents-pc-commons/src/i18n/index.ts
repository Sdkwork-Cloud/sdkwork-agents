import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import enChat from './en-US/agents/workbench/chat.json';
import enSettings from './en-US/agents/workbench/settings.json';
import enCommon from './en-US/agents/workbench/shell.json';
import enPpt from './en-US/agents/workbench/presentation.json';
import zhChat from './zh-CN/agents/workbench/chat.json';
import zhSettings from './zh-CN/agents/workbench/settings.json';
import zhCommon from './zh-CN/agents/workbench/shell.json';
import zhPpt from './zh-CN/agents/workbench/presentation.json';

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
