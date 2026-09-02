import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    port: 8765,
    strictPort: true,
    open: process.env.TAURI_ENV_PLATFORM === undefined
  }
});
