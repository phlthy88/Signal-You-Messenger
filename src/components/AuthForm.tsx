import React, { useState } from 'react';
import { useStore } from '../store';
import { MessageSquare, Mail, Lock, User, Loader2, AlertCircle } from 'lucide-react';

export const AuthForm: React.FC = () => {
  const [isLogin, setIsLogin] = useState(true);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [name, setName] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const login = useStore((state) => state.login);
  const register = useStore((state) => state.register);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      if (isLogin) {
        await login(email, password);
      } else {
        await register(email, password, name);
      }
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <div className="w-16 h-16 rounded-2xl bg-primary flex items-center justify-center mx-auto mb-4 shadow-lg shadow-primary/30">
            <MessageSquare className="w-8 h-8 text-on-primary" />
          </div>
          <h1 className="text-2xl font-bold text-on-background">Signal You Messenger</h1>
          <p className="text-on-surface-variant mt-1">Secure messaging with AI features</p>
        </div>

        {/* Form Card */}
        <div className="bg-surface-container rounded-3xl p-8 shadow-xl">
          <h2 className="text-xl font-semibold text-on-surface mb-6">
            {isLogin ? 'Welcome back' : 'Create account'}
          </h2>

          {error && (
            <div className="flex items-center gap-2 p-3 mb-4 bg-error-container text-on-error-container rounded-xl">
              <AlertCircle size={18} />
              <span className="text-sm">{error}</span>
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-4">
            {!isLogin && (
              <div className="relative">
                <User className="absolute left-4 top-1/2 -translate-y-1/2 text-on-surface-variant" size={20} />
                <input
                  type="text"
                  placeholder="Full name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  required={!isLogin}
                  className="w-full bg-surface-variant/50 text-on-surface rounded-xl py-3 pl-12 pr-4 outline-none focus:ring-2 focus:ring-primary transition-all placeholder:text-on-surface-variant/60"
                />
              </div>
            )}

            <div className="relative">
              <Mail className="absolute left-4 top-1/2 -translate-y-1/2 text-on-surface-variant" size={20} />
              <input
                type="email"
                placeholder="Email address"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                className="w-full bg-surface-variant/50 text-on-surface rounded-xl py-3 pl-12 pr-4 outline-none focus:ring-2 focus:ring-primary transition-all placeholder:text-on-surface-variant/60"
              />
            </div>

            <div className="relative">
              <Lock className="absolute left-4 top-1/2 -translate-y-1/2 text-on-surface-variant" size={20} />
              <input
                type="password"
                placeholder="Password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                minLength={8}
                className="w-full bg-surface-variant/50 text-on-surface rounded-xl py-3 pl-12 pr-4 outline-none focus:ring-2 focus:ring-primary transition-all placeholder:text-on-surface-variant/60"
              />
            </div>

            {!isLogin && (
              <p className="text-xs text-on-surface-variant">
                Password must be at least 8 characters with uppercase, lowercase, and a number.
              </p>
            )}

            <button
              type="submit"
              disabled={loading}
              className="w-full bg-primary text-on-primary rounded-xl py-3 font-semibold hover:shadow-lg hover:shadow-primary/30 transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              {loading && <Loader2 className="animate-spin" size={20} />}
              {isLogin ? 'Sign in' : 'Create account'}
            </button>
          </form>

          <div className="mt-6 text-center">
            <button
              onClick={() => {
                setIsLogin(!isLogin);
                setError('');
              }}
              className="text-primary hover:underline text-sm"
            >
              {isLogin ? "Don't have an account? Sign up" : 'Already have an account? Sign in'}
            </button>
          </div>
        </div>

        {/* Demo credentials hint */}
        <p className="text-center text-on-surface-variant text-xs mt-4">
          Create a new account to get started
        </p>
      </div>
    </div>
  );
};

export default AuthForm;
