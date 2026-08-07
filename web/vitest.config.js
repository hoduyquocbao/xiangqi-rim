// Cấu hình Vitest cho dự án Web Xiangqi AI Debugger
// Định danh đơn từ tiếng Anh: react, config, test, globals, environment, setupFiles, testTimeout, hookTimeout, teardownTimeout
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: [],
    // Thiết lập thời gian timeout 15000ms tránh gián đoạn kiểm thử JSDOM
    testTimeout: 15000,
    hookTimeout: 15000,
    teardownTimeout: 15000,
  },
});

