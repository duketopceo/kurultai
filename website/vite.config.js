import { defineConfig } from 'vite';

// Proxy daemon API (/api/*) and /ui to the local kurultai daemon on 8421
// so the standalone website talks to the real brain without CORS rebuilds.
export default defineConfig({
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
