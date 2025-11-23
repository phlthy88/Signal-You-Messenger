import db from '../config/database.js';
import { v4 as uuidv4 } from 'uuid';
import { ChatSession } from './ChatSession.js';

export class Message {
  static create({ chatId, senderId, content, type = 'text', fileUrl = null }) {
    const id = uuidv4();

    db.prepare(`
      INSERT INTO messages (id, chat_id, sender_id, content, type, file_url, status)
      VALUES (?, ?, ?, ?, ?, ?, 'sent')
    `).run(id, chatId, senderId, content, type, fileUrl);

    // Update chat session timestamp
    db.prepare(`
      UPDATE chat_sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?
    `).run(chatId);

    // Increment unread count for other participants
    ChatSession.incrementUnread(chatId, senderId);

    return this.findById(id);
  }

  static findById(id) {
    return db.prepare(`
      SELECT m.*, u.name as sender_name, u.avatar as sender_avatar
      FROM messages m
      JOIN users u ON m.sender_id = u.id
      WHERE m.id = ?
    `).get(id);
  }

  static getChatMessages(chatId, { limit = 50, offset = 0 } = {}) {
    return db.prepare(`
      SELECT m.*, u.name as sender_name, u.avatar as sender_avatar
      FROM messages m
      JOIN users u ON m.sender_id = u.id
      WHERE m.chat_id = ?
      ORDER BY m.created_at ASC
      LIMIT ? OFFSET ?
    `).all(chatId, limit, offset);
  }

  static updateStatus(id, status) {
    db.prepare(`
      UPDATE messages SET status = ? WHERE id = ?
    `).run(status, id);

    return this.findById(id);
  }

  static markChatMessagesAsRead(chatId, readerId) {
    db.prepare(`
      UPDATE messages
      SET status = 'read'
      WHERE chat_id = ? AND sender_id != ? AND status != 'read'
    `).run(chatId, readerId);

    // Clear unread count
    ChatSession.clearUnread(chatId, readerId);
  }

  static search(chatId, query) {
    return db.prepare(`
      SELECT m.*, u.name as sender_name, u.avatar as sender_avatar
      FROM messages m
      JOIN users u ON m.sender_id = u.id
      WHERE m.chat_id = ? AND m.content LIKE ?
      ORDER BY m.created_at DESC
      LIMIT 50
    `).all(chatId, `%${query}%`);
  }

  static searchAllChats(userId, query) {
    return db.prepare(`
      SELECT m.*, u.name as sender_name, u.avatar as sender_avatar, cs.id as chat_id
      FROM messages m
      JOIN users u ON m.sender_id = u.id
      JOIN chat_sessions cs ON m.chat_id = cs.id
      JOIN chat_participants cp ON cs.id = cp.chat_id
      WHERE cp.user_id = ? AND m.content LIKE ?
      ORDER BY m.created_at DESC
      LIMIT 50
    `).all(userId, `%${query}%`);
  }

  static getRecentForAI(chatId, limit = 10) {
    return db.prepare(`
      SELECT m.content, m.sender_id, u.name as sender_name
      FROM messages m
      JOIN users u ON m.sender_id = u.id
      WHERE m.chat_id = ?
      ORDER BY m.created_at DESC
      LIMIT ?
    `).all(chatId, limit).reverse();
  }
}
