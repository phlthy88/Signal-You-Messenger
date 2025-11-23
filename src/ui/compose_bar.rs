//! Message composition bar

use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::glib;
use libadwaita as adw;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/compose_bar.ui")]
    pub struct ComposeBar {
        #[template_child]
        pub text_view: TemplateChild<gtk4::TextView>,

        #[template_child]
        pub send_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub attach_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub emoji_button: TemplateChild<gtk4::MenuButton>,
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
    }

    fn send_message(&self) {
        let imp = self.imp();
        let buffer = imp.text_view.buffer();
        let (start, end) = buffer.bounds();
        let text = buffer.text(&start, &end, false);

        if !text.trim().is_empty() {
            // TODO: Emit signal to parent to send message
            tracing::info!("Compose bar sending: {}", text);
            buffer.set_text("");
        }
    }

    fn show_attachment_dialog(&self) {
        // TODO: Show file chooser for attachments
        tracing::info!("Attachment dialog requested");
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
    }
}

impl Default for ComposeBar {
    fn default() -> Self {
        Self::new()
    }
}
