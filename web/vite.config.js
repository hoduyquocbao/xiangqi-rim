// Cấu hình đóng gói Vite cho ứng dụng React Web App Cờ Tướng Hoàng Gia (XiangRust)
// Định danh 100% đơn từ tiếng Anh, chú thích 100% tiếng Việt
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    host: '0.0.0.0',
    allowedHosts: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8888',
        changeOrigin: true
      },
      '/ws': {
        target: 'ws://127.0.0.1:8888',
        ws: true,
        changeOrigin: true
      }
    }
  },
  build: {
    target: 'esnext',
    outDir: 'dist'
  }
});
