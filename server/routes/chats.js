import { Router } from 'express';
import { authenticateToken } from '../middleware/auth.js';
import { createChatValidation, sendMessageValidation } from '../middleware/validation.js';
import { ChatSession } from '../models/ChatSession.js';
import { Message } from '../models/Message.js';
import { broadcastToUsers } from '../services/websocket.js';

const router = Router();

// Get all chats for current user
router.get('/', authenticateToken, (req, res) => {
  try {
    const chats = ChatSession.getUserChats(req.user.id);

    // Transform to frontend format
    const formattedChats = chats.map(chat => ({
      id: chat.id,
      contact: chat.participants.find(p => p.id !== req.user.id) || chat.participants[0],
      participants: chat.participants,
      isGroup: chat.is_group,
      name: chat.name,
      messages: [],
      unreadCount: chat.unreadCount,
      pinned: chat.pinned,
      lastMessage: chat.lastMessage ? {
        id: chat.lastMessage.id,
        senderId: chat.lastMessage.sender_id,
        content: chat.lastMessage.content,
        timestamp: chat.lastMessage.created_at,
        isMe: chat.lastMessage.sender_id === req.user.id,
        status: chat.lastMessage.status,
        type: chat.lastMessage.type
      } : null,
      createdAt: chat.created_at
    }));

    res.json({ chats: formattedChats });
  } catch (error) {
    console.error('Get chats error:', error);
    res.status(500).json({ error: 'Failed to fetch chats' });
  }
});

// Create new chat or get existing
router.post('/', authenticateToken, createChatValidation, (req, res) => {
  try {
    const { participantIds, isGroup, name } = req.body;

    // Add current user to participants
    const allParticipants = [...new Set([req.user.id, ...participantIds])];

    // Check for existing direct chat
    if (!isGroup && allParticipants.length === 2) {
      const existingChat = ChatSession.findByParticipants(allParticipants);
      if (existingChat) {
        return res.json({
          chat: formatChatResponse(existingChat, req.user.id),
          isExisting: true
        });
      }
    }

    const chat = ChatSession.create({
      participantIds: allParticipants,
      isGroup: isGroup || false,
      name
    });

    res.status(201).json({
      chat: formatChatResponse(chat, req.user.id),
      isExisting: false
    });
  } catch (error) {
    console.error('Create chat error:', error);
    res.status(500).json({ error: 'Failed to create chat' });
  }
});

// Get chat messages
router.get('/:chatId/messages', authenticateToken, (req, res) => {
  try {
    const { chatId } = req.params;
    const { limit = 50, offset = 0 } = req.query;

    // Verify user is participant
    if (!ChatSession.isParticipant(chatId, req.user.id)) {
      return res.status(403).json({ error: 'Not a participant of this chat' });
    }

    const messages = Message.getChatMessages(chatId, {
      limit: parseInt(limit),
      offset: parseInt(offset)
    });

    // Mark messages as read
    Message.markChatMessagesAsRead(chatId, req.user.id);

    // Format messages
    const formattedMessages = messages.map(m => ({
      id: m.id,
      senderId: m.sender_id,
      senderName: m.sender_name,
      senderAvatar: m.sender_avatar,
      content: m.content,
      timestamp: m.created_at,
      isMe: m.sender_id === req.user.id,
      status: m.status,
      type: m.type,
      fileUrl: m.file_url
    }));

    res.json({ messages: formattedMessages });
  } catch (error) {
    console.error('Get messages error:', error);
    res.status(500).json({ error: 'Failed to fetch messages' });
  }
});

// Send message
router.post('/:chatId/messages', authenticateToken, sendMessageValidation, (req, res) => {
  try {
    const { chatId } = req.params;
    const { content, type = 'text', fileUrl } = req.body;

    // Verify user is participant
    if (!ChatSession.isParticipant(chatId, req.user.id)) {
      return res.status(403).json({ error: 'Not a participant of this chat' });
    }

    const message = Message.create({
      chatId,
      senderId: req.user.id,
      content,
      type,
      fileUrl
    });

    const formattedMessage = {
      id: message.id,
      senderId: message.sender_id,
      senderName: message.sender_name,
      senderAvatar: message.sender_avatar,
      content: message.content,
      timestamp: message.created_at,
      isMe: true,
      status: message.status,
      type: message.type,
      fileUrl: message.file_url
    };

    // Get chat participants for WebSocket broadcast
    const chat = ChatSession.findById(chatId);
    const participantIds = chat.participants.map(p => p.id);

    // Broadcast to other participants
    broadcastToUsers(participantIds, {
      type: 'new_message',
      chatId,
      message: {
        ...formattedMessage,
        isMe: false
      }
    }, req.user.id);

    res.status(201).json({ message: formattedMessage });
  } catch (error) {
    console.error('Send message error:', error);
    res.status(500).json({ error: 'Failed to send message' });
  }
});

// Toggle pin chat
router.post('/:chatId/pin', authenticateToken, (req, res) => {
  try {
    const { chatId } = req.params;

    if (!ChatSession.isParticipant(chatId, req.user.id)) {
      return res.status(403).json({ error: 'Not a participant of this chat' });
    }

    const pinned = ChatSession.togglePin(chatId, req.user.id);
    res.json({ pinned });
  } catch (error) {
    console.error('Toggle pin error:', error);
    res.status(500).json({ error: 'Failed to toggle pin' });
  }
});

// Mark chat as read
router.post('/:chatId/read', authenticateToken, (req, res) => {
  try {
    const { chatId } = req.params;

    if (!ChatSession.isParticipant(chatId, req.user.id)) {
      return res.status(403).json({ error: 'Not a participant of this chat' });
    }

    Message.markChatMessagesAsRead(chatId, req.user.id);
    res.json({ success: true });
  } catch (error) {
    console.error('Mark read error:', error);
    res.status(500).json({ error: 'Failed to mark as read' });
  }
});

// Search messages in chat
router.get('/:chatId/search', authenticateToken, (req, res) => {
  try {
    const { chatId } = req.params;
    const { q } = req.query;

    if (!ChatSession.isParticipant(chatId, req.user.id)) {
      return res.status(403).json({ error: 'Not a participant of this chat' });
    }

    if (!q || q.length < 2) {
      return res.status(400).json({ error: 'Search query too short' });
    }

    const messages = Message.search(chatId, q);
    res.json({ messages });
  } catch (error) {
    console.error('Search error:', error);
    res.status(500).json({ error: 'Search failed' });
  }
});

function formatChatResponse(chat, userId) {
  return {
    id: chat.id,
    contact: chat.participants.find(p => p.id !== userId) || chat.participants[0],
    participants: chat.participants,
    isGroup: chat.is_group,
    name: chat.name,
    messages: [],
    unreadCount: 0,
    pinned: false,
    lastMessage: chat.lastMessage ? {
      id: chat.lastMessage.id,
      senderId: chat.lastMessage.sender_id,
      content: chat.lastMessage.content,
      timestamp: chat.lastMessage.created_at,
      isMe: chat.lastMessage.sender_id === userId,
      status: chat.lastMessage.status,
      type: chat.lastMessage.type
    } : null,
    createdAt: chat.created_at
  };
}

export default router;
