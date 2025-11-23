import { Router } from 'express';
import { User } from '../models/User.js';
import { generateToken, authenticateToken } from '../middleware/auth.js';
import { registerValidation, loginValidation } from '../middleware/validation.js';

const router = Router();

// Register new user
router.post('/register', registerValidation, async (req, res) => {
  try {
    const { email, password, name } = req.body;

    const user = await User.create({ email, password, name });

    const token = generateToken(user.id);

    res.status(201).json({
      message: 'Registration successful',
      user: {
        id: user.id,
        email: user.email,
        name: user.name,
        avatar: user.avatar,
        status: user.status
      },
      token
    });
  } catch (error) {
    if (error.message === 'Email already exists') {
      return res.status(409).json({ error: 'Email already registered' });
    }
    console.error('Registration error:', error);
    res.status(500).json({ error: 'Registration failed' });
  }
});

// Login
router.post('/login', loginValidation, async (req, res) => {
  try {
    const { email, password } = req.body;

    const user = User.findByEmail(email);
    if (!user) {
      return res.status(401).json({ error: 'Invalid email or password' });
    }

    const isValid = await User.verifyPassword(password, user.password);
    if (!isValid) {
      return res.status(401).json({ error: 'Invalid email or password' });
    }

    // Update status to online
    User.updateStatus(user.id, 'online');

    const token = generateToken(user.id);

    res.json({
      message: 'Login successful',
      user: {
        id: user.id,
        email: user.email,
        name: user.name,
        avatar: user.avatar,
        status: 'online'
      },
      token
    });
  } catch (error) {
    console.error('Login error:', error);
    res.status(500).json({ error: 'Login failed' });
  }
});

// Get current user
router.get('/me', authenticateToken, (req, res) => {
  res.json({ user: req.user });
});

// Logout
router.post('/logout', authenticateToken, (req, res) => {
  User.updateStatus(req.user.id, 'offline');
  res.json({ message: 'Logged out successfully' });
});

// Update profile
router.patch('/profile', authenticateToken, async (req, res) => {
  try {
    const { name, avatar } = req.body;
    const user = User.updateProfile(req.user.id, { name, avatar });
    res.json({ user });
  } catch (error) {
    console.error('Profile update error:', error);
    res.status(500).json({ error: 'Failed to update profile' });
  }
});

export default router;
