import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    port: 3000,
    open: process.env.TAURI_ENV_PLATFORM === undefined
  }
});
