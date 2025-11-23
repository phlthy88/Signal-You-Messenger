import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import api, { User, ChatSession, Message, UserSettings } from '../services/api';
import wsService from '../services/websocket';

export type ViewMode = 'chats' | 'contacts' | 'settings';

interface AppState {
  // Auth state
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // View state
  currentView: ViewMode;
  activeChatId: string | null;

  // Data state
  chats: ChatSession[];
  contacts: User[];
  settings: UserSettings;

  // AI state
  aiEnabled: boolean;

  // Actions
  setCurrentView: (view: ViewMode) => void;
  setActiveChatId: (id: string | null) => void;
  setError: (error: string | null) => void;

  // Auth actions
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string, name: string) => Promise<void>;
  logout: () => Promise<void>;
  checkAuth: () => Promise<void>;

  // Chat actions
  loadChats: () => Promise<void>;
  loadChatMessages: (chatId: string) => Promise<void>;
  sendMessage: (chatId: string, content: string, type?: 'text' | 'image' | 'file', fileUrl?: string) => Promise<void>;
  createChat: (contactId: string) => Promise<string>;
  togglePinChat: (chatId: string) => Promise<void>;
  markChatAsRead: (chatId: string) => Promise<void>;

  // Contact actions
  loadContacts: () => Promise<void>;
  addContact: (contactId: string) => Promise<void>;
  removeContact: (contactId: string) => Promise<void>;
  searchUsers: (query: string) => Promise<(User & { isContact: boolean })[]>;

  // Settings actions
  loadSettings: () => Promise<void>;
  updateSettings: (settings: Partial<UserSettings>) => Promise<void>;

  // AI actions
  checkAIStatus: () => Promise<void>;
  getSmartReplies: (chatId: string) => Promise<string[]>;
  summarizeConversation: (chatId: string) => Promise<string>;

  // WebSocket handlers
  handleNewMessage: (data: any) => void;
  handleTyping: (data: any) => void;
  handleUserStatus: (data: any) => void;
}

