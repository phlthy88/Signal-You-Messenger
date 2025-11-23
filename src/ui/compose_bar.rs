//! Message composition bar
//!
//! Provides message input with support for:
//! - Reply/quote preview
//! - Disappearing message timer selection
//! - File attachments
//! - Emoji picker

use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::glib;
use libadwaita as adw;

use crate::signal::types::Message;

/// Disappearing message timer durations in seconds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisappearingTimer {
    #[default]
    Off,
    Seconds30,
    Minutes5,
    Hour1,
    Hours8,
    Day1,
    Week1,
    Weeks4,
}

impl DisappearingTimer {
    /// Get duration in seconds (0 means disabled)
    pub fn duration_seconds(&self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Seconds30 => 30,
            Self::Minutes5 => 5 * 60,
            Self::Hour1 => 60 * 60,
            Self::Hours8 => 8 * 60 * 60,
            Self::Day1 => 24 * 60 * 60,
            Self::Week1 => 7 * 24 * 60 * 60,
            Self::Weeks4 => 4 * 7 * 24 * 60 * 60,
        }
    }

    /// Get human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Seconds30 => "30 seconds",
            Self::Minutes5 => "5 minutes",
            Self::Hour1 => "1 hour",
            Self::Hours8 => "8 hours",
            Self::Day1 => "1 day",
            Self::Week1 => "1 week",
            Self::Weeks4 => "4 weeks",
        }
    }
}

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::RefCell;

    /// Reply context for when replying to a message
    #[derive(Debug, Default, Clone)]
    pub struct ReplyContext {
        pub message_id: String,
        pub sender_name: String,
        pub content_preview: String,
    }

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/compose_bar.ui")]
    pub struct ComposeBar {
        #[template_child]
        pub reply_revealer: TemplateChild<gtk4::Revealer>,

        #[template_child]
        pub reply_sender_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub reply_content_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub disappearing_revealer: TemplateChild<gtk4::Revealer>,

        #[template_child]
        pub disappearing_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub text_view: TemplateChild<gtk4::TextView>,

        #[template_child]
        pub send_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub attach_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub timer_button: TemplateChild<gtk4::MenuButton>,

        #[template_child]
        pub emoji_button: TemplateChild<gtk4::MenuButton>,

        #[template_child]
        pub timer_off: TemplateChild<gtk4::CheckButton>,

        #[template_child]
        pub timer_30s: TemplateChild<gtk4::CheckButton>,

        #[template_child]
        pub timer_5m: TemplateChild<gtk4::CheckButton>,

        #[template_child]
        pub timer_1h: TemplateChild<gtk4::CheckButton>,

        #[template_child]
        pub timer_8h: TemplateChild<gtk4::CheckButton>,

        #[template_child]
        pub timer_1d: TemplateChild<gtk4::CheckButton>,

        #[template_child]
        pub timer_1w: TemplateChild<gtk4::CheckButton>,

        #[template_child]
        pub timer_4w: TemplateChild<gtk4::CheckButton>,

        pub reply_context: RefCell<Option<ReplyContext>>,
        pub disappearing_timer: RefCell<DisappearingTimer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ComposeBar {
        const NAME: &'static str = "ComposeBar";
        type Type = super::ComposeBar;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk4::template_callbacks]
    impl ComposeBar {
        #[template_callback]
        fn on_send_clicked(&self, _button: &gtk4::Button) {
            self.obj().send_message();
        }

        #[template_callback]
        fn on_attach_clicked(&self, _button: &gtk4::Button) {
            self.obj().show_attachment_dialog();
        }

        #[template_callback]
        fn on_cancel_reply_clicked(&self, _button: &gtk4::Button) {
            self.obj().cancel_reply();
        }

        #[template_callback]
        fn on_timer_toggled(&self, button: &gtk4::CheckButton) {
            if !button.is_active() {
                return;
            }

            let timer = if button == &*self.timer_off {
                DisappearingTimer::Off
            } else if button == &*self.timer_30s {
                DisappearingTimer::Seconds30
            } else if button == &*self.timer_5m {
                DisappearingTimer::Minutes5
            } else if button == &*self.timer_1h {
                DisappearingTimer::Hour1
            } else if button == &*self.timer_8h {
                DisappearingTimer::Hours8
            } else if button == &*self.timer_1d {
                DisappearingTimer::Day1
            } else if button == &*self.timer_1w {
                DisappearingTimer::Week1
            } else if button == &*self.timer_4w {
                DisappearingTimer::Weeks4
            } else {
                DisappearingTimer::Off
            };

            self.obj().set_disappearing_timer(timer);
        }
    }

    impl ObjectImpl for ComposeBar {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }

    impl WidgetImpl for ComposeBar {}
    impl BinImpl for ComposeBar {}
}

glib::wrapper! {
    pub struct ComposeBar(ObjectSubclass<imp::ComposeBar>)
        @extends gtk4::Widget, adw::Bin,
        @implements gtk4::Buildable;
}

