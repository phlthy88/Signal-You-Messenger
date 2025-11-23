//! Chat list sidebar component

use gtk4::prelude::*;
use gtk4::glib;
use libadwaita as adw;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/chat_list.ui")]
    pub struct ChatList {
        #[template_child]
        pub list_box: TemplateChild<gtk4::ListBox>,

        #[template_child]
        pub search_entry: TemplateChild<gtk4::SearchEntry>,

        #[template_child]
        pub header_bar: TemplateChild<adw::HeaderBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChatList {
        const NAME: &'static str = "ChatList";
        type Type = super::ChatList;
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
    impl ChatList {
        #[template_callback]
        fn on_row_activated(&self, row: &gtk4::ListBoxRow) {
            // TODO: Emit signal with chat ID
            tracing::info!("Chat row activated: {:?}", row.index());
        }

        #[template_callback]
        fn on_search_changed(&self, entry: &gtk4::SearchEntry) {
            let text = entry.text();
            tracing::info!("Search text: {}", text);
            // TODO: Filter chat list
        }
    }

    impl ObjectImpl for ChatList {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }

    impl WidgetImpl for ChatList {}
    impl BinImpl for ChatList {}
}

glib::wrapper! {
    pub struct ChatList(ObjectSubclass<imp::ChatList>)
        @extends gtk4::Widget, adw::Bin,
        @implements gtk4::Buildable;
}

impl ChatList {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup(&self) {
        // TODO: Load chats from Signal service
        self.load_chats();
    }

    fn load_chats(&self) {
        // TODO: Fetch chats from Signal and populate list
        tracing::info!("Loading chats from Signal service");
    }

    pub fn refresh(&self) {
        self.load_chats();
    }
}

impl Default for ChatList {
    fn default() -> Self {
        Self::new()
    }
}
