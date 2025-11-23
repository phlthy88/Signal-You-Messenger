import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the API before importing store
vi.mock('../../services/api', () => ({
  default: {
    getToken: vi.fn(() => null),
    setToken: vi.fn(),
    login: vi.fn(),
    register: vi.fn(),
    logout: vi.fn(),
    getMe: vi.fn(),
    getChats: vi.fn(),
    getChatMessages: vi.fn(),
    sendMessage: vi.fn(),
    createChat: vi.fn(),
    getContacts: vi.fn(),
    getSettings: vi.fn(),
    getAIStatus: vi.fn(),
  },
  api: {
    getToken: vi.fn(() => null),
    setToken: vi.fn(),
  },
}));

vi.mock('../../services/websocket', () => ({
  default: {
    connect: vi.fn(),
    disconnect: vi.fn(),
    on: vi.fn(),
    off: vi.fn(),
  },
  wsService: {
    connect: vi.fn(),
    disconnect: vi.fn(),
    on: vi.fn(),
    off: vi.fn(),
  },
}));

import { useStore } from '../../store';
import api from '../../services/api';

describe('Store', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset store state
    useStore.setState({
      user: null,
      isAuthenticated: false,
      isLoading: true,
      error: null,
      currentView: 'chats',
      activeChatId: null,
      chats: [],
      contacts: [],
      settings: {
        theme: 'Deep Purple',
        notificationsEnabled: true,
        soundEnabled: true,
      },
      aiEnabled: false,
    });
  });

  describe('View state', () => {
    it('sets current view', () => {
      const { setCurrentView } = useStore.getState();

      setCurrentView('contacts');

      expect(useStore.getState().currentView).toBe('contacts');
    });

    it('sets active chat id', () => {
      const { setActiveChatId } = useStore.getState();

      setActiveChatId('chat-123');

      expect(useStore.getState().activeChatId).toBe('chat-123');
    });
  });

  describe('Auth state', () => {
    it('login sets user and authenticated state', async () => {
      const mockUser = { id: '1', name: 'Test User', email: 'test@example.com', avatar: '', status: 'online' as const };
      (api.login as any).mockResolvedValue({ user: mockUser, token: 'test-token' });
      (api.getChats as any).mockResolvedValue({ chats: [] });
      (api.getContacts as any).mockResolvedValue({ contacts: [] });
      (api.getSettings as any).mockResolvedValue({ settings: {} });
      (api.getAIStatus as any).mockResolvedValue({ enabled: false });

      const { login } = useStore.getState();
      await login('test@example.com', 'password');

      const state = useStore.getState();
      expect(state.user).toEqual(mockUser);
      expect(state.isAuthenticated).toBe(true);
      expect(state.isLoading).toBe(false);
    });

    it('logout clears user and state', async () => {
      // Set up authenticated state
      useStore.setState({
        user: { id: '1', name: 'Test', email: '', avatar: '', status: 'online' },
        isAuthenticated: true,
        chats: [{ id: '1', contact: {} as any, messages: [], unreadCount: 0, pinned: false }],
      });

      (api.logout as any).mockResolvedValue({});

      const { logout } = useStore.getState();
      await logout();

      const state = useStore.getState();
      expect(state.user).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      expect(state.chats).toEqual([]);
    });
  });

  describe('WebSocket handlers', () => {
    it('handleNewMessage adds message to chat', () => {
      useStore.setState({
        chats: [{
          id: 'chat-1',
          contact: { id: 'u1', name: 'Test', avatar: '', status: 'online' },
          messages: [],
          unreadCount: 0,
          pinned: false,
        }],
        activeChatId: null,
      });

      const { handleNewMessage } = useStore.getState();
      handleNewMessage({
        chatId: 'chat-1',
        message: { id: 'm1', content: 'Hello', senderId: 'u1', isMe: false, timestamp: new Date().toISOString(), status: 'sent', type: 'text' },
      });

      const state = useStore.getState();
      expect(state.chats[0].messages).toHaveLength(1);
      expect(state.chats[0].unreadCount).toBe(1);
    });

    it('handleUserStatus updates contact status', () => {
      useStore.setState({
        contacts: [{ id: 'u1', name: 'Test', avatar: '', status: 'offline' }],
        chats: [],
      });

      const { handleUserStatus } = useStore.getState();
      handleUserStatus({ userId: 'u1', status: 'online' });

      const state = useStore.getState();
      expect(state.contacts[0].status).toBe('online');
    });
  });
});
