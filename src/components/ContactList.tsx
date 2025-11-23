import React, { useState, useMemo, useEffect } from 'react';
import { Search, UserPlus, MoreVertical, Trash2, MessageSquare, X, Loader2 } from 'lucide-react';
import { useStore } from '../store';
import api from '../services/api';
import { User } from '../types';

interface ContactListProps {
  contacts: User[];
  onSelectContact: (contact: User) => void;
}

const ContactList: React.FC<ContactListProps> = ({ contacts, onSelectContact }) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [showAddModal, setShowAddModal] = useState(false);
  const [searchResults, setSearchResults] = useState<(User & { isContact: boolean })[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);
  const [userSearch, setUserSearch] = useState('');
  const [contextMenu, setContextMenu] = useState<{ contactId: string; x: number; y: number } | null>(null);

  const addContact = useStore((state) => state.addContact);
  const removeContact = useStore((state) => state.removeContact);

  const filteredContacts = useMemo(() => {
    if (!searchTerm.trim()) return contacts;
    return contacts.filter(contact =>
      contact.name.toLowerCase().includes(searchTerm.toLowerCase())
    );
  }, [contacts, searchTerm]);

  // Search for users to add
  useEffect(() => {
    const searchUsers = async () => {
      if (userSearch.length < 2) {
        setSearchResults([]);
        return;
      }

      setSearchLoading(true);
      try {
        const { users } = await api.searchUsers(userSearch);
        setSearchResults(users);
      } catch (error) {
        console.error('Search failed:', error);
      } finally {
        setSearchLoading(false);
      }
    };

    const debounce = setTimeout(searchUsers, 300);
    return () => clearTimeout(debounce);
  }, [userSearch]);

  const handleAddContact = async (userId: string) => {
    try {
      await addContact(userId);
      setSearchResults(prev =>
        prev.map(u => u.id === userId ? { ...u, isContact: true } : u)
      );
    } catch (error) {
      alert('Failed to add contact');
    }
  };

  const handleRemoveContact = async (contactId: string) => {
    if (confirm('Remove this contact?')) {
      try {
        await removeContact(contactId);
        setContextMenu(null);
      } catch (error) {
        alert('Failed to remove contact');
      }
    }
  };

  const handleContextMenu = (e: React.MouseEvent, contactId: string) => {
    e.preventDefault();
    setContextMenu({ contactId, x: e.clientX, y: e.clientY });
  };

  return (
    <div className="flex flex-col h-full bg-surface-variant/30 border-r border-outline-variant/20 w-80 md:w-96">
      {/* Header */}
      <div className="p-4 flex items-center justify-between">
        <h2 className="text-2xl font-bold text-on-surface">Contacts</h2>
        <button
          onClick={() => setShowAddModal(true)}
          className="p-2 rounded-full bg-tertiary-container text-on-tertiary-container hover:shadow-lg transition-all"
          title="Add Contact"
          aria-label="Add contact"
        >
          <UserPlus size={24} />
        </button>
      </div>

      {/* Search Bar */}
      <div className="px-4 pb-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-on-surface-variant/60" size={18} />
          <input
            type="text"
            placeholder="Search contacts"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full bg-surface-variant/50 text-on-surface rounded-full py-3 pl-10 pr-4 outline-none focus:ring-2 focus:ring-primary/50 transition-all placeholder:text-on-surface-variant/60"
            aria-label="Search contacts"
          />
        </div>
      </div>

      {/* Contacts List */}
      <div className="flex-1 overflow-y-auto px-2 space-y-1 pb-4">
        <div className="px-4 py-2 text-xs font-bold text-primary uppercase tracking-wider">
          {filteredContacts.length} Contact{filteredContacts.length !== 1 ? 's' : ''}
        </div>

        {filteredContacts.length === 0 ? (
          <div className="text-center py-10 text-on-surface-variant/60">
            {searchTerm ? 'No contacts found' : 'No contacts yet. Add some!'}
          </div>
        ) : (
          filteredContacts.map((contact) => (
            <button
              key={contact.id}
              onClick={() => onSelectContact(contact)}
              onContextMenu={(e) => handleContextMenu(e, contact.id)}
              className="w-full flex items-center p-3 rounded-[1.5rem] hover:bg-surface-variant/40 text-on-surface transition-colors group"
            >
              <div className="relative">
                <img
                  src={contact.avatar}
                  alt={contact.name}
                  className="w-12 h-12 rounded-full object-cover border border-outline-variant/20"
                />
                {contact.status === 'online' && (
                  <span className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 border-2 border-surface rounded-full" />
                )}
              </div>

              <div className="ml-4 flex-1 text-left">
                <div className="flex items-center gap-2">
                  <h3 className="font-semibold text-base text-on-surface group-hover:text-on-surface-variant transition-colors">
                    {contact.name}
                  </h3>
                  {contact.status === 'online' && (
                    <span className="w-2 h-2 bg-green-500 rounded-full" title="Online" />
                  )}
                </div>
                <p className="text-sm text-on-surface-variant/70">
                  {contact.status === 'online' ? 'Online' : 'Last seen recently'}
                </p>
              </div>

              <div className="opacity-0 group-hover:opacity-100 transition-opacity">
                <div className="p-2 hover:bg-surface-variant rounded-full text-on-surface-variant">
                  <MessageSquare size={16} />
                </div>
              </div>
            </button>
          ))
        )}
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={() => setContextMenu(null)}
          />
          <div
            className="fixed bg-surface-container rounded-xl shadow-xl p-2 z-50 border border-outline-variant/20"
            style={{ left: contextMenu.x, top: contextMenu.y }}
          >
            <button
              onClick={() => handleRemoveContact(contextMenu.contactId)}
              className="flex items-center gap-2 w-full px-3 py-2 text-error hover:bg-error-container rounded-lg text-sm"
            >
              <Trash2 size={16} />
              Remove contact
            </button>
          </div>
        </>
      )}

      {/* Add Contact Modal */}
      {showAddModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-surface-container rounded-3xl p-6 w-full max-w-md shadow-2xl">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-xl font-bold text-on-surface">Add Contact</h3>
              <button
                onClick={() => {
                  setShowAddModal(false);
                  setUserSearch('');
                  setSearchResults([]);
                }}
                className="p-2 rounded-full hover:bg-surface-variant text-on-surface-variant"
              >
                <X size={20} />
              </button>
            </div>

            <div className="relative mb-4">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-on-surface-variant/60" size={18} />
              <input
                type="text"
                placeholder="Search by name or email"
                value={userSearch}
                onChange={(e) => setUserSearch(e.target.value)}
                className="w-full bg-surface-variant/50 text-on-surface rounded-xl py-3 pl-10 pr-4 outline-none focus:ring-2 focus:ring-primary transition-all placeholder:text-on-surface-variant/60"
                autoFocus
              />
            </div>

            <div className="max-h-64 overflow-y-auto space-y-2">
              {searchLoading && (
                <div className="flex justify-center py-4">
                  <Loader2 className="animate-spin text-primary" size={24} />
                </div>
              )}

              {!searchLoading && searchResults.length === 0 && userSearch.length >= 2 && (
                <p className="text-center text-on-surface-variant py-4">No users found</p>
              )}

              {searchResults.map(user => (
                <div
                  key={user.id}
                  className="flex items-center p-3 rounded-xl bg-surface-variant/30"
                >
                  <img
                    src={user.avatar}
                    alt={user.name}
                    className="w-10 h-10 rounded-full object-cover"
                  />
                  <div className="ml-3 flex-1">
                    <h4 className="font-medium text-on-surface">{user.name}</h4>
                    <p className="text-xs text-on-surface-variant">
                      {user.status === 'online' ? 'Online' : 'Offline'}
                    </p>
                  </div>
                  {user.isContact ? (
                    <span className="text-xs text-primary font-medium">Already added</span>
                  ) : (
                    <button
                      onClick={() => handleAddContact(user.id)}
                      className="p-2 rounded-full bg-primary text-on-primary hover:shadow-lg transition-all"
                    >
                      <UserPlus size={16} />
                    </button>
                  )}
                </div>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ContactList;
