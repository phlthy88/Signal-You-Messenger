import React, { useState } from 'react';
import { ThemeProvider } from './contexts/ThemeContext';
import Sidebar from './components/Sidebar';
import ChatList from './components/ChatList';
import ContactList from './components/ContactList';
import ChatWindow from './components/ChatWindow';
import { ChatSession, Message, User, ViewMode } from './types';
import { v4 as uuidv4 } from 'uuid';

// Mock Users/Contacts
const MOCK_CONTACTS: User[] = [
  { id: 'u1', name: 'Sarah Connor', avatar: 'https://picsum.photos/200/200?random=1', status: 'online' },
  { id: 'u2', name: 'John Wick', avatar: 'https://picsum.photos/200/200?random=2', status: 'offline' },
  { id: 'u3', name: 'Ellen Ripley', avatar: 'https://picsum.photos/200/200?random=3', status: 'online' },
  { id: 'u4', name: 'Neo', avatar: 'https://picsum.photos/200/200?random=4', status: 'typing' },
  { id: 'u5', name: 'Trinity', avatar: 'https://picsum.photos/200/200?random=5', status: 'online' },
  { id: 'u6', name: 'Morpheus', avatar: 'https://picsum.photos/200/200?random=6', status: 'offline' },
];

// Mock Chats
const MOCK_CHATS: ChatSession[] = [
  {
    id: '1',
    contact: MOCK_CONTACTS[0],
    unreadCount: 2,
    pinned: true,
    lastMessage: { id: 'm100', senderId: 'u1', content: 'The machines are coming.', timestamp: new Date(), isMe: false, status: 'read', type: 'text' },
    messages: [
      { id: 'm1', senderId: 'me', content: 'Are you safe?', timestamp: new Date(Date.now() - 10000000), isMe: true, status: 'read', type: 'text' },
      { id: 'm2', senderId: 'u1', content: 'For now.', timestamp: new Date(Date.now() - 9000000), isMe: false, status: 'read', type: 'text' },
      { id: 'm3', senderId: 'me', content: 'We need to meet.', timestamp: new Date(Date.now() - 8000000), isMe: true, status: 'read', type: 'text' },
      { id: 'm4', senderId: 'u1', content: 'The machines are coming.', timestamp: new Date(), isMe: false, status: 'read', type: 'text' },
    ]
  },
  {
    id: '2',
    contact: MOCK_CONTACTS[1],
    unreadCount: 0,
    pinned: false,
    lastMessage: { id: 'm200', senderId: 'me', content: 'Yeah.', timestamp: new Date(Date.now() - 3600000), isMe: true, status: 'delivered', type: 'text' },
    messages: [
       { id: 'm20', senderId: 'u2', content: 'You working again?', timestamp: new Date(Date.now() - 3700000), isMe: false, status: 'read', type: 'text' },
       { id: 'm21', senderId: 'me', content: 'Yeah.', timestamp: new Date(Date.now() - 3600000), isMe: true, status: 'delivered', type: 'text' },
    ]
  },
  {
    id: '3',
    contact: MOCK_CONTACTS[2],
    unreadCount: 0,
    pinned: false,
    lastMessage: { id: 'm300', senderId: 'u3', content: 'Nuke it from orbit.', timestamp: new Date(Date.now() - 86400000), isMe: false, status: 'read', type: 'text' },
    messages: [
      { id: 'm30', senderId: 'me', content: 'What is the plan?', timestamp: new Date(Date.now() - 86500000), isMe: true, status: 'read', type: 'text' },
      { id: 'm31', senderId: 'u3', content: 'Nuke it from orbit.', timestamp: new Date(Date.now() - 86400000), isMe: false, status: 'read', type: 'text' },
    ]
  }
];

function App() {
  const [currentView, setCurrentView] = useState<ViewMode>('chats');
  const [activeChatId, setActiveChatId] = useState<string>('1');
  const [chats, setChats] = useState<ChatSession[]>(MOCK_CHATS);

  const activeChat = chats.find(c => c.id === activeChatId) || chats[0];

  const handleSendMessage = (text: string) => {
    const newMessage: Message = {
      id: uuidv4(),
      senderId: 'me',
      content: text,
      timestamp: new Date(),
      isMe: true,
      status: 'sent',
      type: 'text'
    };

    setChats(prev => prev.map(chat => {
      if (chat.id === activeChatId) {
        return {
          ...chat,
          messages: [...chat.messages, newMessage],
          lastMessage: newMessage
        };
      }
      return chat;
    }));
    
    // Simulate Signal API / Backend Reply
    if (text.toLowerCase().includes('hello') || text.toLowerCase().includes('hi')) {
       setTimeout(() => {
          const reply: Message = {
             id: uuidv4(),
             senderId: activeChat.contact.id,
             content: "Signal is secure. Hello.",
             timestamp: new Date(),
             isMe: false,
             status: 'read',
             type: 'text'
          };
          setChats(prev => prev.map(chat => {
            if (chat.id === activeChatId) {
               return { ...chat, messages: [...chat.messages, reply], lastMessage: reply };
            }
            return chat;
          }));
       }, 1500);
    }
  };

  const handleContactSelect = (contact: User) => {
    // Check if chat already exists
    const existingChat = chats.find(c => c.contact.id === contact.id);
    
    if (existingChat) {
      setActiveChatId(existingChat.id);
    } else {
      // Create new chat session
      const newChat: ChatSession = {
        id: uuidv4(),
        contact: contact,
        messages: [],
        unreadCount: 0,
        pinned: false
      };
      setChats([newChat, ...chats]);
      setActiveChatId(newChat.id);
    }
    
    // Switch view to chat
    setCurrentView('chats');
  };

  return (
    <ThemeProvider>
      <div className="flex h-screen w-screen bg-background text-on-background font-sans overflow-hidden">
         {/* Linux-style Window Controls Mock */}
         <div className="fixed top-0 right-0 p-3 z-50 flex gap-2 window-controls opacity-0 hover:opacity-100 transition-opacity">
           <div className="w-3 h-3 rounded-full bg-yellow-400 cursor-pointer hover:bg-yellow-500"></div>
           <div className="w-3 h-3 rounded-full bg-green-500 cursor-pointer hover:bg-green-600"></div>
           <div className="w-3 h-3 rounded-full bg-red-500 cursor-pointer hover:bg-red-600"></div>
         </div>

        <Sidebar activeView={currentView} onViewChange={setCurrentView} />
        
        <div className="flex flex-1 overflow-hidden border-l border-outline-variant/20 rounded-tl-3xl shadow-2xl bg-surface-container my-2 mr-2">
           {currentView === 'chats' && (
             <ChatList 
               chats={chats} 
               activeChatId={activeChatId} 
               onSelectChat={setActiveChatId} 
             />
           )}
           {currentView === 'contacts' && (
             <ContactList 
               contacts={MOCK_CONTACTS}
               onSelectContact={handleContactSelect}
             />
           )}
           
           <div className="flex-1 bg-surface relative border-l border-outline-variant/10">
              <ChatWindow 
                chat={activeChat}
                onSendMessage={handleSendMessage}
              />
           </div>
        </div>
      </div>
    </ThemeProvider>
  );
}

export default App;