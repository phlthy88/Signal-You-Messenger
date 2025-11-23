import express from 'express';
import cors from 'cors';
import { createServer } from 'http';
import path from 'path';
import { fileURLToPath } from 'url';

import config, { validateConfig } from './config/env.js';
import { initializeDatabase } from './config/database.js';
import { initializeWebSocket } from './services/websocket.js';

// Routes
import authRoutes from './routes/auth.js';
import chatRoutes from './routes/chats.js';
import contactRoutes from './routes/contacts.js';
import aiRoutes from './routes/ai.js';
import settingsRoutes from './routes/settings.js';
import uploadRoutes from './routes/upload.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Validate configuration
validateConfig();

// Initialize database
initializeDatabase();

// Create Express app
const app = express();
const server = createServer(app);

// Initialize WebSocket
initializeWebSocket(server);

// Middleware
app.use(cors({
  origin: config.CORS_ORIGIN,
  credentials: true
}));
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Serve uploaded files
app.use('/uploads', express.static(path.join(__dirname, 'uploads')));

// API Routes
app.use('/api/auth', authRoutes);
app.use('/api/chats', chatRoutes);
app.use('/api/contacts', contactRoutes);
app.use('/api/ai', aiRoutes);
app.use('/api/settings', settingsRoutes);
app.use('/api/upload', uploadRoutes);

// Health check
app.get('/api/health', (req, res) => {
  res.json({
    status: 'ok',
    timestamp: new Date().toISOString(),
    version: '1.0.0'
  });
});

// Error handling middleware
app.use((err, req, res, next) => {
  console.error('Unhandled error:', err);
  res.status(500).json({
    error: config.NODE_ENV === 'development' ? err.message : 'Internal server error'
  });
});

// 404 handler
app.use((req, res) => {
  res.status(404).json({ error: 'Not found' });
});

// Start server
server.listen(config.PORT, () => {
  console.log(`
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   Signal You Messenger - Backend Server                   ║
║                                                           ║
║   Server running on port ${config.PORT}                          ║
║   WebSocket available at ws://localhost:${config.PORT}/ws        ║
║   Environment: ${config.NODE_ENV.padEnd(10)}                         ║
║   AI Enabled: ${config.GEMINI_API_KEY ? 'Yes' : 'No '}                                 ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
  `);
});

export default app;
