import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
const base = process.env.RESERVOIR_BASE_PATH || '/';
export default defineConfig({
  base,
  plugins: [react()],
  server: { host: '127.0.0.1', proxy: { [base + 'api']: { target:'http://127.0.0.1:8080', rewrite:path => '/api' + path.slice((base + 'api').length), cookiePathRewrite:{'/api':base + 'api'} } } },
  build: { rollupOptions: { output: { manualChunks: id => id.includes('@fullcalendar') ? 'calendar' : id.includes('/node_modules/luxon/') ? 'timezone' : undefined } } },
});
