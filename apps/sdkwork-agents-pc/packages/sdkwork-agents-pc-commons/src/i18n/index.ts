import { createInstance } from 'i18next';
import { createElement, type ReactNode } from 'react';
import { I18nextProvider, initReactI18next } from 'react-i18next';
import enChat from './en-US/agents/workbench/chat.json';
import enSettings from './en-US/agents/workbench/settings.json';
import enCommon from './en-US/agents/workbench/shell.json';
import enPpt from './en-US/agents/workbench/presentation.json';
import zhChat from './zh-CN/agents/workbench/chat.json';
import zhSettings from './zh-CN/agents/workbench/settings.json';
import zhCommon from './zh-CN/agents/workbench/shell.json';
import zhPpt from './zh-CN/agents/workbench/presentation.json';

const agentsWorkbenchI18n = createInstance();

void agentsWorkbenchI18n
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
    initAsync: false,
    interpolation: {
      escapeValue: false // react already safes from xss
    }
  });

export function AgentsWorkbenchI18nProvider({ children }: { children: ReactNode }) {
  return createElement(I18nextProvider, { i18n: agentsWorkbenchI18n }, children);
}

export default agentsWorkbenchI18n;
