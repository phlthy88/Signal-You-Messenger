import { getApiBaseUrl, isElectron } from './electron';

// API base URL - will be set dynamically
let API_BASE = import.meta.env.VITE_API_URL || '/api';

// Initialize API URL (call this on app startup)
export async function initializeApi(): Promise<void> {
  API_BASE = await getApiBaseUrl();
  console.log('API initialized with base URL:', API_BASE);
}

interface ApiError {
  error: string;
  details?: { field: string; message: string }[];
}

class ApiClient {
  private token: string | null = null;
  private initialized = false;

  constructor() {
    // Try to restore token from localStorage
    this.token = localStorage.getItem('auth_token');
  }

  async ensureInitialized(): Promise<void> {
    if (!this.initialized) {
      await initializeApi();
      this.initialized = true;
    }
  }

  setToken(token: string | null) {
    this.token = token;
    if (token) {
      localStorage.setItem('auth_token', token);
    } else {
      localStorage.removeItem('auth_token');
    }
  }

  getToken() {
    return this.token;
  }

  getBaseUrl() {
    return API_BASE;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    await this.ensureInitialized();

    const url = `${API_BASE}${endpoint}`;

    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.token) {
      (headers as Record<string, string>)['Authorization'] = `Bearer ${this.token}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    const data = await response.json();

    if (!response.ok) {
      const error = data as ApiError;
      throw new Error(error.error || 'Request failed');
    }

    return data as T;
  }

  // Auth endpoints
  async register(email: string, password: string, name: string) {
    const data = await this.request<{ user: User; token: string }>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, name }),
    });
    this.setToken(data.token);
    return data;
  }

  async login(email: string, password: string) {
    const data = await this.request<{ user: User; token: string }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password }),
    });
    this.setToken(data.token);
    return data;
  }

  async logout() {
    try {
      await this.request('/auth/logout', { method: 'POST' });
    } finally {
      this.setToken(null);
    }
  }

  async getMe() {
    return this.request<{ user: User }>('/auth/me');
  }

  async updateProfile(data: { name?: string; avatar?: string }) {
    return this.request<{ user: User }>('/auth/profile', {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  // Chats endpoints
  async getChats() {
    return this.request<{ chats: ChatSession[] }>('/chats');
  }

  async createChat(participantIds: string[], isGroup = false, name?: string) {
    return this.request<{ chat: ChatSession; isExisting: boolean }>('/chats', {
      method: 'POST',
      body: JSON.stringify({ participantIds, isGroup, name }),
    });
  }

  async getChatMessages(chatId: string, limit = 50, offset = 0) {
    return this.request<{ messages: Message[] }>(
      `/chats/${chatId}/messages?limit=${limit}&offset=${offset}`
    );
  }

  async sendMessage(chatId: string, content: string, type: 'text' | 'image' | 'file' = 'text', fileUrl?: string) {
    return this.request<{ message: Message }>(`/chats/${chatId}/messages`, {
      method: 'POST',
      body: JSON.stringify({ content, type, fileUrl }),
    });
  }

  async togglePinChat(chatId: string) {
    return this.request<{ pinned: boolean }>(`/chats/${chatId}/pin`, {
      method: 'POST',
    });
  }

  async markChatAsRead(chatId: string) {
    return this.request<{ success: boolean }>(`/chats/${chatId}/read`, {
      method: 'POST',
    });
  }

  async searchMessages(chatId: string, query: string) {
    return this.request<{ messages: Message[] }>(`/chats/${chatId}/search?q=${encodeURIComponent(query)}`);
  }

  // Contacts endpoints
  async getContacts() {
    return this.request<{ contacts: User[] }>('/contacts');
  }

  async addContact(contactId: string) {
    return this.request<{ contact: User }>('/contacts', {
      method: 'POST',
      body: JSON.stringify({ contactId }),
    });
  }

  async removeContact(contactId: string) {
    return this.request<{ success: boolean }>(`/contacts/${contactId}`, {
      method: 'DELETE',
    });
  }

  async searchUsers(query: string) {
    return this.request<{ users: (User & { isContact: boolean })[] }>(
      `/contacts/search?q=${encodeURIComponent(query)}`
    );
  }

  async getAllUsers() {
    return this.request<{ users: (User & { isContact: boolean })[] }>('/contacts/all-users');
  }

  // AI endpoints
  async getAIStatus() {
    return this.request<{ enabled: boolean }>('/ai/status');
  }

  async getSmartReplies(chatId: string) {
    return this.request<{ replies: string[] }>('/ai/smart-replies', {
      method: 'POST',
      body: JSON.stringify({ chatId }),
    });
  }

  async summarizeConversation(chatId: string) {
    return this.request<{ summary: string }>('/ai/summarize', {
      method: 'POST',
      body: JSON.stringify({ chatId }),
    });
  }

  // Settings endpoints
  async getSettings() {
    return this.request<{ settings: UserSettings }>('/settings');
  }

  async updateSettings(settings: Partial<UserSettings>) {
    return this.request<{ settings: UserSettings }>('/settings', {
      method: 'PATCH',
      body: JSON.stringify(settings),
    });
  }

  // Upload endpoints
  async uploadFile(file: File) {
    await this.ensureInitialized();

    const formData = new FormData();
    formData.append('file', file);

    const url = `${API_BASE}/upload`;
    const response = await fetch(url, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${this.token}`,
      },
      body: formData,
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.error || 'Upload failed');
    }

    return response.json() as Promise<{
      fileUrl: string;
      filename: string;
      originalName: string;
      mimetype: string;
      size: number;
    }>;
  }

  // Health check
  async healthCheck() {
    return this.request<{ status: string; timestamp: string }>('/health');
  }
}

// Types
export interface User {
  id: string;
  email?: string;
  name: string;
  avatar: string;
  status: 'online' | 'offline' | 'typing';
  lastSeen?: string;
}

export interface Message {
  id: string;
  senderId: string;
  senderName?: string;
  senderAvatar?: string;
  content: string;
  timestamp: string;
  isMe: boolean;
  status: 'sent' | 'delivered' | 'read';
  type: 'text' | 'image' | 'file';
  fileUrl?: string;
}

export interface ChatSession {
  id: string;
  contact: User;
  participants?: User[];
  isGroup?: boolean;
  name?: string;
  messages: Message[];
  unreadCount: number;
  lastMessage?: Message;
  pinned: boolean;
  createdAt?: string;
}

export interface UserSettings {
  theme: string;
  notificationsEnabled: boolean;
  soundEnabled: boolean;
}

export const api = new ApiClient();
export default api;
