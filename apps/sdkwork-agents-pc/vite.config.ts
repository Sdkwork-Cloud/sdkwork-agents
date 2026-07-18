import path from 'node:path';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, '.'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          const normalizedId = id.replaceAll('\\\\', '/');
          if (normalizedId.includes('/sdks/sdkwork-agents-app-sdk/')) return 'sdk-agents';
          if (normalizedId.includes('/sdkwork-iam/sdks/sdkwork-iam-app-sdk/')) return 'sdk-iam';
          if (normalizedId.includes('/sdkwork-drive/sdks/sdkwork-drive-app-sdk/')) return 'sdk-drive';
          if (normalizedId.includes('/sdkwork-knowledgebase/')) return 'sdk-knowledgebase';
          if (normalizedId.includes('/sdkwork-skills/')) return 'sdk-skills';
          if (normalizedId.includes('/sdkwork-voice/')) return 'sdk-voice';
          if (!id.includes('node_modules')) return undefined;
          return undefined;
        },
      },
    },
  },
  server: {
    host: '0.0.0.0',
    hmr: process.env.DISABLE_HMR !== 'true',
    port: 5195,
    proxy: {
      '/app/v3/api': 'http://127.0.0.1:8095',
      '/healthz': 'http://127.0.0.1:8095',
      '/livez': 'http://127.0.0.1:8095',
      '/readyz': 'http://127.0.0.1:8095',
    },
    watch: process.env.DISABLE_HMR === 'true' ? null : {},
  },
});
