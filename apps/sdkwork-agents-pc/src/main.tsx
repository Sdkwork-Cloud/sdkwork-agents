import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import { SdkworkI18nProvider } from '@sdkwork/i18n-pc-react';
import App from './App.tsx';
import './index.css';
import { bootstrapAgentsSdk } from './bootstrap';
import { agentsWorkbenchI18nCatalogs } from './workbench/i18n';
import { agentsI18nRuntimeConfig, resolveAgentsInitialLocale } from './i18n/runtime';

bootstrapAgentsSdk();

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <SdkworkI18nProvider
      catalogs={agentsWorkbenchI18nCatalogs}
      config={agentsI18nRuntimeConfig}
      locale={resolveAgentsInitialLocale()}
      syncDocumentLanguage
    >
      <App />
    </SdkworkI18nProvider>
  </StrictMode>,
);
