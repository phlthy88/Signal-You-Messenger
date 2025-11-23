import db from '../config/database.js';
import { v4 as uuidv4 } from 'uuid';

export class ChatSession {
  static create({ participantIds, isGroup = false, name = null }) {
    const id = uuidv4();

    const transaction = db.transaction(() => {
      // Create chat session
      db.prepare(`
        INSERT INTO chat_sessions (id, is_group, name)
        VALUES (?, ?, ?)
      `).run(id, isGroup ? 1 : 0, name);

      // Add participants
      const insertParticipant = db.prepare(`
        INSERT INTO chat_participants (id, chat_id, user_id)
        VALUES (?, ?, ?)
      `);

      for (const participantId of participantIds) {
        insertParticipant.run(uuidv4(), id, participantId);
      }

      return id;
    });

    transaction();
    return this.findById(id);
  }

  static findById(id) {
    const chat = db.prepare(`
      SELECT id, is_group, name, created_at, updated_at
      FROM chat_sessions WHERE id = ?
    `).get(id);

    if (!chat) return null;

    // Get participants
    chat.participants = db.prepare(`
      SELECT u.id, u.name, u.avatar, u.status, cp.pinned, cp.unread_count
      FROM chat_participants cp
      JOIN users u ON cp.user_id = u.id
      WHERE cp.chat_id = ?
    `).all(id);

    // Get last message
    chat.lastMessage = db.prepare(`
      SELECT m.id, m.sender_id, m.content, m.type, m.status, m.created_at
      FROM messages m
      WHERE m.chat_id = ?
      ORDER BY m.created_at DESC
      LIMIT 1
    `).get(id);

    return chat;
  }

  static findByParticipants(participantIds) {
    // Find existing direct chat between two users
    if (participantIds.length !== 2) return null;

    const chat = db.prepare(`
      SELECT cs.id
      FROM chat_sessions cs
      JOIN chat_participants cp1 ON cs.id = cp1.chat_id AND cp1.user_id = ?
      JOIN chat_participants cp2 ON cs.id = cp2.chat_id AND cp2.user_id = ?
      WHERE cs.is_group = 0
      AND (SELECT COUNT(*) FROM chat_participants WHERE chat_id = cs.id) = 2
    `).get(participantIds[0], participantIds[1]);

    return chat ? this.findById(chat.id) : null;
  }

  static getUserChats(userId) {
    const chatIds = db.prepare(`
      SELECT chat_id FROM chat_participants WHERE user_id = ?
    `).all(userId);

    return chatIds.map(({ chat_id }) => {
      const chat = this.findById(chat_id);

      // Get user's participant info (pinned, unread)
      const userParticipant = db.prepare(`
        SELECT pinned, unread_count FROM chat_participants
        WHERE chat_id = ? AND user_id = ?
      `).get(chat_id, userId);

      return {
        ...chat,
        pinned: userParticipant?.pinned === 1,
        unreadCount: userParticipant?.unread_count || 0
      };
    }).sort((a, b) => {
      // Sort by pinned first, then by last message date
      if (a.pinned !== b.pinned) return b.pinned ? 1 : -1;
      const aTime = a.lastMessage?.created_at || a.created_at;
      const bTime = b.lastMessage?.created_at || b.created_at;
      return new Date(bTime) - new Date(aTime);
    });
  }

  static togglePin(chatId, userId) {
    const current = db.prepare(`
      SELECT pinned FROM chat_participants WHERE chat_id = ? AND user_id = ?
    `).get(chatId, userId);

    const newPinned = current?.pinned === 1 ? 0 : 1;

    db.prepare(`
      UPDATE chat_participants SET pinned = ? WHERE chat_id = ? AND user_id = ?
    `).run(newPinned, chatId, userId);

    return newPinned === 1;
  }

  static clearUnread(chatId, userId) {
    db.prepare(`
      UPDATE chat_participants SET unread_count = 0 WHERE chat_id = ? AND user_id = ?
    `).run(chatId, userId);
  }

  static incrementUnread(chatId, excludeUserId) {
    db.prepare(`
      UPDATE chat_participants
      SET unread_count = unread_count + 1
      WHERE chat_id = ? AND user_id != ?
    `).run(chatId, excludeUserId);
  }

  static isParticipant(chatId, userId) {
    const result = db.prepare(`
      SELECT 1 FROM chat_participants WHERE chat_id = ? AND user_id = ?
    `).get(chatId, userId);

    return !!result;
  }
}
