/**
 * Electron integration utilities
 * Provides a safe interface for communicating with Electron's main process
 */

// Type definitions for the Electron API exposed via preload
interface ElectronAPI {
  // App information
  getVersion: () => Promise<string>;
  getPlatform: () => Promise<{
    platform: string;
    arch: string;
    version: string;
  }>;

  // Backend configuration
  getBackendUrl: () => Promise<string>;
  getWebSocketUrl: () => Promise<string>;
  getUserDataPath: () => Promise<string>;

  // Native features
  showNotification: (options: {
    title: string;
    body: string;
    icon?: string;
  }) => Promise<boolean>;
  setBadgeCount: (count: number) => Promise<boolean>;
  openExternal: (url: string) => Promise<boolean>;

  // Event listeners
  onNewMessage: (callback: () => void) => () => void;
  onNewChat: (callback: () => void) => () => void;
  onOpenSettings: (callback: () => void) => () => void;

  // Environment flag
  isElectron: boolean;
}

// Extend the global Window interface
declare global {
  interface Window {
    electronAPI?: ElectronAPI;
  }
}

/**
 * Check if running in Electron environment
 */
export function isElectron(): boolean {
  return typeof window !== 'undefined' && !!window.electronAPI?.isElectron;
}

/**
 * Get the backend API URL
 * In Electron, this comes from the main process
 * In web, this uses environment variables or defaults
 */
export async function getApiBaseUrl(): Promise<string> {
  if (isElectron() && window.electronAPI) {
    try {
      const backendUrl = await window.electronAPI.getBackendUrl();
      return `${backendUrl}/api`;
    } catch (e) {
      console.error('Failed to get backend URL from Electron:', e);
    }
  }

  // Fallback for web or if Electron API fails
  return import.meta.env.VITE_API_URL || '/api';
}

/**
 * Get the WebSocket URL
 * In Electron, this comes from the main process
 * In web, this uses environment variables or defaults
 */
export async function getWebSocketUrl(): Promise<string> {
  if (isElectron() && window.electronAPI) {
    try {
      return await window.electronAPI.getWebSocketUrl();
    } catch (e) {
      console.error('Failed to get WebSocket URL from Electron:', e);
    }
  }

  // Fallback for web or if Electron API fails
  return import.meta.env.VITE_WS_URL || 'ws://localhost:3001/ws';
}

/**
 * Show a native notification
 * Falls back to browser notifications if not in Electron
 */
export async function showNotification(
  title: string,
  body: string,
  icon?: string
): Promise<boolean> {
  if (isElectron() && window.electronAPI) {
    return window.electronAPI.showNotification({ title, body, icon });
  }

  // Fallback to browser notifications
  if ('Notification' in window) {
    if (Notification.permission === 'granted') {
      new Notification(title, { body, icon });
      return true;
    } else if (Notification.permission !== 'denied') {
      const permission = await Notification.requestPermission();
      if (permission === 'granted') {
        new Notification(title, { body, icon });
        return true;
      }
    }
  }

  return false;
}

/**
 * Set the app badge count (dock badge on macOS, taskbar on Windows/Linux)
 */
export async function setBadgeCount(count: number): Promise<boolean> {
  if (isElectron() && window.electronAPI) {
    return window.electronAPI.setBadgeCount(count);
  }

  // Browser fallback - some browsers support navigator.setAppBadge
  if ('setAppBadge' in navigator) {
    try {
      if (count > 0) {
        await (navigator as any).setAppBadge(count);
      } else {
        await (navigator as any).clearAppBadge();
      }
      return true;
    } catch (e) {
      console.warn('Failed to set app badge:', e);
    }
  }

  return false;
}

/**
 * Open an external URL in the default browser
 */
export async function openExternal(url: string): Promise<boolean> {
  if (isElectron() && window.electronAPI) {
    return window.electronAPI.openExternal(url);
  }

  // Browser fallback
  window.open(url, '_blank', 'noopener,noreferrer');
  return true;
}

/**
 * Get app version
 */
export async function getAppVersion(): Promise<string> {
  if (isElectron() && window.electronAPI) {
    return window.electronAPI.getVersion();
  }
  return '1.0.0'; // Default version for web
}

/**
 * Get platform information
 */
export async function getPlatformInfo(): Promise<{
  platform: string;
  arch: string;
  version: string;
  isElectron: boolean;
}> {
  if (isElectron() && window.electronAPI) {
    const info = await window.electronAPI.getPlatform();
    return { ...info, isElectron: true };
  }

  return {
    platform: 'web',
    arch: 'unknown',
    version: navigator.userAgent,
    isElectron: false,
  };
}

/**
 * Subscribe to main process events
 */
export function onMainProcessEvent(
  event: 'new-message' | 'new-chat' | 'open-settings',
  callback: () => void
): () => void {
  if (isElectron() && window.electronAPI) {
    switch (event) {
      case 'new-message':
        return window.electronAPI.onNewMessage(callback);
      case 'new-chat':
        return window.electronAPI.onNewChat(callback);
      case 'open-settings':
        return window.electronAPI.onOpenSettings(callback);
    }
  }

  // Return no-op unsubscribe for non-Electron
  return () => {};
}

export default {
  isElectron,
  getApiBaseUrl,
  getWebSocketUrl,
  showNotification,
  setBadgeCount,
  openExternal,
  getAppVersion,
  getPlatformInfo,
  onMainProcessEvent,
};
