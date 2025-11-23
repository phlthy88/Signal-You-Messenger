//! Device linking view with QR code

use gtk4::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use gtk4::glib;
use libadwaita as adw;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Debug, Default, gtk4::CompositeTemplate)]
    #[template(resource = "/com/signalyou/Messenger/ui/link_device_view.ui")]
    pub struct LinkDeviceView {
        #[template_child]
        pub qr_code_image: TemplateChild<gtk4::Picture>,

        #[template_child]
        pub status_label: TemplateChild<gtk4::Label>,

        #[template_child]
        pub refresh_button: TemplateChild<gtk4::Button>,

        #[template_child]
        pub spinner: TemplateChild<gtk4::Spinner>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LinkDeviceView {
        const NAME: &'static str = "LinkDeviceView";
        type Type = super::LinkDeviceView;
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
    impl LinkDeviceView {
        #[template_callback]
        fn on_refresh_clicked(&self, _button: &gtk4::Button) {
            self.obj().generate_qr_code();
        }
    }

    impl ObjectImpl for LinkDeviceView {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }

    impl WidgetImpl for LinkDeviceView {}
    impl BinImpl for LinkDeviceView {}
}

glib::wrapper! {
    pub struct LinkDeviceView(ObjectSubclass<imp::LinkDeviceView>)
        @extends gtk4::Widget, adw::Bin,
        @implements gtk4::Buildable;
}

impl LinkDeviceView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    fn setup(&self) {
        self.generate_qr_code();
    }

    pub fn generate_qr_code(&self) {
        let imp = self.imp();

        // Show loading state
        imp.spinner.set_visible(true);
        imp.spinner.set_spinning(true);
        imp.qr_code_image.set_visible(false);
        imp.status_label.set_text("Generating QR code...");

        // TODO: Generate actual QR code from Signal linking URI
        // This requires:
        // 1. Generate key pair for this device
        // 2. Create provisioning URI
        // 3. Generate QR code image
        // 4. Listen for provisioning response

        glib::spawn_future_local(glib::clone!(
            @weak self as view => async move {
                // Simulate QR code generation
                glib::timeout_future(std::time::Duration::from_secs(1)).await;

                let imp = view.imp();
                imp.spinner.set_spinning(false);
                imp.spinner.set_visible(false);
                imp.qr_code_image.set_visible(true);
                imp.status_label.set_text(
                    "Scan this QR code with Signal on your phone\n\
                     Settings → Linked Devices → Link New Device"
                );
            }
        ));
    }

    pub fn set_linking_in_progress(&self) {
        let imp = self.imp();
        imp.status_label.set_text("Linking device...");
        imp.spinner.set_visible(true);
        imp.spinner.set_spinning(true);
        imp.refresh_button.set_sensitive(false);
    }

    pub fn set_linked_successfully(&self) {
        let imp = self.imp();
        imp.status_label.set_text("Device linked successfully!");
        imp.spinner.set_spinning(false);
        imp.spinner.set_visible(false);
    }

    pub fn set_linking_failed(&self, error: &str) {
        let imp = self.imp();
        imp.status_label.set_text(&format!("Linking failed: {}", error));
        imp.spinner.set_spinning(false);
        imp.spinner.set_visible(false);
        imp.refresh_button.set_sensitive(true);
    }
}

impl Default for LinkDeviceView {
    fn default() -> Self {
        Self::new()
    }
}
