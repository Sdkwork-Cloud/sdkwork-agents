import { createSdkworkMessageCatalog } from '@sdkwork/i18n-pc-react';
import enChat from './en-US/agents/workbench/chat.json';
import enSettings from './en-US/agents/workbench/settings.json';
import enCommon from './en-US/agents/workbench/shell.json';
import enPpt from './en-US/agents/workbench/presentation.json';
import zhChat from './zh-CN/agents/workbench/chat.json';
import zhSettings from './zh-CN/agents/workbench/settings.json';
import zhCommon from './zh-CN/agents/workbench/shell.json';
import zhPpt from './zh-CN/agents/workbench/presentation.json';

export const agentsWorkbenchChatCatalog = createSdkworkMessageCatalog({
  defaultLocale: 'en-US',
  locales: {
    'en-US': enChat,
    'zh-CN': zhChat,
  },
  namespace: 'chat',
});

export const agentsWorkbenchSettingsCatalog = createSdkworkMessageCatalog({
  defaultLocale: 'en-US',
  locales: {
    'en-US': enSettings,
    'zh-CN': zhSettings,
  },
  namespace: 'settings',
});

export const agentsWorkbenchCommonCatalog = createSdkworkMessageCatalog({
  defaultLocale: 'en-US',
  locales: {
    'en-US': enCommon,
    'zh-CN': zhCommon,
  },
  namespace: 'common',
});

export const agentsWorkbenchPresentationCatalog = createSdkworkMessageCatalog({
  defaultLocale: 'en-US',
  locales: {
    'en-US': enPpt,
    'zh-CN': zhPpt,
  },
  namespace: 'ppt',
});

export const agentsWorkbenchI18nCatalogs = Object.freeze([
  agentsWorkbenchChatCatalog,
  agentsWorkbenchSettingsCatalog,
  agentsWorkbenchCommonCatalog,
]);
