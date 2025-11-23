//! Individual message row widget
//!
//! Displays a single message with support for:
//! - Read receipts (status icons)
//! - Message reactions
//! - Reply/quote previews
//! - Disappearing message timers

use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::glib;
use libadwaita as adw;

use crate::signal::types::{Message, MessageContent, MessageStatus, Reaction};

mod imp {
    use super::*;
    use gtk4::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/message_row.ui")]
    pub struct MessageRow {
        #[template_child]
        pub avatar: TemplateChild<adw::Avatar>,

        #[template_child]
        pub content_box: TemplateChild<gtk4::Box>,

        #[template_child]
        pub sender_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub quote_box: TemplateChild<gtk4::Box>,

        #[template_child]
        pub quote_sender_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub quote_content_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub bubble: TemplateChild<gtk4::Box>,

        #[template_child]
        pub message_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub expiry_box: TemplateChild<gtk4::Box>,

        #[template_child]
        pub expiry_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub time_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub status_icon: TemplateChild<gtk4::Image>,

        #[template_child]
        pub reactions_box: TemplateChild<gtk4::FlowBox>,

        pub is_outgoing: RefCell<bool>,
        pub message_id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "MessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk4::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk4::template_callbacks]
    impl MessageRow {
        #[template_callback]
        fn on_reaction_clicked(&self, _flow_box: &gtk4::FlowBox, child: &gtk4::FlowBoxChild) {
            // Handle reaction click (toggle own reaction)
            if let Some(label) = child.child().and_downcast::<gtk4::Label>() {
                tracing::debug!("Reaction clicked: {}", label.text());
                // TODO: Emit signal to toggle reaction
            }
        }
    }

    impl ObjectImpl for MessageRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_gestures();
        }
    }

    impl WidgetImpl for MessageRow {}
    impl BoxImpl for MessageRow {}
}

glib::wrapper! {
    pub struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Buildable;
}

