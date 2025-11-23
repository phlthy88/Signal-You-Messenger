import React, { useState } from 'react';
import { User } from '../types';
import { Search, UserPlus, MoreVertical } from 'lucide-react';

interface ContactListProps {
  contacts: User[];
  onSelectContact: (contact: User) => void;
}

const ContactList: React.FC<ContactListProps> = ({ contacts, onSelectContact }) => {
  const [searchTerm, setSearchTerm] = useState('');

  const filteredContacts = contacts.filter(contact => 
    contact.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="flex flex-col h-full bg-surface-variant/30 border-r border-outline-variant/20 w-80 md:w-96">
      {/* Header */}
      <div className="p-4 flex items-center justify-between">
        <h2 className="text-2xl font-bold text-on-surface">Contacts</h2>
        <button className="p-2 rounded-full bg-tertiary-container text-on-tertiary-container hover:shadow-lg transition-all" title="Add Contact">
          <UserPlus size={24} />
        </button>
      </div>

      {/* Search Bar */}
      <div className="px-4 pb-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-on-surface-variant/60" size={18} />
          <input 
            type="text" 
            placeholder="Search people"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full bg-surface-variant/50 text-on-surface rounded-full py-3 pl-10 pr-4 outline-none focus:ring-2 focus:ring-primary/50 transition-all placeholder:text-on-surface-variant/60"
          />
        </div>
      </div>

      {/* Contacts List */}
      <div className="flex-1 overflow-y-auto px-2 space-y-1 pb-4">
        <div className="px-4 py-2 text-xs font-bold text-primary uppercase tracking-wider">
          {filteredContacts.length} Contacts
        </div>
        
        {filteredContacts.map((contact) => (
          <button
            key={contact.id}
            onClick={() => onSelectContact(contact)}
            className="w-full flex items-center p-3 rounded-[1.5rem] hover:bg-surface-variant/40 text-on-surface transition-colors group"
          >
            <div className="relative">
              <img 
                src={contact.avatar} 
                alt={contact.name} 
                className="w-12 h-12 rounded-full object-cover border border-outline-variant/20"
              />
              {contact.status === 'online' && (
                <span className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 border-2 border-surface rounded-full"></span>
              )}
            </div>
            
            <div className="ml-4 flex-1 text-left">
              <div className="flex items-center gap-2">
                <h3 className="font-semibold text-base text-on-surface group-hover:text-on-surface-variant transition-colors">
                  {contact.name}
                </h3>
                {contact.status === 'online' && (
                  <span className="w-2 h-2 bg-green-500 rounded-full" title="Online"></span>
                )}
              </div>
              <p className="text-sm text-on-surface-variant/70">
                {contact.status === 'online' ? 'Online' : 'Last seen recently'}
              </p>
            </div>

            <div className="opacity-0 group-hover:opacity-100 transition-opacity">
               <div className="p-2 hover:bg-surface-variant rounded-full text-on-surface-variant">
                 <MoreVertical size={16} />
               </div>
            </div>
          </button>
        ))}

        {filteredContacts.length === 0 && (
          <div className="text-center py-10 text-on-surface-variant/60">
            <p>No contacts found.</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default ContactList;