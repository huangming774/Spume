import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  base: '/build/', // Ensure built assets use /build/ prefix
  plugins: [react()],
  build: {
    outDir: 'dist',
  },
});
