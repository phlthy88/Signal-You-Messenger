import React, { useState, useEffect, useRef } from 'react';
import { ChatSession, Message } from '../types';
import { Send, Paperclip, MoreVertical, Phone, Video, Smile, Sparkles, X, ChevronDown } from 'lucide-react';
import { format } from 'date-fns';
import { generateSmartReplies, summarizeConversation } from '../services/gemini';

interface ChatWindowProps {
  chat: ChatSession;
  onSendMessage: (text: string) => void;
}

const ChatWindow: React.FC<ChatWindowProps> = ({ chat, onSendMessage }) => {
  const [inputText, setInputText] = useState('');
  const [smartReplies, setSmartReplies] = useState<string[]>([]);
  const [loadingReplies, setLoadingReplies] = useState(false);
  const [showSummary, setShowSummary] = useState(false);
  const [summary, setSummary] = useState('');
  const [loadingSummary, setLoadingSummary] = useState(false);
  
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
    // Reset AI states when chat changes
    setSmartReplies([]);
    setShowSummary(false);
    setSummary('');
  }, [chat.id]);

  const handleSend = () => {
    if (!inputText.trim()) return;
    onSendMessage(inputText);
    setInputText('');
    setSmartReplies([]);
  };

  const fetchSmartReplies = async () => {
    setLoadingReplies(true);
    const replies = await generateSmartReplies(chat.messages);
    setSmartReplies(replies);
    setLoadingReplies(false);
  };

  const handleSummarize = async () => {
    if (showSummary) {
      setShowSummary(false);
      return;
    }
    setShowSummary(true);
    setLoadingSummary(true);
    const result = await summarizeConversation(chat.messages);
    setSummary(result);
    setLoadingSummary(false);
  };

  return (
    <div className="flex flex-col h-full bg-surface text-on-surface overflow-hidden relative">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-outline-variant/20 bg-surface/90 backdrop-blur-md z-10">
        <div className="flex items-center space-x-4">
          <img src={chat.contact.avatar} alt={chat.contact.name} className="w-10 h-10 rounded-full object-cover" />
          <div>
            <h2 className="font-bold text-lg leading-tight">{chat.contact.name}</h2>
            <p className="text-xs text-on-surface-variant font-medium">Signal • Secure Message</p>
          </div>
        </div>
        
        <div className="flex items-center space-x-2">
          <button 
            onClick={handleSummarize}
            className={`p-2 rounded-full transition-colors ${showSummary ? 'bg-tertiary-container text-on-tertiary-container' : 'hover:bg-surface-variant text-on-surface-variant'}`}
            title="Summarize with AI"
          >
            <Sparkles size={20} />
          </button>
          <button className="p-2 rounded-full hover:bg-surface-variant text-on-surface-variant">
            <Video size={20} />
          </button>
          <button className="p-2 rounded-full hover:bg-surface-variant text-on-surface-variant">
            <Phone size={20} />
          </button>
          <button className="p-2 rounded-full hover:bg-surface-variant text-on-surface-variant">
            <MoreVertical size={20} />
          </button>
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
            <button onClick={() => setShowSummary(false)} className="text-on-surface-variant hover:text-on-surface">
              <X size={16} />
            </button>
          </div>
        </div>
      )}

      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <div className="flex justify-center my-4">
          <span className="bg-surface-variant/50 text-on-surface-variant text-xs px-3 py-1 rounded-full">
            {format(new Date(), 'MMMM d, yyyy')}
          </span>
        </div>
        
        {chat.messages.map((msg) => (
          <div key={msg.id} className={`flex ${msg.isMe ? 'justify-end' : 'justify-start'} group`}>
            <div 
              className={`max-w-[70%] px-4 py-3 rounded-2xl text-sm md:text-base leading-relaxed relative shadow-sm ${
                msg.isMe 
                  ? 'bg-primary text-on-primary rounded-tr-sm' 
                  : 'bg-surface-variant text-on-surface-variant rounded-tl-sm'
              }`}
            >
              {msg.content}
              <div className={`text-[10px] mt-1 text-right opacity-70 ${msg.isMe ? 'text-on-primary' : 'text-on-surface-variant'}`}>
                {format(msg.timestamp, 'HH:mm')}
                {msg.isMe && <span className="ml-1 text-lg leading-none">✓</span>}
              </div>
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>

      {/* Smart Replies Overlay */}
      {smartReplies.length > 0 && (
        <div className="absolute bottom-20 left-0 right-0 px-4 flex gap-2 overflow-x-auto pb-2 scrollbar-hide">
           {smartReplies.map((reply, idx) => (
             <button
               key={idx}
               onClick={() => {
                 onSendMessage(reply);
                 setSmartReplies([]);
               }}
               className="bg-secondary-container hover:bg-secondary-container/80 text-on-secondary-container px-4 py-2 rounded-2xl text-sm whitespace-nowrap shadow-sm border border-secondary/10 transition-transform active:scale-95"
             >
               {reply}
             </button>
           ))}
           <button onClick={() => setSmartReplies([])} className="p-2 rounded-full bg-surface-variant text-on-surface-variant">
             <X size={16} />
           </button>
        </div>
      )}

      {/* Input Area */}
      <div className="p-4 bg-surface mt-auto">
        <div className="flex items-end space-x-2 bg-surface-variant/30 rounded-[2rem] p-2 border border-outline-variant/30 focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/30 transition-all">
          <button className="p-3 text-on-surface-variant hover:text-primary transition-colors rounded-full hover:bg-surface-variant/50">
            <Smile size={24} />
          </button>
          
          <div className="flex-1 min-h-[48px] py-3">
             <textarea 
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  handleSend();
                }
              }}
              placeholder="Signal message"
              className="w-full bg-transparent border-none outline-none text-on-surface placeholder:text-on-surface-variant/50 resize-none h-6 max-h-32 overflow-y-auto"
              style={{ height: 'auto', minHeight: '24px' }}
              rows={1}
            />
          </div>

          {!inputText && (
            <button 
              onClick={fetchSmartReplies}
              disabled={loadingReplies}
              className={`p-3 transition-colors rounded-full hover:bg-surface-variant/50 ${loadingReplies ? 'text-tertiary animate-pulse' : 'text-on-surface-variant hover:text-tertiary'}`}
              title="Magic Compose"
            >
              <Sparkles size={24} />
            </button>
          )}

          {!inputText && (
            <button className="p-3 text-on-surface-variant hover:text-primary transition-colors rounded-full hover:bg-surface-variant/50">
              <Paperclip size={24} />
            </button>
          )}

          {inputText ? (
            <button 
              onClick={handleSend}
              className="p-3 bg-primary text-on-primary rounded-full hover:shadow-lg hover:bg-primary/90 transition-all active:scale-90"
            >
              <Send size={20} />
            </button>
          ) : (
            <button className="p-3 bg-surface-variant text-on-surface-variant rounded-full hover:bg-surface-variant/80 transition-all">
               <div className="w-5 h-5 border-2 border-current rounded-full flex items-center justify-center">
                 <div className="w-2 h-2 bg-current rounded-full" />
               </div>
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default ChatWindow;