import { app, BrowserWindow, shell, ipcMain, Tray, Menu, nativeImage, Notification, dialog } from 'electron';
import path from 'path';
import { fileURLToPath } from 'url';
import { spawn, fork } from 'child_process';
import { mkdirSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Environment configuration
const isDev = process.env.NODE_ENV === 'development' || !app.isPackaged;
const BACKEND_PORT = parseInt(process.env.BACKEND_PORT || '3001', 10);
const FRONTEND_PORT = 3000;

let mainWindow = null;
let tray = null;
let backendProcess = null;
let isQuitting = false;

// Single instance lock
const gotTheLock = app.requestSingleInstanceLock();

if (!gotTheLock) {
  app.quit();
} else {
  app.on('second-instance', () => {
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.focus();
    }
  });
}

/**
 * Get the path to the server directory
 */
function getServerPath() {
  if (isDev) {
    return path.join(__dirname, '..', 'server');
  }
  // In production, server is bundled in resources
  return path.join(process.resourcesPath, 'server');
}

/**
 * Get the path to the frontend dist directory
 */
function getFrontendPath() {
  if (isDev) {
    return path.join(__dirname, '..', 'dist');
  }
  return path.join(process.resourcesPath, 'dist');
}

/**
 * Start the backend server as a subprocess
 */
async function startBackend() {
  return new Promise((resolve, reject) => {
    const serverPath = getServerPath();
    const serverEntry = path.join(serverPath, 'index.js');

    console.log(`Starting backend server from: ${serverEntry}`);

    // Set up environment variables for the server
    const env = {
      ...process.env,
      PORT: BACKEND_PORT.toString(),
      NODE_ENV: isDev ? 'development' : 'production',
      // Use app data directory for database and uploads in production
      DATA_DIR: isDev
        ? path.join(serverPath, 'data')
        : path.join(app.getPath('userData'), 'data'),
      UPLOAD_DIR: isDev
        ? path.join(serverPath, 'uploads')
        : path.join(app.getPath('userData'), 'uploads'),
    };

    // Ensure data directories exist
    try {
      mkdirSync(env.DATA_DIR, { recursive: true });
      mkdirSync(env.UPLOAD_DIR, { recursive: true });
    } catch (err) {
      // Directories may already exist
    }

    backendProcess = fork(serverEntry, [], {
      cwd: serverPath,
      env,
      stdio: ['pipe', 'pipe', 'pipe', 'ipc'],
    });

    backendProcess.stdout?.on('data', (data) => {
      console.log(`[Server] ${data.toString().trim()}`);
    });

    backendProcess.stderr?.on('data', (data) => {
      console.error(`[Server Error] ${data.toString().trim()}`);
    });

    backendProcess.on('error', (err) => {
      console.error('Failed to start backend:', err);
      reject(err);
    });

    backendProcess.on('exit', (code) => {
      console.log(`Backend process exited with code ${code}`);
      if (!isQuitting && code !== 0) {
        // Attempt to restart if crashed
        setTimeout(() => startBackend(), 2000);
      }
    });

    // Wait for server to be ready
    const checkServer = async () => {
      const maxAttempts = 30;
      let attempts = 0;

      while (attempts < maxAttempts) {
        try {
          const response = await fetch(`http://localhost:${BACKEND_PORT}/api/health`);
          if (response.ok) {
            console.log('Backend server is ready');
            resolve();
            return;
          }
        } catch (e) {
          // Server not ready yet
        }
        attempts++;
        await new Promise(r => setTimeout(r, 500));
      }
      reject(new Error('Backend server failed to start'));
    };

    checkServer();
  });
}

/**
 * Stop the backend server
 */
function stopBackend() {
  if (backendProcess) {
    backendProcess.kill();
    backendProcess = null;
  }
}

/**
 * Create the main application window
 */