impl MessageRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Create a message row from a Message struct
    pub fn from_message(message: &Message, is_outgoing: bool) -> Self {
        let row = Self::new();
        row.set_message(message, is_outgoing);
        row
    }

    fn setup_gestures(&self) {
        // Long press to show context menu (reactions, reply, etc.)
        let gesture = gtk4::GestureLongPress::new();
        gesture.connect_pressed(glib::clone!(
            @weak self as row => move |_, _x, _y| {
                row.show_context_menu();
            }
        ));
        self.imp().bubble.add_controller(gesture);

        // Right-click context menu
        let click_gesture = gtk4::GestureClick::builder()
            .button(3) // Right mouse button
            .build();
        click_gesture.connect_released(glib::clone!(
            @weak self as row => move |_, _, x, y| {
                row.show_context_menu_at(x, y);
            }
        ));
        self.imp().bubble.add_controller(click_gesture);
    }

    /// Set message content from a Message struct
    pub fn set_message(&self, message: &Message, is_outgoing: bool) {
        let imp = self.imp();
        imp.message_id.replace(Some(message.id.clone()));

        // Set content based on message type
        match &message.content {
            MessageContent::Text { body } => {
                imp.message_label.set_text(body);
            }
            MessageContent::Image { caption, .. } => {
                imp.message_label.set_text(caption.as_deref().unwrap_or("[Image]"));
                // TODO: Show image thumbnail
            }
            MessageContent::Video { caption, .. } => {
                imp.message_label.set_text(caption.as_deref().unwrap_or("[Video]"));
            }
            MessageContent::Audio { .. } => {
                imp.message_label.set_text("[Audio]");
            }
            MessageContent::Voice { duration_ms, .. } => {
                let seconds = duration_ms / 1000;
                imp.message_label.set_text(&format!("[Voice message {}:{:02}]", seconds / 60, seconds % 60));
            }
            MessageContent::File { attachment } => {
                let name = attachment.file_name.as_deref().unwrap_or("file");
                imp.message_label.set_text(&format!("[File: {}]", name));
            }
            MessageContent::Sticker { .. } => {
                imp.message_label.set_text("[Sticker]");
            }
            MessageContent::Contact { contact } => {
                imp.message_label.set_text(&format!("[Contact: {}]", contact.name));
            }
            MessageContent::Location { name, .. } => {
                let label = name.as_deref().unwrap_or("Shared location");
                imp.message_label.set_text(&format!("[Location: {}]", label));
            }
        }

        // Set time
        self.set_timestamp(message.timestamp);

        // Set outgoing status
        self.set_outgoing(is_outgoing);

        // Set delivery status for outgoing messages
        if is_outgoing {
            self.set_status(message.status);
        }

        // Set quote if present
        if let Some(quote) = &message.quote {
            self.set_quote(Some(&quote.sender.uuid.to_string()), Some(&Self::get_content_preview(&quote.content)));
        }

        // Set reactions
        if !message.reactions.is_empty() {
            self.set_reactions(&message.reactions);
        }

        // Set expiry if disappearing message
        if let Some(expires_at) = message.expires_at {
            self.set_expiry(Some(expires_at));
        }
    }

    fn get_content_preview(content: &MessageContent) -> String {
        match content {
            MessageContent::Text { body } => {
                if body.len() > 50 {
                    format!("{}...", &body[..50])
                } else {
                    body.clone()
                }
            }
            MessageContent::Image { .. } => "Photo".to_string(),
            MessageContent::Video { .. } => "Video".to_string(),
            MessageContent::Audio { .. } => "Audio".to_string(),
            MessageContent::Voice { .. } => "Voice message".to_string(),
            MessageContent::File { attachment } => {
                attachment.file_name.clone().unwrap_or_else(|| "File".to_string())
            }
            MessageContent::Sticker { .. } => "Sticker".to_string(),
            MessageContent::Contact { contact } => contact.name.clone(),
            MessageContent::Location { .. } => "Location".to_string(),
        }
    }

    pub fn set_content(&self, content: &str) {
        self.imp().message_label.set_text(content);
    }

    pub fn set_timestamp(&self, timestamp: i64) {
        let datetime = chrono::DateTime::from_timestamp_millis(timestamp)
            .unwrap_or_else(chrono::Utc::now);
        let time_str = datetime.format("%H:%M").to_string();
        self.imp().time_label.set_text(&time_str);
    }

    pub fn set_time(&self, time: &str) {
        self.imp().time_label.set_text(time);
    }

    pub fn set_outgoing(&self, outgoing: bool) {
        let imp = self.imp();
        imp.is_outgoing.replace(outgoing);

        // Clear previous styles
        self.remove_css_class("outgoing-message");
        self.remove_css_class("incoming-message");
        imp.bubble.remove_css_class("outgoing-bubble");
        imp.bubble.remove_css_class("incoming-bubble");

        if outgoing {
            self.add_css_class("outgoing-message");
            imp.bubble.add_css_class("outgoing-bubble");
            imp.avatar.set_visible(false);
            imp.status_icon.set_visible(true);
            // Align content to the right
            imp.content_box.set_halign(gtk4::Align::End);
        } else {
            self.add_css_class("incoming-message");
            imp.bubble.add_css_class("incoming-bubble");
            imp.avatar.set_visible(true);
            imp.status_icon.set_visible(false);
            // Align content to the left
            imp.content_box.set_halign(gtk4::Align::Start);
        }
    }

    pub fn set_status(&self, status: MessageStatus) {
        let imp = self.imp();

        let (icon_name, css_class) = match status {
            MessageStatus::Sending => ("mail-send-symbolic", "status-sending"),
            MessageStatus::Sent => ("emblem-ok-symbolic", "status-sent"),
            MessageStatus::Delivered => ("mail-delivered-symbolic", "status-delivered"),
            MessageStatus::Read => ("eye-open-symbolic", "status-read"),
            MessageStatus::Failed => ("dialog-error-symbolic", "status-failed"),
        };

        imp.status_icon.set_icon_name(Some(icon_name));
        imp.status_icon.set_visible(true);

        // Update CSS class for coloring
        imp.status_icon.remove_css_class("status-sending");
        imp.status_icon.remove_css_class("status-sent");
        imp.status_icon.remove_css_class("status-delivered");
        imp.status_icon.remove_css_class("status-read");
        imp.status_icon.remove_css_class("status-failed");
        imp.status_icon.add_css_class(css_class);
    }

    pub fn set_sender_name(&self, name: &str) {
        let imp = self.imp();
        imp.avatar.set_text(Some(name));
        imp.sender_label.set_text(name);
        imp.sender_label.set_visible(true);
    }

    /// Set the quoted/replied message preview
    pub fn set_quote(&self, sender: Option<&str>, content: Option<&str>) {
        let imp = self.imp();

        if let (Some(sender), Some(content)) = (sender, content) {
            imp.quote_sender_label.set_text(sender);
            imp.quote_content_label.set_text(content);
            imp.quote_box.set_visible(true);
        } else {
            imp.quote_box.set_visible(false);
        }
    }

    /// Set message reactions
    pub fn set_reactions(&self, reactions: &[Reaction]) {
        let imp = self.imp();

        // Clear existing reactions
        while let Some(child) = imp.reactions_box.first_child() {
            imp.reactions_box.remove(&child);
        }

        if reactions.is_empty() {
            imp.reactions_box.set_visible(false);
            return;
        }

        // Group reactions by emoji and count
        let mut emoji_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for reaction in reactions {
            *emoji_counts.entry(reaction.emoji.clone()).or_insert(0) += 1;
        }

        // Create reaction badges
        for (emoji, count) in emoji_counts {
            let badge = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .css_classes(["reaction-badge"])
                .build();

            let emoji_label = gtk4::Label::new(Some(&emoji));
            badge.append(&emoji_label);

            if count > 1 {
                let count_label = gtk4::Label::builder()
                    .label(&count.to_string())
                    .css_classes(["caption"])
                    .build();
                badge.append(&count_label);
            }

            imp.reactions_box.append(&badge);
        }

        imp.reactions_box.set_visible(true);
    }

    /// Add a single reaction
    pub fn add_reaction(&self, emoji: &str, sender: &str) {
        tracing::debug!("Adding reaction {} from {}", emoji, sender);
        // TODO: Update reactions list and refresh display
    }

    /// Set disappearing message expiry time
    pub fn set_expiry(&self, expires_at: Option<i64>) {
        let imp = self.imp();

        if let Some(expires_at) = expires_at {
            let now = chrono::Utc::now().timestamp_millis();
            let remaining = expires_at - now;

            if remaining > 0 {
                let remaining_secs = remaining / 1000;
                let label = if remaining_secs < 60 {
                    format!("{}s", remaining_secs)
                } else if remaining_secs < 3600 {
                    format!("{}m", remaining_secs / 60)
                } else if remaining_secs < 86400 {
                    format!("{}h", remaining_secs / 3600)
                } else {
                    format!("{}d", remaining_secs / 86400)
                };

                imp.expiry_label.set_text(&label);
                imp.expiry_box.set_visible(true);
            } else {
                // Message has expired
                imp.expiry_label.set_text("Expired");
                imp.expiry_box.set_visible(true);
            }
        } else {
            imp.expiry_box.set_visible(false);
        }
    }

    fn show_context_menu(&self) {
        self.show_context_menu_at(0.0, 0.0);
    }

    fn show_context_menu_at(&self, _x: f64, _y: f64) {
        let menu = gio::Menu::new();

        // Reaction quick access
        let reactions_section = gio::Menu::new();
        for emoji in &["👍", "❤️", "😂", "😮", "😢", "😡"] {
            reactions_section.append(Some(emoji), Some(&format!("message.react::{}", emoji)));
        }
        menu.append_section(None, &reactions_section);

        // Actions section
        let actions_section = gio::Menu::new();
        actions_section.append(Some("Reply"), Some("message.reply"));
        actions_section.append(Some("Forward"), Some("message.forward"));
        actions_section.append(Some("Copy"), Some("message.copy"));

        // Only show delete for own messages
        if *self.imp().is_outgoing.borrow() {
            actions_section.append(Some("Delete"), Some("message.delete"));
        }

        menu.append_section(None, &actions_section);

        let popover = gtk4::PopoverMenu::from_model(Some(&menu));
        popover.set_parent(&*self.imp().bubble);
        popover.popup();
    }

    /// Get the message ID
    pub fn message_id(&self) -> Option<String> {
        self.imp().message_id.borrow().clone()
    }
}

impl Default for MessageRow {
    fn default() -> Self {
        Self::new()
    }
}

use gtk4::gio;