export const useStore = create<AppState>()(
  persist(
    (set, get) => ({
      // Initial state
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

      // Basic actions
      setCurrentView: (view) => set({ currentView: view }),
      setActiveChatId: (id) => {
        set({ activeChatId: id });
        if (id) {
          get().markChatAsRead(id);
        }
      },
      setError: (error) => set({ error }),

      // Auth actions
      login: async (email, password) => {
        set({ isLoading: true, error: null });
        try {
          const { user } = await api.login(email, password);
          set({ user, isAuthenticated: true, isLoading: false });
          wsService.connect();
          await Promise.all([
            get().loadChats(),
            get().loadContacts(),
            get().loadSettings(),
            get().checkAIStatus(),
          ]);
        } catch (error) {
          set({ error: (error as Error).message, isLoading: false });
          throw error;
        }
      },

      register: async (email, password, name) => {
        set({ isLoading: true, error: null });
        try {
          const { user } = await api.register(email, password, name);
          set({ user, isAuthenticated: true, isLoading: false });
          wsService.connect();
          await get().loadSettings();
          await get().checkAIStatus();
        } catch (error) {
          set({ error: (error as Error).message, isLoading: false });
          throw error;
        }
      },

      logout: async () => {
        try {
          await api.logout();
        } finally {
          wsService.disconnect();
          set({
            user: null,
            isAuthenticated: false,
            chats: [],
            contacts: [],
            activeChatId: null,
            currentView: 'chats',
          });
        }
      },

      checkAuth: async () => {
        const token = api.getToken();
        if (!token) {
          set({ isLoading: false, isAuthenticated: false });
          return;
        }

        try {
          const { user } = await api.getMe();
          set({ user, isAuthenticated: true, isLoading: false });
          wsService.connect();
          await Promise.all([
            get().loadChats(),
            get().loadContacts(),
            get().loadSettings(),
            get().checkAIStatus(),
          ]);
        } catch (error) {
          api.setToken(null);
          set({ isLoading: false, isAuthenticated: false });
        }
      },

      // Chat actions
      loadChats: async () => {
        try {
          const { chats } = await api.getChats();
          set({ chats });
        } catch (error) {
          console.error('Failed to load chats:', error);
        }
      },

      loadChatMessages: async (chatId) => {
        try {
          const { messages } = await api.getChatMessages(chatId);
          set((state) => ({
            chats: state.chats.map((chat) =>
              chat.id === chatId ? { ...chat, messages } : chat
            ),
          }));
        } catch (error) {
          console.error('Failed to load messages:', error);
        }
      },

      sendMessage: async (chatId, content, type = 'text', fileUrl) => {
        try {
          const { message } = await api.sendMessage(chatId, content, type, fileUrl);
          set((state) => ({
            chats: state.chats.map((chat) =>
              chat.id === chatId
                ? {
                    ...chat,
                    messages: [...chat.messages, message],
                    lastMessage: message,
                  }
                : chat
            ),
          }));
        } catch (error) {
          console.error('Failed to send message:', error);
          throw error;
        }
      },

      createChat: async (contactId) => {
        try {
          const { chat, isExisting } = await api.createChat([contactId]);
          if (!isExisting) {
            set((state) => ({ chats: [chat, ...state.chats] }));
          }
          return chat.id;
        } catch (error) {
          console.error('Failed to create chat:', error);
          throw error;
        }
      },

      togglePinChat: async (chatId) => {
        try {
          const { pinned } = await api.togglePinChat(chatId);
          set((state) => ({
            chats: state.chats.map((chat) =>
              chat.id === chatId ? { ...chat, pinned } : chat
            ),
          }));
        } catch (error) {
          console.error('Failed to toggle pin:', error);
        }
      },

      markChatAsRead: async (chatId) => {
        try {
          await api.markChatAsRead(chatId);
          set((state) => ({
            chats: state.chats.map((chat) =>
              chat.id === chatId ? { ...chat, unreadCount: 0 } : chat
            ),
          }));
        } catch (error) {
          console.error('Failed to mark as read:', error);
        }
      },

      // Contact actions
      loadContacts: async () => {
        try {
          const { contacts } = await api.getContacts();
          set({ contacts });
        } catch (error) {
          console.error('Failed to load contacts:', error);
        }
      },

      addContact: async (contactId) => {
        try {
          const { contact } = await api.addContact(contactId);
          set((state) => ({ contacts: [...state.contacts, contact] }));
        } catch (error) {
          console.error('Failed to add contact:', error);
          throw error;
        }
      },

      removeContact: async (contactId) => {
        try {
          await api.removeContact(contactId);
          set((state) => ({
            contacts: state.contacts.filter((c) => c.id !== contactId),
          }));
        } catch (error) {
          console.error('Failed to remove contact:', error);
          throw error;
        }
      },

      searchUsers: async (query) => {
        try {
          const { users } = await api.searchUsers(query);
          return users;
        } catch (error) {
          console.error('Failed to search users:', error);
          return [];
        }
      },

      // Settings actions
      loadSettings: async () => {
        try {
          const { settings } = await api.getSettings();
          set({ settings });
        } catch (error) {
          console.error('Failed to load settings:', error);
        }
      },

      updateSettings: async (newSettings) => {
        try {
          const { settings } = await api.updateSettings(newSettings);
          set({ settings });
        } catch (error) {
          console.error('Failed to update settings:', error);
          throw error;
        }
      },

      // AI actions
      checkAIStatus: async () => {
        try {
          const { enabled } = await api.getAIStatus();
          set({ aiEnabled: enabled });
        } catch (error) {
          set({ aiEnabled: false });
        }
      },

      getSmartReplies: async (chatId) => {
        try {
          const { replies } = await api.getSmartReplies(chatId);
          return replies;
        } catch (error) {
          console.error('Failed to get smart replies:', error);
          return [];
        }
      },

      summarizeConversation: async (chatId) => {
        try {
          const { summary } = await api.summarizeConversation(chatId);
          return summary;
        } catch (error) {
          console.error('Failed to summarize:', error);
          return 'Failed to generate summary.';
        }
      },

      // WebSocket handlers
      handleNewMessage: (data) => {
        const { chatId, message } = data;
        set((state) => {
          const chatExists = state.chats.some((c) => c.id === chatId);
          if (!chatExists) {
            // Reload chats to get the new one
            get().loadChats();
            return state;
          }

          return {
            chats: state.chats.map((chat) =>
              chat.id === chatId
                ? {
                    ...chat,
                    messages: [...chat.messages, message],
                    lastMessage: message,
                    unreadCount:
                      state.activeChatId === chatId
                        ? chat.unreadCount
                        : chat.unreadCount + 1,
                  }
                : chat
            ),
          };
        });
      },

      handleTyping: (data) => {
        const { userId, chatId, isTyping } = data;
        set((state) => ({
          chats: state.chats.map((chat) =>
            chat.id === chatId
              ? {
                  ...chat,
                  contact: {
                    ...chat.contact,
                    status: isTyping ? 'typing' : 'online',
                  },
                }
              : chat
          ),
        }));
      },

      handleUserStatus: (data) => {
        const { userId, status } = data;
        set((state) => ({
          contacts: state.contacts.map((contact) =>
            contact.id === userId ? { ...contact, status } : contact
          ),
          chats: state.chats.map((chat) =>
            chat.contact.id === userId
              ? { ...chat, contact: { ...chat.contact, status } }
              : chat
          ),
        }));
      },
    }),
    {
      name: 'signal-messenger-storage',
      partialize: (state) => ({
        settings: state.settings,
      }),
    }
  )
);

// Initialize WebSocket handlers
export function initializeWebSocketHandlers() {
  const store = useStore.getState();

  wsService.on('new_message', store.handleNewMessage);
  wsService.on('typing', store.handleTyping);
  wsService.on('user_status', store.handleUserStatus);
  wsService.on('auth_success', () => {
    console.log('WebSocket authenticated');
  });
}

export default useStore;
