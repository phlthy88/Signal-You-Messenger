import { Router } from 'express';
import { authenticateToken } from '../middleware/auth.js';
import { ChatSession } from '../models/ChatSession.js';
import { Message } from '../models/Message.js';
import { generateSmartReplies, summarizeConversation, isAIEnabled } from '../services/gemini.js';

const router = Router();

// Get AI status
router.get('/status', authenticateToken, (req, res) => {
  res.json({ enabled: isAIEnabled() });
});

// Generate smart replies
router.post('/smart-replies', authenticateToken, async (req, res) => {
  try {
    const { chatId } = req.body;

    if (!isAIEnabled()) {
      return res.status(503).json({ error: 'AI service not configured' });
    }

    if (!chatId) {
      return res.status(400).json({ error: 'Chat ID required' });
    }

    // Verify user is participant
    if (!ChatSession.isParticipant(chatId, req.user.id)) {
      return res.status(403).json({ error: 'Not a participant of this chat' });
    }

    // Get recent messages for context
    const messages = Message.getRecentForAI(chatId, 10);

    if (messages.length === 0) {
      return res.json({ replies: [] });
    }

    const result = await generateSmartReplies(messages, req.user.id);

    if (result.error) {
      return res.status(500).json({ error: result.error });
    }

    res.json({ replies: result.replies });
  } catch (error) {
    console.error('Smart replies error:', error);
    res.status(500).json({ error: 'Failed to generate smart replies' });
  }
});

// Summarize conversation
router.post('/summarize', authenticateToken, async (req, res) => {
  try {
    const { chatId } = req.body;

    if (!isAIEnabled()) {
      return res.status(503).json({ error: 'AI service not configured' });
    }

    if (!chatId) {
      return res.status(400).json({ error: 'Chat ID required' });
    }

    // Verify user is participant
    if (!ChatSession.isParticipant(chatId, req.user.id)) {
      return res.status(403).json({ error: 'Not a participant of this chat' });
    }

    // Get all messages for summary
    const messages = Message.getChatMessages(chatId, { limit: 100 });

    if (messages.length === 0) {
      return res.json({ summary: 'No messages to summarize.' });
    }

    const result = await summarizeConversation(messages, req.user.id);

    if (result.error) {
      return res.status(500).json({ error: result.error });
    }

    res.json({ summary: result.summary });
  } catch (error) {
    console.error('Summarize error:', error);
    res.status(500).json({ error: 'Failed to generate summary' });
  }
});

export default router;
