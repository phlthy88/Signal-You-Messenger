//! Main application window

use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::{gio, glib};
use libadwaita as adw;

use crate::ui::{ChatList, ChatView, LinkDeviceView};

mod imp {
    use super::*;
    use adw::subclass::prelude::*;
    use std::cell::RefCell;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/window.ui")]
    pub struct SignalYouWindow {
        #[template_child]
        pub split_view: TemplateChild<adw::NavigationSplitView>,

        #[template_child]
        pub chat_list: TemplateChild<ChatList>,

        #[template_child]
        pub chat_view: TemplateChild<ChatView>,

        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,

        pub is_linked: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SignalYouWindow {
        const NAME: &'static str = "SignalYouWindow";
        type Type = super::SignalYouWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            ChatList::ensure_type();
            ChatView::ensure_type();
            LinkDeviceView::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk4::template_callbacks]
    impl SignalYouWindow {
        #[template_callback]
        fn on_chat_selected(&self, chat_id: Option<String>) {
            if let Some(id) = chat_id {
                self.chat_view.load_chat(&id);
                self.split_view.set_show_content(true);
            }
        }
    }

    impl ObjectImpl for SignalYouWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_actions();
            self.obj().check_device_linked();
        }
    }

    impl WidgetImpl for SignalYouWindow {}
    impl WindowImpl for SignalYouWindow {}
    impl ApplicationWindowImpl for SignalYouWindow {}
    impl AdwApplicationWindowImpl for SignalYouWindow {}
}

glib::wrapper! {
    pub struct SignalYouWindow(ObjectSubclass<imp::SignalYouWindow>)
        @extends gtk4::Widget, gtk4::Window, gtk4::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SignalYouWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .property("title", "Signal You")
            .property("default-width", 1000)
            .property("default-height", 700)
            .build()
    }

    fn setup_actions(&self) {
        // New chat action
        let action_new_chat = gio::ActionEntry::builder("new-chat")
            .activate(move |window: &Self, _, _| {
                window.show_new_chat_dialog();
            })
            .build();

        // Search action
        let action_search = gio::ActionEntry::builder("search")
            .activate(move |window: &Self, _, _| {
                window.toggle_search();
            })
            .build();

        self.add_action_entries([action_new_chat, action_search]);
    }

    fn check_device_linked(&self) {
        // TODO: Check if device is linked to Signal account
        // If not linked, show LinkDeviceView instead of chat list
        let imp = self.imp();
        let is_linked = imp.is_linked.borrow();

        if !*is_linked {
            self.show_link_device_view();
        }
    }

    fn show_link_device_view(&self) {
        // TODO: Show QR code for linking device to Signal
        tracing::info!("Device not linked, showing link device view");
    }

    fn show_new_chat_dialog(&self) {
        // TODO: Implement new chat dialog
        tracing::info!("New chat dialog requested");
    }

    fn toggle_search(&self) {
        // TODO: Implement search toggle
        tracing::info!("Search toggled");
    }

    pub fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.imp().toast_overlay.add_toast(toast);
    }
}
