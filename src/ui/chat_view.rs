//! Chat view component for displaying messages

use gtk4::prelude::*;
use gtk4::{gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;

use super::ComposeBar;

mod imp {
    use super::*;
    use gtk4::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/chat_view.ui")]
    pub struct ChatView {
        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,

        #[template_child]
        pub title_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub status_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub message_list: TemplateChild<gtk4::ListView>,

        #[template_child]
        pub compose_bar: TemplateChild<ComposeBar>,

        #[template_child]
        pub scrolled_window: TemplateChild<gtk4::ScrolledWindow>,

        pub current_chat_id: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatView {
        const NAME: &'static str = "ChatView";
        type Type = super::ChatView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            ComposeBar::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk4::template_callbacks]
    impl ChatView {}

    impl ObjectImpl for ChatView {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for ChatView {}
    impl BinImpl for ChatView {}
}

glib::wrapper! {
    pub struct ChatView(ObjectSubclass<imp::ChatView>)
        @extends gtk4::Widget, adw::Bin,
        @implements gtk4::Buildable;
}

impl ChatView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn load_chat(&self, chat_id: &str) {
        let imp = self.imp();
        imp.current_chat_id.replace(Some(chat_id.to_string()));

        // TODO: Load messages from Signal service
        tracing::info!("Loading chat: {}", chat_id);
        self.load_messages(chat_id);
    }

    fn load_messages(&self, chat_id: &str) {
        // TODO: Fetch messages from Signal service
        tracing::info!("Loading messages for chat: {}", chat_id);
    }

    pub fn scroll_to_bottom(&self) {
        let imp = self.imp();
        let adj = imp.scrolled_window.vadjustment();
        adj.set_value(adj.upper() - adj.page_size());
    }

    pub fn send_message(&self, content: &str) {
        let imp = self.imp();
        if let Some(chat_id) = imp.current_chat_id.borrow().as_ref() {
            // TODO: Send message via Signal protocol
            tracing::info!("Sending message to {}: {}", chat_id, content);
        }
    }
}

impl Default for ChatView {
    fn default() -> Self {
        Self::new()
    }
}
