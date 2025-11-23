import path from 'path';
import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', '');
  const isElectron = process.env.ELECTRON === 'true';
  const backendPort = process.env.BACKEND_PORT || '3001';

  return {
    // Base path for Electron production builds
    base: isElectron ? './' : '/',

    server: {
      port: 3000,
      host: '0.0.0.0',
      proxy: {
        '/api': {
          target: `http://localhost:${backendPort}`,
          changeOrigin: true,
        },
        '/ws': {
          target: `ws://localhost:${backendPort}`,
          ws: true,
        },
        '/uploads': {
          target: `http://localhost:${backendPort}`,
          changeOrigin: true,
        },
      },
    },

    plugins: [
      react(),
    ],

    define: {
      // Electron environment flag
      'import.meta.env.ELECTRON': JSON.stringify(isElectron),
      'import.meta.env.BACKEND_PORT': JSON.stringify(backendPort),
    },

    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
      },
    },

    build: {
      outDir: 'dist',
      sourcemap: mode !== 'production',
      // Ensure assets are loaded correctly in Electron
      assetsDir: 'assets',
      rollupOptions: {
        output: {
          manualChunks: {
            vendor: ['react', 'react-dom'],
            store: ['zustand'],
          },
        },
      },
    },

    // Optimize deps for Electron
    optimizeDeps: {
      exclude: ['electron'],
    },
  };
});
