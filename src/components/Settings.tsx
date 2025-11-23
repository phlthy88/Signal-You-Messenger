import React, { useState } from 'react';
import { User, Bell, Volume2, Palette, Shield, HelpCircle, LogOut, ChevronRight, Check, Loader2 } from 'lucide-react';
import { useStore } from '../store';
import { useTheme } from '../contexts/ThemeContext';

const Settings: React.FC = () => {
  const user = useStore((state) => state.user);
  const settings = useStore((state) => state.settings);
  const updateSettings = useStore((state) => state.updateSettings);
  const logout = useStore((state) => state.logout);
  const { currentTheme, setTheme, availableThemes } = useTheme();

  const [showThemes, setShowThemes] = useState(false);
  const [loading, setLoading] = useState<string | null>(null);

  const handleToggle = async (key: 'notificationsEnabled' | 'soundEnabled') => {
    setLoading(key);
    try {
      await updateSettings({ [key]: !settings[key] });
    } catch (error) {
      alert('Failed to update setting');
    } finally {
      setLoading(null);
    }
  };

  const handleLogout = async () => {
    if (confirm('Are you sure you want to sign out?')) {
      await logout();
    }
  };

  return (
    <div className="flex flex-col h-full bg-surface-variant/30 overflow-y-auto">
      {/* Header */}
      <div className="p-6 border-b border-outline-variant/20">
        <h2 className="text-2xl font-bold text-on-surface">Settings</h2>
      </div>

      {/* Profile Section */}
      <div className="p-4">
        <div className="bg-surface-container rounded-2xl p-4">
          <div className="flex items-center gap-4">
            <div className="w-16 h-16 rounded-full bg-primary-container overflow-hidden">
              {user?.avatar ? (
                <img src={user.avatar} alt={user.name} className="w-full h-full object-cover" />
              ) : (
                <div className="w-full h-full flex items-center justify-center text-primary text-xl font-bold">
                  {user?.name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2)}
                </div>
              )}
            </div>
            <div className="flex-1">
              <h3 className="font-bold text-lg text-on-surface">{user?.name}</h3>
              <p className="text-sm text-on-surface-variant">{user?.email}</p>
            </div>
            <button className="p-2 rounded-full hover:bg-surface-variant text-primary">
              <User size={20} />
            </button>
          </div>
        </div>
      </div>

      {/* Settings Groups */}
      <div className="p-4 space-y-4">
        {/* Appearance */}
        <div className="bg-surface-container rounded-2xl overflow-hidden">
          <div className="px-4 py-3 border-b border-outline-variant/20">
            <h4 className="text-xs font-bold text-primary uppercase tracking-wider">Appearance</h4>
          </div>

          <button
            onClick={() => setShowThemes(!showThemes)}
            className="w-full flex items-center justify-between p-4 hover:bg-surface-variant/50 transition-colors"
          >
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-surface-variant flex items-center justify-center">
                <Palette size={20} className="text-primary" />
              </div>
              <div className="text-left">
                <p className="font-medium text-on-surface">Theme</p>
                <p className="text-sm text-on-surface-variant">{currentTheme}</p>
              </div>
            </div>
            <ChevronRight size={20} className={`text-on-surface-variant transition-transform ${showThemes ? 'rotate-90' : ''}`} />
          </button>

          {showThemes && (
            <div className="px-4 pb-4 space-y-2">
              {availableThemes.map(theme => (
                <button
                  key={theme.name}
                  onClick={() => setTheme(theme.name)}
                  className={`w-full flex items-center justify-between p-3 rounded-xl transition-colors ${
                    currentTheme === theme.name
                      ? 'bg-primary-container text-on-primary-container'
                      : 'hover:bg-surface-variant text-on-surface'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <div
                      className="w-6 h-6 rounded-full"
                      style={{ backgroundColor: theme.colors['--md-sys-color-primary'] }}
                    />
                    <span>{theme.name}</span>
                  </div>
                  {currentTheme === theme.name && <Check size={18} />}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Notifications */}
        <div className="bg-surface-container rounded-2xl overflow-hidden">
          <div className="px-4 py-3 border-b border-outline-variant/20">
            <h4 className="text-xs font-bold text-primary uppercase tracking-wider">Notifications</h4>
          </div>

          <div className="flex items-center justify-between p-4 hover:bg-surface-variant/50 transition-colors">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-surface-variant flex items-center justify-center">
                <Bell size={20} className="text-primary" />
              </div>
              <div>
                <p className="font-medium text-on-surface">Push Notifications</p>
                <p className="text-sm text-on-surface-variant">Get notified about new messages</p>
              </div>
            </div>
            <button
              onClick={() => handleToggle('notificationsEnabled')}
              disabled={loading === 'notificationsEnabled'}
              className={`relative w-12 h-7 rounded-full transition-colors ${
                settings.notificationsEnabled ? 'bg-primary' : 'bg-outline'
              }`}
              aria-label="Toggle notifications"
            >
              {loading === 'notificationsEnabled' ? (
                <Loader2 className="absolute inset-0 m-auto w-4 h-4 text-on-primary animate-spin" />
              ) : (
                <div
                  className={`absolute top-1 w-5 h-5 rounded-full bg-white shadow transition-transform ${
                    settings.notificationsEnabled ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              )}
            </button>
          </div>

          <div className="flex items-center justify-between p-4 hover:bg-surface-variant/50 transition-colors">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-surface-variant flex items-center justify-center">
                <Volume2 size={20} className="text-primary" />
              </div>
              <div>
                <p className="font-medium text-on-surface">Sound</p>
                <p className="text-sm text-on-surface-variant">Play sounds for messages</p>
              </div>
            </div>
            <button
              onClick={() => handleToggle('soundEnabled')}
              disabled={loading === 'soundEnabled'}
              className={`relative w-12 h-7 rounded-full transition-colors ${
                settings.soundEnabled ? 'bg-primary' : 'bg-outline'
              }`}
              aria-label="Toggle sound"
            >
              {loading === 'soundEnabled' ? (
                <Loader2 className="absolute inset-0 m-auto w-4 h-4 text-on-primary animate-spin" />
              ) : (
                <div
                  className={`absolute top-1 w-5 h-5 rounded-full bg-white shadow transition-transform ${
                    settings.soundEnabled ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              )}
            </button>
          </div>
        </div>

        {/* Privacy & Security */}
        <div className="bg-surface-container rounded-2xl overflow-hidden">
          <div className="px-4 py-3 border-b border-outline-variant/20">
            <h4 className="text-xs font-bold text-primary uppercase tracking-wider">Privacy & Security</h4>
          </div>

          <button className="w-full flex items-center justify-between p-4 hover:bg-surface-variant/50 transition-colors">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-surface-variant flex items-center justify-center">
                <Shield size={20} className="text-primary" />
              </div>
              <div className="text-left">
                <p className="font-medium text-on-surface">Privacy Settings</p>
                <p className="text-sm text-on-surface-variant">Manage your privacy</p>
              </div>
            </div>
            <ChevronRight size={20} className="text-on-surface-variant" />
          </button>
        </div>

        {/* Help & Support */}
        <div className="bg-surface-container rounded-2xl overflow-hidden">
          <button className="w-full flex items-center justify-between p-4 hover:bg-surface-variant/50 transition-colors">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-surface-variant flex items-center justify-center">
                <HelpCircle size={20} className="text-primary" />
              </div>
              <div className="text-left">
                <p className="font-medium text-on-surface">Help & Support</p>
                <p className="text-sm text-on-surface-variant">Get help, contact us</p>
              </div>
            </div>
            <ChevronRight size={20} className="text-on-surface-variant" />
          </button>
        </div>

        {/* Sign Out */}
        <button
          onClick={handleLogout}
          className="w-full bg-error-container text-on-error-container rounded-2xl p-4 flex items-center justify-center gap-2 hover:shadow-lg transition-all"
        >
          <LogOut size={20} />
          <span className="font-medium">Sign Out</span>
        </button>

        {/* Version */}
        <p className="text-center text-on-surface-variant text-xs py-4">
          Signal You Messenger v1.0.0
        </p>
      </div>
    </div>
  );
};

export default Settings;
