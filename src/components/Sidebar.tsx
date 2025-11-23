import React from 'react';
import { MessageSquare, Phone, Settings, Users, Palette, LogOut } from 'lucide-react';
import { useTheme } from '../contexts/ThemeContext';
import { useStore } from '../store';
import { ViewMode } from '../types';

interface SidebarProps {
  activeView: ViewMode;
  onViewChange: (view: ViewMode) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ activeView, onViewChange }) => {
  const { currentTheme, setTheme, availableThemes } = useTheme();
  const user = useStore((state) => state.user);
  const logout = useStore((state) => state.logout);

  const handleLogout = async () => {
    if (confirm('Are you sure you want to sign out?')) {
      await logout();
    }
  };

  return (
    <div className="w-20 bg-surface h-full flex flex-col items-center py-6 border-r border-outline-variant/20 z-20">
      <div className="mb-8">
        <div className="w-10 h-10 rounded-xl bg-primary flex items-center justify-center shadow-lg shadow-primary/30">
          <svg viewBox="0 0 24 24" fill="none" className="w-6 h-6 text-on-primary" stroke="currentColor" strokeWidth="2">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          </svg>
        </div>
      </div>

      <nav className="flex-1 flex flex-col space-y-2 w-full px-2">
        <SidebarButton
          icon={<MessageSquare size={24} />}
          active={activeView === 'chats'}
          onClick={() => onViewChange('chats')}
          label="Chats"
        />
        <SidebarButton
          icon={<Users size={24} />}
          active={activeView === 'contacts'}
          onClick={() => onViewChange('contacts')}
          label="Contacts"
        />
        <SidebarButton
          icon={<Phone size={24} />}
          active={false}
          onClick={() => alert('Calls feature coming soon!')}
          label="Calls"
        />
      </nav>

      <div className="mt-auto flex flex-col space-y-4 items-center w-full px-2">
        {/* Theme Picker Dropup */}
        <div className="group relative">
          <button
            className="p-3 rounded-2xl text-on-surface-variant hover:bg-surface-variant/50 transition-colors"
            aria-label="Change theme"
          >
            <Palette size={24} />
          </button>

          <div className="absolute bottom-full left-full ml-2 mb-2 w-48 bg-surface-container rounded-2xl shadow-xl p-2 hidden group-hover:block z-50 border border-outline-variant/20">
            <h4 className="px-3 py-2 text-xs font-bold text-on-surface-variant uppercase">Theme</h4>
            {availableThemes.map(t => (
              <button
                key={t.name}
                onClick={() => setTheme(t.name)}
                className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                  currentTheme === t.name
                    ? 'bg-primary-container text-on-primary-container'
                    : 'text-on-surface hover:bg-surface-variant'
                }`}
              >
                <div className="flex items-center gap-2">
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: t.colors['--md-sys-color-primary'] }}
                  />
                  {t.name}
                </div>
              </button>
            ))}
          </div>
        </div>

        <SidebarButton
          icon={<Settings size={24} />}
          active={activeView === 'settings'}
          onClick={() => onViewChange('settings')}
          label="Settings"
        />

        <SidebarButton
          icon={<LogOut size={24} />}
          active={false}
          onClick={handleLogout}
          label="Sign out"
        />

        {/* User avatar */}
        {user && (
          <div
            className="w-10 h-10 rounded-full bg-surface-variant/50 flex items-center justify-center overflow-hidden mt-4 cursor-pointer hover:ring-2 hover:ring-primary/50 transition-all"
            title={user.name}
          >
            {user.avatar ? (
              <img src={user.avatar} alt={user.name} className="w-full h-full object-cover" />
            ) : (
              <span className="text-xs font-bold text-primary">
                {user.name.split(' ').map(n => n[0]).join('').toUpperCase().slice(0, 2)}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

interface SidebarButtonProps {
  icon: React.ReactNode;
  active: boolean;
  onClick: () => void;
  label: string;
}

const SidebarButton: React.FC<SidebarButtonProps> = ({ icon, active, onClick, label }) => (
  <button
    onClick={onClick}
    title={label}
    aria-label={label}
    className={`w-full p-3 flex items-center justify-center rounded-2xl transition-all duration-300 relative group ${
      active
        ? 'bg-secondary-container text-on-secondary-container shadow-sm'
        : 'text-on-surface-variant hover:bg-surface-variant/30 hover:text-on-surface'
    }`}
  >
    {icon}
    <span className="absolute left-full ml-2 px-2 py-1 bg-surface-variant text-on-surface-variant text-xs rounded opacity-0 group-hover:opacity-100 pointer-events-none whitespace-nowrap shadow-md z-50 transition-opacity">
      {label}
    </span>
  </button>
);

export default Sidebar;
