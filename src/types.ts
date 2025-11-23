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

export interface ThemeColors {
  primary: string;
  name: string;
}

export type ViewMode = 'chats' | 'contacts' | 'settings';
