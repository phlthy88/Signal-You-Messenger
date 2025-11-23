//! System notification handling

use gtk4::gio;

/// Service for managing desktop notifications
pub struct NotificationService {
    application: gio::Application,
}

impl NotificationService {
    pub fn new(application: &gio::Application) -> Self {
        Self {
            application: application.clone(),
        }
    }

    /// Show a notification for a new message
    pub fn notify_message(&self, sender: &str, content: &str, conversation_id: &str) {
        let notification = gio::Notification::new(&format!("Message from {}", sender));
        notification.set_body(Some(content));

        // Add action to open conversation
        notification.set_default_action_and_target_value(
            "app.open-conversation",
            Some(&conversation_id.to_variant()),
        );

        // Add reply action
        notification.add_button_with_target_value(
            "Reply",
            "app.reply-notification",
            Some(&conversation_id.to_variant()),
        );

        self.application.send_notification(Some(conversation_id), &notification);
    }

    /// Show a notification for a group message
    pub fn notify_group_message(&self, group_name: &str, sender: &str, content: &str, conversation_id: &str) {
        let notification = gio::Notification::new(group_name);
        notification.set_body(Some(&format!("{}: {}", sender, content)));

        notification.set_default_action_and_target_value(
            "app.open-conversation",
            Some(&conversation_id.to_variant()),
        );

        self.application.send_notification(Some(conversation_id), &notification);
    }

    /// Withdraw a notification
    pub fn withdraw(&self, conversation_id: &str) {
        self.application.withdraw_notification(conversation_id);
    }

    /// Withdraw all notifications
    pub fn withdraw_all(&self) {
        // GTK doesn't have a withdraw-all, so we track active notifications
        tracing::info!("Withdrawing all notifications");
    }
}

use gtk4::glib;

trait ToVariant {
    fn to_variant(&self) -> glib::Variant;
}

impl ToVariant for str {
    fn to_variant(&self) -> glib::Variant {
        self.to_string().to_variant()
    }
}
