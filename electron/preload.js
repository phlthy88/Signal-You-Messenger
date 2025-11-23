/**
 * Preload script for Signal You Messenger
 * This script runs in a privileged context and exposes a safe API
 * to the renderer process through contextBridge.
 */

import { contextBridge, ipcRenderer } from 'electron';

// Expose protected methods to the renderer process
contextBridge.exposeInMainWorld('electronAPI', {
  // App information
  getVersion: () => ipcRenderer.invoke('get-version'),
  getPlatform: () => ipcRenderer.invoke('get-platform'),

  // Backend configuration
  getBackendUrl: () => ipcRenderer.invoke('get-backend-url'),
  getWebSocketUrl: () => ipcRenderer.invoke('get-websocket-url'),
  getUserDataPath: () => ipcRenderer.invoke('get-user-data-path'),

  // Native features
  showNotification: (options) => ipcRenderer.invoke('show-notification', options),
  setBadgeCount: (count) => ipcRenderer.invoke('set-badge-count', count),
  openExternal: (url) => ipcRenderer.invoke('open-external', url),

  // Event listeners from main process
  onNewMessage: (callback) => {
    const subscription = (event) => callback();
    ipcRenderer.on('new-message', subscription);
    return () => ipcRenderer.removeListener('new-message', subscription);
  },

  onNewChat: (callback) => {
    const subscription = (event) => callback();
    ipcRenderer.on('new-chat', subscription);
    return () => ipcRenderer.removeListener('new-chat', subscription);
  },

  onOpenSettings: (callback) => {
    const subscription = (event) => callback();
    ipcRenderer.on('open-settings', subscription);
    return () => ipcRenderer.removeListener('open-settings', subscription);
  },

  // Check if running in Electron
  isElectron: true,
});

// Add platform-specific class to document
window.addEventListener('DOMContentLoaded', () => {
  const platform = process.platform;
  document.body.classList.add(`platform-${platform}`);
  document.body.classList.add('electron-app');
});
