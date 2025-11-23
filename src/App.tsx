import React, { useEffect } from 'react';
import { ThemeProvider } from './contexts/ThemeContext';
import { useStore, initializeWebSocketHandlers } from './store';
import Sidebar from './components/Sidebar';
import ChatList from './components/ChatList';
import ContactList from './components/ContactList';
import ChatWindow from './components/ChatWindow';
import Settings from './components/Settings';
import AuthForm from './components/AuthForm';
import ErrorBoundary from './components/ErrorBoundary';
import { User } from './types';

function App() {
  const isAuthenticated = useStore((state) => state.isAuthenticated);
  const isLoading = useStore((state) => state.isLoading);
  const checkAuth = useStore((state) => state.checkAuth);
  const currentView = useStore((state) => state.currentView);
  const setCurrentView = useStore((state) => state.setCurrentView);
  const activeChatId = useStore((state) => state.activeChatId);
  const setActiveChatId = useStore((state) => state.setActiveChatId);
  const chats = useStore((state) => state.chats);
  const contacts = useStore((state) => state.contacts);
  const createChat = useStore((state) => state.createChat);
  const loadChatMessages = useStore((state) => state.loadChatMessages);

  const activeChat = chats.find(c => c.id === activeChatId) || chats[0];

  // Check authentication on mount
  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  // Initialize WebSocket handlers when authenticated
  useEffect(() => {
    if (isAuthenticated) {
      initializeWebSocketHandlers();
    }
  }, [isAuthenticated]);

  // Load messages when active chat changes
  useEffect(() => {
    if (activeChatId) {
      loadChatMessages(activeChatId);
    }
  }, [activeChatId, loadChatMessages]);

  // Handle contact selection - create or open chat
  const handleContactSelect = async (contact: User) => {
    try {
      const chatId = await createChat(contact.id);
      setActiveChatId(chatId);
      setCurrentView('chats');
    } catch (error) {
      console.error('Failed to create chat:', error);
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 rounded-2xl bg-primary flex items-center justify-center mx-auto mb-4 shadow-lg shadow-primary/30 animate-pulse">
            <svg viewBox="0 0 24 24" fill="none" className="w-8 h-8 text-on-primary" stroke="currentColor" strokeWidth="2">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            </svg>
          </div>
          <p className="text-on-surface-variant">Loading...</p>
        </div>
      </div>
    );
  }

  // Not authenticated - show login
  if (!isAuthenticated) {
    return (
      <ThemeProvider>
        <AuthForm />
      </ThemeProvider>
    );
  }

  // Main app
  return (
    <ThemeProvider>
      <div className="flex h-screen w-screen bg-background text-on-background font-sans overflow-hidden">
        {/* Linux-style Window Controls Mock */}
        <div className="fixed top-0 right-0 p-3 z-50 flex gap-2 window-controls opacity-0 hover:opacity-100 transition-opacity">
          <div className="w-3 h-3 rounded-full bg-yellow-400 cursor-pointer hover:bg-yellow-500" />
          <div className="w-3 h-3 rounded-full bg-green-500 cursor-pointer hover:bg-green-600" />
          <div className="w-3 h-3 rounded-full bg-red-500 cursor-pointer hover:bg-red-600" />
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
              contacts={contacts}
              onSelectContact={handleContactSelect}
            />
          )}
          {currentView === 'settings' && <Settings />}

          {currentView !== 'settings' && (
            <div className="flex-1 bg-surface relative border-l border-outline-variant/10">
              {activeChat ? (
                <ChatWindow chat={activeChat} />
              ) : (
                <div className="flex items-center justify-center h-full text-on-surface-variant">
                  <div className="text-center">
                    <div className="w-16 h-16 rounded-full bg-surface-variant flex items-center justify-center mx-auto mb-4">
                      <svg viewBox="0 0 24 24" fill="none" className="w-8 h-8" stroke="currentColor" strokeWidth="2">
                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                      </svg>
                    </div>
                    <p className="text-lg font-medium">Select a chat to start messaging</p>
                    <p className="text-sm mt-1">Or start a new conversation from Contacts</p>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </ThemeProvider>
  );
}

function AppWithErrorBoundary() {
  return (
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  );
}

export default AppWithErrorBoundary;
