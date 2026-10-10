import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: './e2e', workers: 1, timeout: 60000,
  use: { baseURL: 'http://127.0.0.1:5173', browserName: 'chromium', trace: 'retain-on-failure', screenshot: 'only-on-failure' },
  webServer: [
    { command: 'python ../scripts/test-server.py', url: 'http://127.0.0.1:8080/api/health', timeout: 30000, reuseExistingServer: false },
    { command: 'npm run dev -- --port 5173 --strictPort', url: 'http://127.0.0.1:5173', timeout: 30000, reuseExistingServer: false },
  ],
});
