import db from '../config/database.js';
import { v4 as uuidv4 } from 'uuid';

export class Contact {
  static add(userId, contactId) {
    const id = uuidv4();

    try {
      db.prepare(`
        INSERT INTO contacts (id, user_id, contact_id)
        VALUES (?, ?, ?)
      `).run(id, userId, contactId);

      return { id, userId, contactId };
    } catch (error) {
      if (error.code === 'SQLITE_CONSTRAINT_UNIQUE') {
        throw new Error('Contact already exists');
      }
      throw error;
    }
  }

  static remove(userId, contactId) {
    const result = db.prepare(`
      DELETE FROM contacts WHERE user_id = ? AND contact_id = ?
    `).run(userId, contactId);

    return result.changes > 0;
  }

  static getAll(userId) {
    return db.prepare(`
      SELECT u.id, u.name, u.avatar, u.status, u.last_seen
      FROM contacts c
      JOIN users u ON c.contact_id = u.id
      WHERE c.user_id = ?
      ORDER BY u.name ASC
    `).all(userId);
  }

  static exists(userId, contactId) {
    const result = db.prepare(`
      SELECT 1 FROM contacts WHERE user_id = ? AND contact_id = ?
    `).get(userId, contactId);

    return !!result;
  }
}
