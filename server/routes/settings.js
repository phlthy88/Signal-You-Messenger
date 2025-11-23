import { Router } from 'express';
import { authenticateToken } from '../middleware/auth.js';
import { updateSettingsValidation } from '../middleware/validation.js';
import { Settings } from '../models/Settings.js';

const router = Router();

// Get user settings
router.get('/', authenticateToken, (req, res) => {
  try {
    const settings = Settings.get(req.user.id);
    res.json({ settings });
  } catch (error) {
    console.error('Get settings error:', error);
    res.status(500).json({ error: 'Failed to fetch settings' });
  }
});

// Update settings
router.patch('/', authenticateToken, updateSettingsValidation, (req, res) => {
  try {
    const { theme, notificationsEnabled, soundEnabled } = req.body;

    const settings = Settings.update(req.user.id, {
      theme,
      notificationsEnabled,
      soundEnabled
    });

    res.json({ settings });
  } catch (error) {
    console.error('Update settings error:', error);
    res.status(500).json({ error: 'Failed to update settings' });
  }
});

export default router;
