import React, { useState, useMemo } from 'react';
import { format, isToday, isYesterday } from 'date-fns';
import { Search, Plus, Pin, Check, CheckCheck } from 'lucide-react';
import { useStore } from '../store';
import { ChatSession } from '../types';

interface ChatListProps {
  chats: ChatSession[];
  activeChatId: string | null;
  onSelectChat: (id: string) => void;
}

const ChatList: React.FC<ChatListProps> = ({ chats, activeChatId, onSelectChat }) => {
  const [searchTerm, setSearchTerm] = useState('');
  const setCurrentView = useStore((state) => state.setCurrentView);

  const filteredChats = useMemo(() => {
    if (!searchTerm.trim()) return chats;
    const term = searchTerm.toLowerCase();
    return chats.filter(chat =>
      chat.contact.name.toLowerCase().includes(term) ||
      chat.lastMessage?.content.toLowerCase().includes(term)
    );
  }, [chats, searchTerm]);

  const formatMessageTime = (timestamp: string) => {
    const date = new Date(timestamp);
    if (isToday(date)) {
      return format(date, 'HH:mm');
    }
    if (isYesterday(date)) {
      return 'Yesterday';
    }
    return format(date, 'MMM d');
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'read':
        return <CheckCheck size={14} className="text-primary" />;
      case 'delivered':
        return <CheckCheck size={14} />;
      default:
        return <Check size={14} />;
    }
  };

  return (
    <div className="flex flex-col h-full bg-surface-variant/30 border-r border-outline-variant/20 w-80 md:w-96">
      {/* Header */}
      <div className="p-4 flex items-center justify-between">
        <h2 className="text-2xl font-bold text-on-surface">Chats</h2>
        <button
          onClick={() => setCurrentView('contacts')}
          className="p-2 rounded-full bg-primary-container text-on-primary-container hover:shadow-lg transition-all"
          title="New chat"
          aria-label="Start new chat"
        >
          <Plus size={24} />
        </button>
      </div>

      {/* Search Bar */}
      <div className="px-4 pb-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-on-surface-variant/60" size={18} />
          <input
            type="text"
            placeholder="Search chats"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full bg-surface-variant/50 text-on-surface rounded-full py-3 pl-10 pr-4 outline-none focus:ring-2 focus:ring-primary/50 transition-all placeholder:text-on-surface-variant/60"
            aria-label="Search chats"
          />
        </div>
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto px-2 space-y-1">
        {filteredChats.length === 0 ? (
          <div className="text-center py-10 text-on-surface-variant/60">
            {searchTerm ? 'No chats found' : 'No conversations yet'}
          </div>
        ) : (
          filteredChats.map((chat) => (
            <button
              key={chat.id}
              onClick={() => onSelectChat(chat.id)}
              className={`w-full flex items-center p-3 rounded-[1.5rem] transition-colors ${
                activeChatId === chat.id
                  ? 'bg-secondary-container text-on-secondary-container'
                  : 'hover:bg-surface-variant/40 text-on-surface'
              }`}
            >
              <div className="relative">
                <img
                  src={chat.contact.avatar}
                  alt={chat.contact.name}
                  className="w-12 h-12 rounded-full object-cover"
                />
                {chat.contact.status === 'online' && (
                  <span className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 border-2 border-surface rounded-full" />
                )}
                {chat.contact.status === 'typing' && (
                  <span className="absolute bottom-0 right-0 w-3 h-3 bg-yellow-500 border-2 border-surface rounded-full animate-pulse" />
                )}
              </div>

              <div className="ml-4 flex-1 text-left overflow-hidden">
                <div className="flex justify-between items-center">
                  <div className="flex items-center gap-1">
                    <h3 className={`font-semibold text-base truncate ${
                      activeChatId === chat.id ? 'text-on-secondary-container' : 'text-on-surface'
                    }`}>
                      {chat.contact.name}
                    </h3>
                    {chat.pinned && <Pin size={12} className="text-primary" />}
                  </div>
                  {chat.lastMessage && (
                    <span className={`text-xs shrink-0 ${
                      activeChatId === chat.id ? 'text-on-secondary-container/70' : 'text-on-surface-variant'
                    }`}>
                      {formatMessageTime(chat.lastMessage.timestamp)}
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-1">
                  {chat.lastMessage?.isMe && (
                    <span className={activeChatId === chat.id ? 'text-on-secondary-container/70' : 'text-on-surface-variant'}>
                      {getStatusIcon(chat.lastMessage.status)}
                    </span>
                  )}
                  <p className={`text-sm truncate flex-1 ${
                    activeChatId === chat.id ? 'text-on-secondary-container/80' : 'text-on-surface-variant'
                  }`}>
                    {chat.contact.status === 'typing'
                      ? 'typing...'
                      : chat.lastMessage?.content || 'No messages yet'
                    }
                  </p>
                  {chat.unreadCount > 0 && (
                    <span className="bg-primary text-on-primary text-xs rounded-full px-2 py-0.5 min-w-[20px] text-center">
                      {chat.unreadCount}
                    </span>
                  )}
                </div>
              </div>
            </button>
          ))
        )}
      </div>
    </div>
  );
};

export default ChatList;
