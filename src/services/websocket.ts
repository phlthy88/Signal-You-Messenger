import { api } from './api';
import { getWebSocketUrl, isElectron } from './electron';

type MessageHandler = (data: any) => void;

class WebSocketService {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private handlers: Map<string, Set<MessageHandler>> = new Map();
  private pingInterval: NodeJS.Timeout | null = null;
  private wsUrl: string | null = null;

  async connect() {
    const token = api.getToken();
    if (!token) {
      console.warn('No auth token, cannot connect to WebSocket');
      return;
    }

    // Get WebSocket URL (may be async in Electron)
    if (!this.wsUrl) {
      this.wsUrl = await getWebSocketUrl();
    }

    try {
      this.ws = new WebSocket(this.wsUrl);

      this.ws.onopen = () => {
        console.log('WebSocket connected to:', this.wsUrl);
        this.reconnectAttempts = 0;

        // Authenticate
        this.send({ type: 'auth', token });

        // Start ping interval
        this.startPing();
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          this.handleMessage(data);
        } catch (error) {
          console.error('WebSocket message parse error:', error);
        }
      };

      this.ws.onclose = () => {
        console.log('WebSocket disconnected');
        this.stopPing();
        this.attemptReconnect();
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    } catch (error) {
      console.error('WebSocket connection error:', error);
      this.attemptReconnect();
    }
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
      console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
      setTimeout(() => this.connect(), delay);
    }
  }

  private startPing() {
    this.pingInterval = setInterval(() => {
      this.send({ type: 'ping' });
    }, 30000);
  }

  private stopPing() {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }

  disconnect() {
    this.stopPing();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    // Reset URL so it will be fetched again on reconnect
    this.wsUrl = null;
  }

  send(data: object) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    }
  }

  private handleMessage(data: any) {
    const handlers = this.handlers.get(data.type);
    if (handlers) {
      handlers.forEach(handler => handler(data));
    }

    // Also call 'all' handlers
    const allHandlers = this.handlers.get('*');
    if (allHandlers) {
      allHandlers.forEach(handler => handler(data));
    }
  }

  on(type: string, handler: MessageHandler) {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type)!.add(handler);

    // Return unsubscribe function
    return () => {
      this.handlers.get(type)?.delete(handler);
    };
  }

  off(type: string, handler: MessageHandler) {
    this.handlers.get(type)?.delete(handler);
  }

  // Convenience methods
  sendTyping(chatId: string, isTyping: boolean) {
    this.send({ type: 'typing', chatId, isTyping });
  }

  sendMessageRead(chatId: string, messageId: string) {
    this.send({ type: 'message_read', chatId, messageId });
  }

  // Reset reconnection attempts (useful when connection is restored)
  resetReconnection() {
    this.reconnectAttempts = 0;
  }

  // Check connection status
  isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }
}

export const wsService = new WebSocketService();
export default wsService;
