export interface User {
  id: string;
  name: string;
  avatar: string;
  status: 'online' | 'offline' | 'typing';
}

export interface Message {
  id: string;
  senderId: string;
  content: string;
  timestamp: Date;
  isMe: boolean;
  status: 'sent' | 'delivered' | 'read';
  type: 'text' | 'image' | 'file';
  fileUrl?: string;
}

export interface ChatSession {
  id: string;
  contact: User;
  messages: Message[];
  unreadCount: number;
  lastMessage?: Message;
  pinned: boolean;
}

export interface ThemeColors {
  primary: string;
  name: string;
}

export enum GeminiModel {
  FLASH = 'gemini-2.5-flash',
  PRO = 'gemini-3-pro-preview'
}

export type ViewMode = 'chats' | 'contacts' | 'settings';