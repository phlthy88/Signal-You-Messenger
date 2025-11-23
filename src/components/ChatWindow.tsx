import React, { useState, useEffect, useRef, useCallback } from 'react';
import { Send, Paperclip, MoreVertical, Phone, Video, Smile, Sparkles, X, Pin, Trash2, Image } from 'lucide-react';
import { format, isToday, isYesterday } from 'date-fns';
import { useStore } from '../store';
import { ChatSession } from '../types';
import EmojiPicker from './EmojiPicker';
import wsService from '../services/websocket';
import api from '../services/api';

interface ChatWindowProps {
  chat: ChatSession;
}

const ChatWindow: React.FC<ChatWindowProps> = ({ chat }) => {
  const [inputText, setInputText] = useState('');
  const [smartReplies, setSmartReplies] = useState<string[]>([]);
  const [loadingReplies, setLoadingReplies] = useState(false);
  const [showSummary, setShowSummary] = useState(false);
  const [summary, setSummary] = useState('');
  const [loadingSummary, setLoadingSummary] = useState(false);
  const [showEmojiPicker, setShowEmojiPicker] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const [uploading, setUploading] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const typingTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const sendMessage = useStore((state) => state.sendMessage);
  const loadChatMessages = useStore((state) => state.loadChatMessages);
  const togglePinChat = useStore((state) => state.togglePinChat);
  const getSmartReplies = useStore((state) => state.getSmartReplies);
  const summarizeConversation = useStore((state) => state.summarizeConversation);
  const aiEnabled = useStore((state) => state.aiEnabled);
  const user = useStore((state) => state.user);

  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  // Load messages when chat changes
  useEffect(() => {
    if (chat.id) {
      loadChatMessages(chat.id);
    }
    setSmartReplies([]);
    setShowSummary(false);
    setSummary('');
    setInputText('');
  }, [chat.id, loadChatMessages]);

  // Scroll to bottom when messages change
  useEffect(() => {
    scrollToBottom();
  }, [chat.messages, scrollToBottom]);

  const handleSend = async () => {
    if (!inputText.trim()) return;

    const text = inputText;
    setInputText('');
    setSmartReplies([]);

    try {
      await sendMessage(chat.id, text);
      wsService.sendTyping(chat.id, false);
    } catch (error) {
      setInputText(text);
      alert('Failed to send message');
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInputText(e.target.value);

    // Send typing indicator
    wsService.sendTyping(chat.id, true);

    // Clear previous timeout
    if (typingTimeoutRef.current) {
      clearTimeout(typingTimeoutRef.current);
    }

    // Stop typing indicator after 2 seconds of no typing
    typingTimeoutRef.current = setTimeout(() => {
      wsService.sendTyping(chat.id, false);
    }, 2000);
  };

  const fetchSmartReplies = async () => {
    if (!aiEnabled || chat.messages.length === 0) return;

    setLoadingReplies(true);
    try {
      const replies = await getSmartReplies(chat.id);
      setSmartReplies(replies);
    } finally {
      setLoadingReplies(false);
    }
  };

  const handleSummarize = async () => {
    if (showSummary) {
      setShowSummary(false);
      return;
    }

    if (!aiEnabled || chat.messages.length === 0) return;

    setShowSummary(true);
    setLoadingSummary(true);
    try {
      const result = await summarizeConversation(chat.id);
      setSummary(result);
    } finally {
      setLoadingSummary(false);
    }
  };

  const handleEmojiSelect = (emoji: string) => {
    setInputText(prev => prev + emoji);
    setShowEmojiPicker(false);
  };

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setUploading(true);
    try {
      const { fileUrl } = await api.uploadFile(file);
      const type = file.type.startsWith('image/') ? 'image' : 'file';
      await sendMessage(chat.id, file.name, type, fileUrl);
    } catch (error) {
      alert('Failed to upload file');
    } finally {
      setUploading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  const formatMessageDate = (timestamp: string) => {
    const date = new Date(timestamp);
    if (isToday(date)) return 'Today';
    if (isYesterday(date)) return 'Yesterday';
    return format(date, 'MMMM d, yyyy');
  };

  const shouldShowDateSeparator = (index: number) => {
    if (index === 0) return true;
    const current = new Date(chat.messages[index].timestamp);
    const previous = new Date(chat.messages[index - 1].timestamp);
    return current.toDateString() !== previous.toDateString();
  };

  return (
    <div className="flex flex-col h-full bg-surface text-on-surface overflow-hidden relative">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-outline-variant/20 bg-surface/90 backdrop-blur-md z-10">
        <div className="flex items-center space-x-4">
          <img
            src={chat.contact.avatar}
            alt={chat.contact.name}
            className="w-10 h-10 rounded-full object-cover"
          />
          <div>
            <h2 className="font-bold text-lg leading-tight">{chat.contact.name}</h2>
            <p className="text-xs text-on-surface-variant font-medium">
              {chat.contact.status === 'typing'
                ? 'typing...'
                : chat.contact.status === 'online'
                ? 'Online'
                : 'Last seen recently'}
            </p>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {aiEnabled && (
            <button
              onClick={handleSummarize}
              className={`p-2 rounded-full transition-colors ${
                showSummary
                  ? 'bg-tertiary-container text-on-tertiary-container'
                  : 'hover:bg-surface-variant text-on-surface-variant'
              }`}
              title="Summarize with AI"
              aria-label="Summarize conversation"
            >
              <Sparkles size={20} />
            </button>
          )}
          <button
            className="p-2 rounded-full hover:bg-surface-variant text-on-surface-variant"
            title="Video call (coming soon)"
          >
            <Video size={20} />
          </button>
          <button
            className="p-2 rounded-full hover:bg-surface-variant text-on-surface-variant"
            title="Voice call (coming soon)"
          >
            <Phone size={20} />
          </button>
          <div className="relative">
            <button
              onClick={() => setShowMenu(!showMenu)}
              className="p-2 rounded-full hover:bg-surface-variant text-on-surface-variant"
            >
              <MoreVertical size={20} />
            </button>
            {showMenu && (
              <>
                <div className="fixed inset-0 z-40" onClick={() => setShowMenu(false)} />
                <div className="absolute right-0 top-full mt-2 bg-surface-container rounded-xl shadow-xl p-2 z-50 border border-outline-variant/20 w-48">
                  <button
                    onClick={() => {
                      togglePinChat(chat.id);
                      setShowMenu(false);
                    }}
                    className="flex items-center gap-2 w-full px-3 py-2 hover:bg-surface-variant rounded-lg text-sm text-on-surface"
                  >
                    <Pin size={16} />
                    {chat.pinned ? 'Unpin chat' : 'Pin chat'}
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      </div>

      {/* Summary Banner */}
      {showSummary && (
        <div className="bg-tertiary-container/30 border-b border-tertiary/10 p-4 animate-in slide-in-from-top-2">
          <div className="flex items-start justify-between">
            <div className="flex items-start space-x-3">
              <Sparkles className="text-tertiary mt-1" size={16} />
              <div>
                <h4 className="text-xs font-bold text-tertiary uppercase tracking-wider mb-1">AI Summary</h4>
                <p className="text-sm text-on-surface leading-relaxed">
                  {loadingSummary ? 'Analyzing conversation...' : summary}
                </p>
              </div>
            </div>
            <button
              onClick={() => setShowSummary(false)}
              className="text-on-surface-variant hover:text-on-surface"
              aria-label="Close summary"
            >
              <X size={16} />
            </button>
          </div>
        </div>
      )}

      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {chat.messages.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-on-surface-variant">
            <div className="w-16 h-16 rounded-full bg-surface-variant flex items-center justify-center mb-4">
              <Send size={24} />
            </div>
            <p className="text-lg font-medium">No messages yet</p>
            <p className="text-sm">Send a message to start the conversation</p>
          </div>
        ) : (
          chat.messages.map((msg, index) => (
            <React.Fragment key={msg.id}>
              {shouldShowDateSeparator(index) && (
                <div className="flex justify-center my-4">
                  <span className="bg-surface-variant/50 text-on-surface-variant text-xs px-3 py-1 rounded-full">
                    {formatMessageDate(msg.timestamp)}
                  </span>
                </div>
              )}
              <div className={`flex ${msg.isMe ? 'justify-end' : 'justify-start'} group`}>
                <div
                  className={`max-w-[70%] px-4 py-3 rounded-2xl text-sm md:text-base leading-relaxed relative shadow-sm ${
                    msg.isMe
                      ? 'bg-primary text-on-primary rounded-tr-sm'
                      : 'bg-surface-variant text-on-surface-variant rounded-tl-sm'
                  }`}
                >
                  {msg.type === 'image' && msg.fileUrl && (
                    <img
                      src={`http://localhost:3001${msg.fileUrl}`}
                      alt="Shared image"
                      className="rounded-lg max-w-full mb-2"
                    />
                  )}
                  {msg.type === 'file' && msg.fileUrl && (
                    <a
                      href={`http://localhost:3001${msg.fileUrl}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="flex items-center gap-2 p-2 bg-black/10 rounded-lg mb-2 hover:bg-black/20"
                    >
                      <Paperclip size={16} />
                      <span className="text-sm truncate">{msg.content}</span>
                    </a>
                  )}
                  {msg.type === 'text' && msg.content}
                  <div
                    className={`text-[10px] mt-1 text-right opacity-70 ${
                      msg.isMe ? 'text-on-primary' : 'text-on-surface-variant'
                    }`}
                  >
                    {format(new Date(msg.timestamp), 'HH:mm')}
                    {msg.isMe && (
                      <span className="ml-1">
                        {msg.status === 'read' ? '✓✓' : msg.status === 'delivered' ? '✓✓' : '✓'}
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </React.Fragment>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Smart Replies Overlay */}
      {smartReplies.length > 0 && (
        <div className="absolute bottom-20 left-0 right-0 px-4 flex gap-2 overflow-x-auto pb-2 scrollbar-hide">
          {smartReplies.map((reply, idx) => (
            <button
              key={idx}
              onClick={async () => {
                await sendMessage(chat.id, reply);
                setSmartReplies([]);
              }}
              className="bg-secondary-container hover:bg-secondary-container/80 text-on-secondary-container px-4 py-2 rounded-2xl text-sm whitespace-nowrap shadow-sm border border-secondary/10 transition-transform active:scale-95"
            >
              {reply}
            </button>
          ))}
          <button
            onClick={() => setSmartReplies([])}
            className="p-2 rounded-full bg-surface-variant text-on-surface-variant"
            aria-label="Close smart replies"
          >
            <X size={16} />
          </button>
        </div>
      )}

      {/* Emoji Picker */}
      {showEmojiPicker && (
        <div className="absolute bottom-20 left-4 z-50">
          <EmojiPicker onSelect={handleEmojiSelect} onClose={() => setShowEmojiPicker(false)} />
        </div>
      )}

      {/* Input Area */}
      <div className="p-4 bg-surface mt-auto">
        <div className="flex items-end space-x-2 bg-surface-variant/30 rounded-[2rem] p-2 border border-outline-variant/30 focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/30 transition-all">
          <button
            onClick={() => setShowEmojiPicker(!showEmojiPicker)}
            className="p-3 text-on-surface-variant hover:text-primary transition-colors rounded-full hover:bg-surface-variant/50"
            aria-label="Open emoji picker"
          >
            <Smile size={24} />
          </button>

          <div className="flex-1 min-h-[48px] py-3">
            <textarea
              value={inputText}
              onChange={handleInputChange}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  handleSend();
                }
              }}
              placeholder="Message"
              className="w-full bg-transparent border-none outline-none text-on-surface placeholder:text-on-surface-variant/50 resize-none h-6 max-h-32 overflow-y-auto"
              style={{ height: 'auto', minHeight: '24px' }}
              rows={1}
              aria-label="Message input"
            />
          </div>

          {!inputText && aiEnabled && (
            <button
              onClick={fetchSmartReplies}
              disabled={loadingReplies || chat.messages.length === 0}
              className={`p-3 transition-colors rounded-full hover:bg-surface-variant/50 ${
                loadingReplies ? 'text-tertiary animate-pulse' : 'text-on-surface-variant hover:text-tertiary'
              }`}
              title="Smart Compose"
              aria-label="Get smart replies"
            >
              <Sparkles size={24} />
            </button>
          )}

          {!inputText && (
            <>
              <input
                ref={fileInputRef}
                type="file"
                onChange={handleFileUpload}
                className="hidden"
                accept="image/*,.pdf,.doc,.docx,.txt"
              />
              <button
                onClick={() => fileInputRef.current?.click()}
                disabled={uploading}
                className="p-3 text-on-surface-variant hover:text-primary transition-colors rounded-full hover:bg-surface-variant/50"
                aria-label="Attach file"
              >
                {uploading ? (
                  <div className="animate-spin w-6 h-6 border-2 border-primary border-t-transparent rounded-full" />
                ) : (
                  <Paperclip size={24} />
                )}
              </button>
            </>
          )}

          {inputText ? (
            <button
              onClick={handleSend}
              className="p-3 bg-primary text-on-primary rounded-full hover:shadow-lg hover:bg-primary/90 transition-all active:scale-90"
              aria-label="Send message"
            >
              <Send size={20} />
            </button>
          ) : (
            <button
              onClick={() => fileInputRef.current?.click()}
              className="p-3 bg-surface-variant text-on-surface-variant rounded-full hover:bg-surface-variant/80 transition-all"
              aria-label="Send image"
            >
              <Image size={20} />
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default ChatWindow;
