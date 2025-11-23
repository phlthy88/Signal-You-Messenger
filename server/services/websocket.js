import { WebSocketServer } from 'ws';
import jwt from 'jsonwebtoken';
import config from '../config/env.js';
import { User } from '../models/User.js';

const clients = new Map(); // userId -> Set of WebSocket connections

export function initializeWebSocket(server) {
  const wss = new WebSocketServer({ server, path: '/ws' });

  wss.on('connection', (ws, req) => {
    let userId = null;

    ws.on('message', async (data) => {
      try {
        const message = JSON.parse(data.toString());

        switch (message.type) {
          case 'auth':
            // Authenticate the WebSocket connection
            try {
              const decoded = jwt.verify(message.token, config.JWT_SECRET);
              userId = decoded.userId;

              // Add to clients map
              if (!clients.has(userId)) {
                clients.set(userId, new Set());
              }
              clients.get(userId).add(ws);

              // Update user status to online
              User.updateStatus(userId, 'online');

              // Notify contacts that user is online
              broadcastUserStatus(userId, 'online');

              ws.send(JSON.stringify({ type: 'auth_success', userId }));
            } catch (err) {
              ws.send(JSON.stringify({ type: 'auth_error', error: 'Invalid token' }));
            }
            break;

          case 'typing':
            // Broadcast typing status to chat participants
            if (userId && message.chatId) {
              broadcastToChat(message.chatId, {
                type: 'typing',
                userId,
                chatId: message.chatId,
                isTyping: message.isTyping
              }, userId);
            }
            break;

          case 'message_read':
            // Broadcast read receipts
            if (userId && message.chatId) {
              broadcastToChat(message.chatId, {
                type: 'message_read',
                userId,
                chatId: message.chatId,
                messageId: message.messageId
              }, userId);
            }
            break;

          case 'ping':
            ws.send(JSON.stringify({ type: 'pong' }));
            break;
        }
      } catch (error) {
        console.error('WebSocket message error:', error);
      }
    });

    ws.on('close', () => {
      if (userId) {
        const userConnections = clients.get(userId);
        if (userConnections) {
          userConnections.delete(ws);
          if (userConnections.size === 0) {
            clients.delete(userId);
            // Update user status to offline
            User.updateStatus(userId, 'offline');
            broadcastUserStatus(userId, 'offline');
          }
        }
      }
    });

    ws.on('error', (error) => {
      console.error('WebSocket error:', error);
    });
  });

  return wss;
}

// Send message to specific user
export function sendToUser(userId, message) {
  const userConnections = clients.get(userId);
  if (userConnections) {
    const payload = JSON.stringify(message);
    userConnections.forEach(ws => {
      if (ws.readyState === 1) { // OPEN
        ws.send(payload);
      }
    });
  }
}

// Broadcast to all participants in a chat
export function broadcastToChat(chatId, message, excludeUserId = null) {
  // This requires getting chat participants - done at the route level
  // For now, this is a placeholder that will be called with participant IDs
}

// Broadcast message to multiple users
export function broadcastToUsers(userIds, message, excludeUserId = null) {
  const payload = JSON.stringify(message);
  userIds.forEach(userId => {
    if (userId !== excludeUserId) {
      const userConnections = clients.get(userId);
      if (userConnections) {
        userConnections.forEach(ws => {
          if (ws.readyState === 1) {
            ws.send(payload);
          }
        });
      }
    }
  });
}

// Broadcast user status change
function broadcastUserStatus(userId, status) {
  const message = { type: 'user_status', userId, status };
  // Broadcast to all connected clients
  clients.forEach((connections, clientUserId) => {
    if (clientUserId !== userId) {
      const payload = JSON.stringify(message);
      connections.forEach(ws => {
        if (ws.readyState === 1) {
          ws.send(payload);
        }
      });
    }
  });
}

// Get online status
export function isUserOnline(userId) {
  return clients.has(userId) && clients.get(userId).size > 0;
}

// Get all online users
export function getOnlineUsers() {
  return Array.from(clients.keys());
}
