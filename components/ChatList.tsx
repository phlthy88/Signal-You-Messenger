import React from 'react';
import { ChatSession } from '../types';
import { format } from 'date-fns';
import { Search, Plus } from 'lucide-react';

interface ChatListProps {
  chats: ChatSession[];
  activeChatId: string | null;
  onSelectChat: (id: string) => void;
}

const ChatList: React.FC<ChatListProps> = ({ chats, activeChatId, onSelectChat }) => {
  return (
    <div className="flex flex-col h-full bg-surface-variant/30 border-r border-outline-variant/20 w-80 md:w-96">
      {/* Header */}
      <div className="p-4 flex items-center justify-between">
        <h2 className="text-2xl font-bold text-on-surface">Chats</h2>
        <button className="p-2 rounded-full bg-primary-container text-on-primary-container hover:shadow-lg transition-all">
          <Plus size={24} />
        </button>
      </div>

      {/* Search Bar */}
      <div className="px-4 pb-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-on-surface-variant/60" size={18} />
          <input 
            type="text" 
            placeholder="Search" 
            className="w-full bg-surface-variant/50 text-on-surface rounded-full py-3 pl-10 pr-4 outline-none focus:ring-2 focus:ring-primary/50 transition-all placeholder:text-on-surface-variant/60"
          />
        </div>
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto px-2 space-y-1">
        {chats.map((chat) => (
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
                <span className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 border-2 border-surface rounded-full"></span>
              )}
            </div>
            
            <div className="ml-4 flex-1 text-left overflow-hidden">
              <div className="flex justify-between items-baseline">
                <h3 className={`font-semibold text-base truncate ${activeChatId === chat.id ? 'text-on-secondary-container' : 'text-on-surface'}`}>
                  {chat.contact.name}
                </h3>
                {chat.lastMessage && (
                  <span className={`text-xs ${activeChatId === chat.id ? 'text-on-secondary-container/70' : 'text-on-surface-variant'}`}>
                    {format(chat.lastMessage.timestamp, 'HH:mm')}
                  </span>
                )}
              </div>
              <p className={`text-sm truncate ${activeChatId === chat.id ? 'text-on-secondary-container/80' : 'text-on-surface-variant'}`}>
                 {chat.lastMessage?.isMe ? 'You: ' : ''}{chat.lastMessage?.content || 'No messages yet'}
              </p>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
};

export default ChatList;