import db from '../config/database.js';
import bcrypt from 'bcryptjs';
import { v4 as uuidv4 } from 'uuid';

export class User {
  static async create({ email, password, name, avatar }) {
    const id = uuidv4();
    const hashedPassword = await bcrypt.hash(password, 12);
    const defaultAvatar = avatar || `https://ui-avatars.com/api/?name=${encodeURIComponent(name)}&background=random`;

    try {
      const stmt = db.prepare(`
        INSERT INTO users (id, email, password, name, avatar, status)
        VALUES (?, ?, ?, ?, ?, 'offline')
      `);

      stmt.run(id, email, hashedPassword, name, defaultAvatar);

      // Create default settings
      db.prepare(`
        INSERT INTO user_settings (user_id) VALUES (?)
      `).run(id);

      return this.findById(id);
    } catch (error) {
      if (error.code === 'SQLITE_CONSTRAINT_UNIQUE') {
        throw new Error('Email already exists');
      }
      throw error;
    }
  }

  static findById(id) {
    return db.prepare(`
      SELECT id, email, name, avatar, status, last_seen, created_at
      FROM users WHERE id = ?
    `).get(id);
  }

  static findByEmail(email) {
    return db.prepare(`
      SELECT id, email, password, name, avatar, status, last_seen, created_at
      FROM users WHERE email = ?
    `).get(email);
  }

  static async verifyPassword(plainPassword, hashedPassword) {
    return bcrypt.compare(plainPassword, hashedPassword);
  }

  static updateStatus(id, status) {
    return db.prepare(`
      UPDATE users SET status = ?, last_seen = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
      WHERE id = ?
    `).run(status, id);
  }

  static updateProfile(id, { name, avatar }) {
    const updates = [];
    const values = [];

    if (name) {
      updates.push('name = ?');
      values.push(name);
    }
    if (avatar) {
      updates.push('avatar = ?');
      values.push(avatar);
    }

    if (updates.length === 0) return this.findById(id);

    updates.push('updated_at = CURRENT_TIMESTAMP');
    values.push(id);

    db.prepare(`
      UPDATE users SET ${updates.join(', ')} WHERE id = ?
    `).run(...values);

    return this.findById(id);
  }

  static search(query, excludeUserId) {
    return db.prepare(`
      SELECT id, name, avatar, status, last_seen
      FROM users
      WHERE (name LIKE ? OR email LIKE ?) AND id != ?
      LIMIT 20
    `).all(`%${query}%`, `%${query}%`, excludeUserId);
  }

  static getAll(excludeUserId) {
    return db.prepare(`
      SELECT id, name, avatar, status, last_seen
      FROM users WHERE id != ?
      ORDER BY name ASC
    `).all(excludeUserId);
  }
}
