import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Build stamp for the topbar so screenshots identify which build is live.
const UI_BUILD = (() => {
  try {
    return execSync('git rev-parse --short HEAD', { cwd: __dirname }).toString().trim();
  } catch {
    return new Date().toISOString().slice(0, 16);
  }
})();

export default defineConfig({
  root: '.',
  base: '/ui/',
  plugins: [react()],
  define: {
    __UI_BUILD__: JSON.stringify(UI_BUILD),
  },
  build: {
    outDir: path.resolve(__dirname, '../ui'),
    emptyOutDir: false,
    rollupOptions: {
      input: {
        brain: path.resolve(__dirname, 'brain.html'),
        'ui-next': path.resolve(__dirname, 'ui-next.html'),
      },
      output: {
        entryFileNames: 'assets/[name]-[hash].js',
        chunkFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash][extname]',
      },
    },
  },
  server: {
    port: 5174,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8421',
        changeOrigin: true,
      },
    },
  },
});
