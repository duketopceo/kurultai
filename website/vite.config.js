import { defineConfig } from 'vite';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Optional preview of ../ui — product UI is daemon GET /ui (embedded from ui/).
export default defineConfig({
  root: path.resolve(__dirname, '../ui'),
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
