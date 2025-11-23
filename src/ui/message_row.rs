//! Individual message row widget

use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;

mod imp {
    use super::*;
    use gtk4::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/message_row.ui")]
    pub struct MessageRow {
        #[template_child]
        pub content_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub time_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub status_icon: TemplateChild<gtk4::Image>,

        #[template_child]
        pub avatar: TemplateChild<adw::Avatar>,

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
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MessageRow {}
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

    pub fn set_content(&self, content: &str) {
        self.imp().content_label.set_text(content);
    }

    pub fn set_time(&self, time: &str) {
        self.imp().time_label.set_text(time);
    }

    pub fn set_outgoing(&self, outgoing: bool) {
        let imp = self.imp();
        imp.is_outgoing.replace(outgoing);

        // Style based on direction
        if outgoing {
            self.add_css_class("outgoing-message");
            imp.avatar.set_visible(false);
        } else {
            self.add_css_class("incoming-message");
            imp.avatar.set_visible(true);
        }
    }

    pub fn set_status(&self, status: MessageStatus) {
        let imp = self.imp();
        let icon_name = match status {
            MessageStatus::Sending => "mail-send-symbolic",
            MessageStatus::Sent => "emblem-ok-symbolic",
            MessageStatus::Delivered => "mail-read-symbolic",
            MessageStatus::Read => "eye-open-symbolic",
            MessageStatus::Failed => "dialog-error-symbolic",
        };
        imp.status_icon.set_icon_name(Some(icon_name));
    }

    pub fn set_sender_name(&self, name: &str) {
        self.imp().avatar.set_text(Some(name));
    }
}

impl Default for MessageRow {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Read,
    Failed,
}
