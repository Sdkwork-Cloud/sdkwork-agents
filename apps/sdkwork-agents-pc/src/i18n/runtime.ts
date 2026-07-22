import {
  defineSdkworkI18nRuntimeConfig,
  normalizeSdkworkLocale,
} from '@sdkwork/i18n-pc-react';

export const agentsI18nRuntimeConfig = defineSdkworkI18nRuntimeConfig({
  activeLocales: ['en-US', 'zh-CN'],
  defaultLocale: 'en-US',
  fallbackLocale: 'en-US',
  loadingStrategy: 'eager-core-lazy-feature',
  supportedLocales: ['en-US', 'zh-CN'],
});

export function resolveAgentsInitialLocale(): string {
  if (typeof window === 'undefined') {
    return agentsI18nRuntimeConfig.defaultLocale;
  }

  const explicitLocale = window.localStorage.getItem('user_explicit_lang');
  if (explicitLocale) {
    return normalizeSdkworkLocale(explicitLocale, agentsI18nRuntimeConfig);
  }

  const browserLocales = [window.navigator.language, ...(window.navigator.languages ?? [])];
  for (const browserLocale of browserLocales) {
    const resolvedLocale = normalizeSdkworkLocale(browserLocale, agentsI18nRuntimeConfig);
    if (resolvedLocale !== agentsI18nRuntimeConfig.defaultLocale || browserLocale.toLowerCase().startsWith('en')) {
      return resolvedLocale;
    }
  }

  return agentsI18nRuntimeConfig.defaultLocale;
}
