import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import AuthForm from '../../components/AuthForm';
import { useStore } from '../../store';

// Mock the store
vi.mock('../../store', () => ({
  useStore: vi.fn(),
}));

describe('AuthForm', () => {
  const mockLogin = vi.fn();
  const mockRegister = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useStore as any).mockImplementation((selector: any) => {
      const state = {
        login: mockLogin,
        register: mockRegister,
      };
      return selector(state);
    });
  });

  it('renders login form by default', () => {
    render(<AuthForm />);

    expect(screen.getByText('Welcome back')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Email address')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Password')).toBeInTheDocument();
    expect(screen.getByText('Sign in')).toBeInTheDocument();
  });

  it('switches to register form when clicking signup link', async () => {
    const user = userEvent.setup();
    render(<AuthForm />);

    await user.click(screen.getByText("Don't have an account? Sign up"));

    expect(screen.getByText('Create account')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Full name')).toBeInTheDocument();
  });

  it('calls login with credentials on submit', async () => {
    const user = userEvent.setup();
    mockLogin.mockResolvedValue({});
    render(<AuthForm />);

    await user.type(screen.getByPlaceholderText('Email address'), 'test@example.com');
    await user.type(screen.getByPlaceholderText('Password'), 'password123');
    await user.click(screen.getByText('Sign in'));

    await waitFor(() => {
      expect(mockLogin).toHaveBeenCalledWith('test@example.com', 'password123');
    });
  });

  it('displays error message on login failure', async () => {
    const user = userEvent.setup();
    mockLogin.mockRejectedValue(new Error('Invalid credentials'));
    render(<AuthForm />);

    await user.type(screen.getByPlaceholderText('Email address'), 'test@example.com');
    await user.type(screen.getByPlaceholderText('Password'), 'wrong');
    await user.click(screen.getByText('Sign in'));

    await waitFor(() => {
      expect(screen.getByText('Invalid credentials')).toBeInTheDocument();
    });
  });

  it('calls register with name, email, and password', async () => {
    const user = userEvent.setup();
    mockRegister.mockResolvedValue({});
    render(<AuthForm />);

    // Switch to register
    await user.click(screen.getByText("Don't have an account? Sign up"));

    await user.type(screen.getByPlaceholderText('Full name'), 'John Doe');
    await user.type(screen.getByPlaceholderText('Email address'), 'john@example.com');
    await user.type(screen.getByPlaceholderText('Password'), 'Password123');
    await user.click(screen.getByText('Create account'));

    await waitFor(() => {
      expect(mockRegister).toHaveBeenCalledWith('john@example.com', 'Password123', 'John Doe');
    });
  });
});
