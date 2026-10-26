import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    port: 8765,
    strictPort: true,
    open: process.env.TAURI_ENV_PLATFORM === undefined,
    // Only frontend sources matter to the dev server; crawling Rust builds and agent artifacts made it stall
    watch: { ignored: ['**/target/**', '**/.agents/**', '**/src/**', '**/wiki/**', '**/tickets/**', '**/gen/**'] }
  }
});