impl ComposeBar {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup(&self) {
        // Setup key bindings for send on Enter
        let imp = self.imp();

        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed(glib::clone!(
            @weak self as compose_bar => @default-return glib::Propagation::Proceed,
            move |_, key, _, modifier| {
                if key == gtk4::gdk::Key::Return
                    && !modifier.contains(gtk4::gdk::ModifierType::SHIFT_MASK)
                {
                    compose_bar.send_message();
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));

        imp.text_view.add_controller(controller);

        // Monitor text changes to enable/disable send button
        let buffer = imp.text_view.buffer();
        buffer.connect_changed(glib::clone!(
            @weak self as compose_bar => move |buffer| {
                let has_text = buffer.char_count() > 0;
                compose_bar.imp().send_button.set_sensitive(has_text);
            }
        ));

        // Initial state
        imp.send_button.set_sensitive(false);
    }

    fn send_message(&self) {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        let (start, end) = buffer.bounds();
        let text = buffer.text(&start, &end, false);

        if !text.trim().is_empty() {
            // Get reply context if replying
            let reply_context = imp.reply_context.borrow().clone();
            let disappearing_timer = *imp.disappearing_timer.borrow();

            // Emit signal with message data
            tracing::info!(
                "Sending message: {} (reply: {:?}, timer: {:?})",
                text,
                reply_context.as_ref().map(|r| &r.message_id),
                disappearing_timer
            );

            // Clear the input
            buffer.set_text("");

            // Clear reply context
            self.cancel_reply();
        }
    }

    fn show_attachment_dialog(&self) {
        tracing::info!("Attachment dialog requested");

        let dialog = gtk4::FileDialog::builder()
            .title("Select Attachment")
            .modal(true)
            .build();

        dialog.open(
            self.root().and_downcast_ref::<gtk4::Window>(),
            None::<&gtk4::gio::Cancellable>,
            glib::clone!(
                @weak self as compose_bar => move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            tracing::info!("Selected attachment: {:?}", path);
                            // TODO: Handle file attachment
                        }
                    }
                }
            ),
        );
    }

    /// Set reply mode with the message being replied to
    pub fn set_reply_to(&self, message: &Message) {
        let imp = self.imp();

        // Get sender name (use UUID as fallback)
        let sender_name = message.sender.phone_number
            .clone()
            .unwrap_or_else(|| message.sender.uuid.to_string());

        // Get content preview
        let content_preview = match &message.content {
            crate::signal::types::MessageContent::Text { body } => {
                if body.len() > 50 {
                    format!("{}...", &body[..50])
                } else {
                    body.clone()
                }
            }
            crate::signal::types::MessageContent::Image { .. } => "Photo".to_string(),
            crate::signal::types::MessageContent::Video { .. } => "Video".to_string(),
            crate::signal::types::MessageContent::Audio { .. } => "Audio".to_string(),
            crate::signal::types::MessageContent::Voice { .. } => "Voice message".to_string(),
            crate::signal::types::MessageContent::File { attachment } => {
                attachment.file_name.clone().unwrap_or_else(|| "File".to_string())
            }
            crate::signal::types::MessageContent::Sticker { .. } => "Sticker".to_string(),
            crate::signal::types::MessageContent::Contact { contact } => contact.name.clone(),
            crate::signal::types::MessageContent::Location { .. } => "Location".to_string(),
        };

        // Store reply context
        imp.reply_context.replace(Some(imp::ReplyContext {
            message_id: message.id.clone(),
            sender_name: sender_name.clone(),
            content_preview: content_preview.clone(),
        }));

        // Update UI
        imp.reply_sender_label.set_text(&sender_name);
        imp.reply_content_label.set_text(&content_preview);
        imp.reply_revealer.set_reveal_child(true);

        // Focus text input
        imp.text_view.grab_focus();
    }

    /// Cancel reply mode
    pub fn cancel_reply(&self) {
        let imp = self.imp();
        imp.reply_context.replace(None);
        imp.reply_revealer.set_reveal_child(false);
    }

    /// Get the current reply context (message ID being replied to)
    pub fn reply_message_id(&self) -> Option<String> {
        self.imp().reply_context.borrow().as_ref().map(|r| r.message_id.clone())
    }

    /// Set the disappearing message timer
    pub fn set_disappearing_timer(&self, timer: DisappearingTimer) {
        let imp = self.imp();
        imp.disappearing_timer.replace(timer);

        if timer == DisappearingTimer::Off {
            imp.disappearing_revealer.set_reveal_child(false);
            imp.timer_button.remove_css_class("accent");
        } else {
            imp.disappearing_label.set_text(&format!("Messages disappear after {}", timer.label()));
            imp.disappearing_revealer.set_reveal_child(true);
            imp.timer_button.add_css_class("accent");
        }

        tracing::info!("Disappearing timer set to: {:?}", timer);
    }

    /// Get the current disappearing timer setting
    pub fn disappearing_timer(&self) -> DisappearingTimer {
        *self.imp().disappearing_timer.borrow()
    }

    /// Get the expiry timestamp if disappearing messages are enabled
    pub fn expiry_timestamp(&self) -> Option<i64> {
        let timer = *self.imp().disappearing_timer.borrow();
        if timer == DisappearingTimer::Off {
            None
        } else {
            let duration_ms = timer.duration_seconds() as i64 * 1000;
            Some(chrono::Utc::now().timestamp_millis() + duration_ms)
        }
    }

    pub fn get_text(&self) -> String {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        let (start, end) = buffer.bounds();
        buffer.text(&start, &end, false).to_string()
    }

    pub fn clear(&self) {
        let imp = self.imp();
        imp.text_view.buffer().set_text("");
        self.cancel_reply();
    }

    /// Focus the text input
    pub fn focus_input(&self) {
        self.imp().text_view.grab_focus();
    }
}

impl Default for ComposeBar {
    fn default() -> Self {
        Self::new()
    }
}
