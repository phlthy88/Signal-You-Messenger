import { body, param, query, validationResult } from 'express-validator';

export const handleValidationErrors = (req, res, next) => {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    return res.status(400).json({
      error: 'Validation failed',
      details: errors.array().map(err => ({
        field: err.path,
        message: err.msg
      }))
    });
  }
  next();
};

// Auth validation rules
export const registerValidation = [
  body('email')
    .isEmail()
    .normalizeEmail()
    .withMessage('Valid email is required'),
  body('password')
    .isLength({ min: 8 })
    .withMessage('Password must be at least 8 characters')
    .matches(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/)
    .withMessage('Password must contain uppercase, lowercase and number'),
  body('name')
    .trim()
    .isLength({ min: 2, max: 50 })
    .withMessage('Name must be between 2 and 50 characters'),
  handleValidationErrors
];

export const loginValidation = [
  body('email')
    .isEmail()
    .normalizeEmail()
    .withMessage('Valid email is required'),
  body('password')
    .notEmpty()
    .withMessage('Password is required'),
  handleValidationErrors
];

// Message validation rules
export const sendMessageValidation = [
  body('content')
    .trim()
    .isLength({ min: 1, max: 5000 })
    .withMessage('Message content must be between 1 and 5000 characters'),
  body('type')
    .optional()
    .isIn(['text', 'image', 'file'])
    .withMessage('Invalid message type'),
  handleValidationErrors
];

// Chat validation rules
export const createChatValidation = [
  body('participantIds')
    .isArray({ min: 1 })
    .withMessage('At least one participant is required'),
  body('participantIds.*')
    .isUUID()
    .withMessage('Invalid participant ID'),
  handleValidationErrors
];

// Contact validation rules
export const addContactValidation = [
  body('contactId')
    .isUUID()
    .withMessage('Valid contact ID is required'),
  handleValidationErrors
];

// Settings validation rules
export const updateSettingsValidation = [
  body('theme')
    .optional()
    .isIn(['Deep Purple', 'Forest Green', 'Ocean Blue', 'Dark Mode'])
    .withMessage('Invalid theme'),
  body('notificationsEnabled')
    .optional()
    .isBoolean()
    .withMessage('notificationsEnabled must be a boolean'),
  body('soundEnabled')
    .optional()
    .isBoolean()
    .withMessage('soundEnabled must be a boolean'),
  handleValidationErrors
];
