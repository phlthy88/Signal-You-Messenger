import { Router } from 'express';
import { authenticateToken } from '../middleware/auth.js';
import { addContactValidation } from '../middleware/validation.js';
import { Contact } from '../models/Contact.js';
import { User } from '../models/User.js';
import { isUserOnline } from '../services/websocket.js';

const router = Router();

// Get all contacts
router.get('/', authenticateToken, (req, res) => {
  try {
    const contacts = Contact.getAll(req.user.id);

    // Add real-time online status
    const contactsWithStatus = contacts.map(contact => ({
      ...contact,
      status: isUserOnline(contact.id) ? 'online' : contact.status
    }));

    res.json({ contacts: contactsWithStatus });
  } catch (error) {
    console.error('Get contacts error:', error);
    res.status(500).json({ error: 'Failed to fetch contacts' });
  }
});

// Add contact
router.post('/', authenticateToken, addContactValidation, (req, res) => {
  try {
    const { contactId } = req.body;

    // Can't add self
    if (contactId === req.user.id) {
      return res.status(400).json({ error: 'Cannot add yourself as contact' });
    }

    // Check if contact user exists
    const contactUser = User.findById(contactId);
    if (!contactUser) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Add contact (bidirectional)
    Contact.add(req.user.id, contactId);

    // Optionally add reverse relationship
    try {
      Contact.add(contactId, req.user.id);
    } catch (e) {
      // Ignore if already exists
    }

    res.status(201).json({
      contact: {
        ...contactUser,
        status: isUserOnline(contactUser.id) ? 'online' : contactUser.status
      }
    });
  } catch (error) {
    if (error.message === 'Contact already exists') {
      return res.status(409).json({ error: 'Contact already exists' });
    }
    console.error('Add contact error:', error);
    res.status(500).json({ error: 'Failed to add contact' });
  }
});

// Remove contact
router.delete('/:contactId', authenticateToken, (req, res) => {
  try {
    const { contactId } = req.params;

    const removed = Contact.remove(req.user.id, contactId);

    if (!removed) {
      return res.status(404).json({ error: 'Contact not found' });
    }

    res.json({ success: true });
  } catch (error) {
    console.error('Remove contact error:', error);
    res.status(500).json({ error: 'Failed to remove contact' });
  }
});

// Search users (for adding new contacts)
router.get('/search', authenticateToken, (req, res) => {
  try {
    const { q } = req.query;

    if (!q || q.length < 2) {
      return res.status(400).json({ error: 'Search query too short' });
    }

    const users = User.search(q, req.user.id);

    // Mark which users are already contacts
    const existingContacts = Contact.getAll(req.user.id);
    const contactIds = new Set(existingContacts.map(c => c.id));

    const usersWithContactStatus = users.map(user => ({
      ...user,
      isContact: contactIds.has(user.id),
      status: isUserOnline(user.id) ? 'online' : user.status
    }));

    res.json({ users: usersWithContactStatus });
  } catch (error) {
    console.error('Search users error:', error);
    res.status(500).json({ error: 'Search failed' });
  }
});

// Get all users (for demo purposes)
router.get('/all-users', authenticateToken, (req, res) => {
  try {
    const users = User.getAll(req.user.id);

    const existingContacts = Contact.getAll(req.user.id);
    const contactIds = new Set(existingContacts.map(c => c.id));

    const usersWithStatus = users.map(user => ({
      ...user,
      isContact: contactIds.has(user.id),
      status: isUserOnline(user.id) ? 'online' : user.status
    }));

    res.json({ users: usersWithStatus });
  } catch (error) {
    console.error('Get all users error:', error);
    res.status(500).json({ error: 'Failed to fetch users' });
  }
});

export default router;