function createWindow() {
  // Get icon path
  const iconPath = isDev
    ? path.join(__dirname, '..', 'build', 'icons', 'icon.png')
    : path.join(process.resourcesPath, 'icons', 'icon.png');

  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 400,
    minHeight: 600,
    title: 'Signal You Messenger',
    icon: iconPath,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
      webSecurity: true,
    },
    show: false,
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'default',
    frame: process.platform !== 'darwin',
    backgroundColor: '#FFFBFE',
  });

  // Show window when ready
  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
  });

  // Load the app
  if (isDev) {
    // In development, load from Vite dev server
    mainWindow.loadURL(`http://localhost:${FRONTEND_PORT}`);
    mainWindow.webContents.openDevTools();
  } else {
    // In production, load from built files
    mainWindow.loadFile(path.join(getFrontendPath(), 'index.html'));
  }

  // Handle external links
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: 'deny' };
  });

  // Prevent navigation away from app
  mainWindow.webContents.on('will-navigate', (event, url) => {
    const appUrl = isDev
      ? `http://localhost:${FRONTEND_PORT}`
      : `file://${getFrontendPath()}`;

    if (!url.startsWith(appUrl) && !url.startsWith('http://localhost:' + BACKEND_PORT)) {
      event.preventDefault();
      shell.openExternal(url);
    }
  });

  // Handle window close
  mainWindow.on('close', (event) => {
    if (!isQuitting) {
      event.preventDefault();
      mainWindow.hide();
    }
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  return mainWindow;
}

/**
 * Create system tray icon
 */
function createTray() {
  const iconPath = isDev
    ? path.join(__dirname, '..', 'build', 'icons', 'tray-icon.png')
    : path.join(process.resourcesPath, 'icons', 'tray-icon.png');

  const icon = nativeImage.createFromPath(iconPath);
  tray = new Tray(icon.resize({ width: 16, height: 16 }));

  const contextMenu = Menu.buildFromTemplate([
    {
      label: 'Open Signal You',
      click: () => {
        if (mainWindow) {
          mainWindow.show();
        }
      },
    },
    {
      label: 'New Message',
      click: () => {
        if (mainWindow) {
          mainWindow.show();
          mainWindow.webContents.send('new-message');
        }
      },
    },
    { type: 'separator' },
    {
      label: 'Settings',
      click: () => {
        if (mainWindow) {
          mainWindow.show();
          mainWindow.webContents.send('open-settings');
        }
      },
    },
    { type: 'separator' },
    {
      label: 'Quit',
      click: () => {
        isQuitting = true;
        app.quit();
      },
    },
  ]);

  tray.setToolTip('Signal You Messenger');
  tray.setContextMenu(contextMenu);

  tray.on('click', () => {
    if (mainWindow) {
      if (mainWindow.isVisible()) {
        mainWindow.hide();
      } else {
        mainWindow.show();
      }
    }
  });
}

/**
 * Create application menu
 */
function createMenu() {
  const template = [
    {
      label: 'File',
      submenu: [
        {
          label: 'New Chat',
          accelerator: 'CmdOrCtrl+N',
          click: () => {
            mainWindow?.webContents.send('new-chat');
          },
        },
        { type: 'separator' },
        {
          label: 'Preferences',
          accelerator: 'CmdOrCtrl+,',
          click: () => {
            mainWindow?.webContents.send('open-settings');
          },
        },
        { type: 'separator' },
        {
          label: 'Quit',
          accelerator: process.platform === 'darwin' ? 'Cmd+Q' : 'Alt+F4',
          click: () => {
            isQuitting = true;
            app.quit();
          },
        },
      ],
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'selectAll' },
      ],
    },
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { type: 'separator' },
        { role: 'resetZoom' },
        { role: 'zoomIn' },
        { role: 'zoomOut' },
        { type: 'separator' },
        { role: 'togglefullscreen' },
        ...(isDev ? [
          { type: 'separator' },
          { role: 'toggleDevTools' },
        ] : []),
      ],
    },
    {
      label: 'Window',
      submenu: [
        { role: 'minimize' },
        { role: 'close' },
        ...(process.platform === 'darwin' ? [
          { type: 'separator' },
          { role: 'front' },
        ] : []),
      ],
    },
    {
      label: 'Help',
      submenu: [
        {
          label: 'About Signal You',
          click: () => {
            dialog.showMessageBox(mainWindow, {
              type: 'info',
              title: 'About Signal You Messenger',
              message: 'Signal You Messenger',
              detail: `Version ${app.getVersion()}\n\nSecure messaging with AI features.\n\nBuilt with Electron, React, and Node.js.`,
            });
          },
        },
        { type: 'separator' },
        {
          label: 'Learn More',
          click: () => {
            shell.openExternal('https://github.com/phlthy88/Signal-You-Messenger');
          },
        },
      ],
    },
  ];

  // macOS specific menu items
  if (process.platform === 'darwin') {
    template.unshift({
      label: app.name,
      submenu: [
        { role: 'about' },
        { type: 'separator' },
        { role: 'services' },
        { type: 'separator' },
        { role: 'hide' },
        { role: 'hideOthers' },
        { role: 'unhide' },
        { type: 'separator' },
        { role: 'quit' },
      ],
    });
  }

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

/**
 * Set up IPC handlers for communication between main and renderer
 */
function setupIPC() {
  // Get app version
  ipcMain.handle('get-version', () => app.getVersion());

  // Get platform info
  ipcMain.handle('get-platform', () => ({
    platform: process.platform,
    arch: process.arch,
    version: process.getSystemVersion(),
  }));

  // Show native notification
  ipcMain.handle('show-notification', (event, { title, body, icon }) => {
    if (Notification.isSupported()) {
      const notification = new Notification({
        title,
        body,
        icon,
        silent: false,
      });

      notification.on('click', () => {
        if (mainWindow) {
          mainWindow.show();
          mainWindow.focus();
        }
      });

      notification.show();
      return true;
    }
    return false;
  });

  // Set badge count (macOS/Linux)
  ipcMain.handle('set-badge-count', (event, count) => {
    if (process.platform === 'darwin' || process.platform === 'linux') {
      app.setBadgeCount(count);
    }
    return true;
  });

  // Open external URL
  ipcMain.handle('open-external', (event, url) => {
    shell.openExternal(url);
    return true;
  });

  // Get user data path
  ipcMain.handle('get-user-data-path', () => app.getPath('userData'));

  // Get backend URL
  ipcMain.handle('get-backend-url', () => `http://localhost:${BACKEND_PORT}`);

  // Get WebSocket URL
  ipcMain.handle('get-websocket-url', () => `ws://localhost:${BACKEND_PORT}/ws`);
}

// App lifecycle events
app.on('ready', async () => {
  try {
    // Start backend first
    await startBackend();

    // Create window and UI
    createWindow();
    createTray();
    createMenu();
    setupIPC();

    console.log('Signal You Messenger started successfully');
  } catch (error) {
    console.error('Failed to start application:', error);
    dialog.showErrorBox(
      'Startup Error',
      `Failed to start Signal You Messenger:\n\n${error.message}`
    );
    app.quit();
  }
});

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow();
  } else {
    mainWindow.show();
  }
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    isQuitting = true;
    app.quit();
  }
});

app.on('before-quit', () => {
  isQuitting = true;
  stopBackend();
});

app.on('quit', () => {
  stopBackend();
});

// Handle certificate errors (for development)
if (isDev) {
  app.on('certificate-error', (event, webContents, url, error, certificate, callback) => {
    if (url.startsWith('https://localhost')) {
      event.preventDefault();
      callback(true);
    } else {
      callback(false);
    }
  });
}
