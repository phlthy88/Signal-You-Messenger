import db from '../config/database.js';

export class Settings {
  static get(userId) {
    const settings = db.prepare(`
      SELECT theme, notifications_enabled, sound_enabled
      FROM user_settings WHERE user_id = ?
    `).get(userId);

    if (!settings) {
      // Create default settings if not exists
      this.create(userId);
      return this.get(userId);
    }

    return {
      theme: settings.theme,
      notificationsEnabled: settings.notifications_enabled === 1,
      soundEnabled: settings.sound_enabled === 1
    };
  }

  static create(userId) {
    db.prepare(`
      INSERT OR IGNORE INTO user_settings (user_id) VALUES (?)
    `).run(userId);
  }

  static update(userId, { theme, notificationsEnabled, soundEnabled }) {
    const updates = [];
    const values = [];

    if (theme !== undefined) {
      updates.push('theme = ?');
      values.push(theme);
    }
    if (notificationsEnabled !== undefined) {
      updates.push('notifications_enabled = ?');
      values.push(notificationsEnabled ? 1 : 0);
    }
    if (soundEnabled !== undefined) {
      updates.push('sound_enabled = ?');
      values.push(soundEnabled ? 1 : 0);
    }

    if (updates.length === 0) return this.get(userId);

    updates.push('updated_at = CURRENT_TIMESTAMP');
    values.push(userId);

    db.prepare(`
      UPDATE user_settings SET ${updates.join(', ')} WHERE user_id = ?
    `).run(...values);

    return this.get(userId);
  }
}
