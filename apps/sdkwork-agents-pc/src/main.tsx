import {StrictMode} from 'react';
import {createRoot} from 'react-dom/client';
import App from './App.tsx';
import './index.css';
import '@sdkwork/agents-pc-commons/i18n';
import { bootstrapAgentsSdk } from './bootstrap';

bootstrapAgentsSdk();

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
