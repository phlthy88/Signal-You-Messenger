import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import { isElectron, onMainProcessEvent, getAppVersion, getPlatformInfo } from './services/electron';

// Initialize app
async function initializeApp() {
  // Log platform info
  const platformInfo = await getPlatformInfo();
  const version = await getAppVersion();

  console.log(`Signal You Messenger v${version}`);
  console.log('Platform:', platformInfo.platform, platformInfo.arch);
  console.log('Running in Electron:', platformInfo.isElectron);

  // Set up CSS class for platform-specific styling
  if (platformInfo.isElectron) {
    document.body.classList.add('electron-app');
    document.body.classList.add(`platform-${platformInfo.platform}`);
  }

  // Mount React app
  const rootElement = document.getElementById('root');
  if (!rootElement) {
    throw new Error('Could not find root element to mount to');
  }

  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );

  // Register service worker for PWA (only in web/browser, not in Electron)
  if (!isElectron() && 'serviceWorker' in navigator && import.meta.env.PROD) {
    window.addEventListener('load', () => {
      navigator.serviceWorker.register('/sw.js').catch(console.error);
    });
  }
}

// Run initialization
initializeApp().catch(console.error);
