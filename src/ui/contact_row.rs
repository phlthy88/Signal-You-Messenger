//! Contact row widget for chat list

use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::glib;
use libadwaita as adw;

mod imp {
    use super::*;
    use gtk4::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/contact_row.ui")]
    pub struct ContactRow {
        #[template_child]
        pub avatar: TemplateChild<adw::Avatar>,

        #[template_child]
        pub name_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub message_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub time_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub unread_badge: TemplateChild<gtk4::Label>,

        pub chat_id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContactRow {
        const NAME: &'static str = "ContactRow";
        type Type = super::ContactRow;
        type ParentType = gtk4::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContactRow {}
    impl WidgetImpl for ContactRow {}
    impl ListBoxRowImpl for ContactRow {}
}

glib::wrapper! {
    pub struct ContactRow(ObjectSubclass<imp::ContactRow>)
        @extends gtk4::Widget, gtk4::ListBoxRow,
        @implements gtk4::Buildable;
}

impl ContactRow {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_chat_id(&self, id: &str) {
        self.imp().chat_id.replace(Some(id.to_string()));
    }

    pub fn get_chat_id(&self) -> Option<String> {
        self.imp().chat_id.borrow().clone()
    }

    pub fn set_name(&self, name: &str) {
        let imp = self.imp();
        imp.name_label.set_text(name);
        imp.avatar.set_text(Some(name));
    }

    pub fn set_last_message(&self, message: &str) {
        self.imp().message_label.set_text(message);
    }

    pub fn set_time(&self, time: &str) {
        self.imp().time_label.set_text(time);
    }

    pub fn set_unread_count(&self, count: u32) {
        let imp = self.imp();
        if count > 0 {
            imp.unread_badge.set_text(&count.to_string());
            imp.unread_badge.set_visible(true);
        } else {
            imp.unread_badge.set_visible(false);
        }
    }

    pub fn set_avatar_icon(&self, icon_name: Option<&str>) {
        self.imp().avatar.set_icon_name(icon_name);
    }
}

impl Default for ContactRow {
    fn default() -> Self {
        Self::new()
    }
}
