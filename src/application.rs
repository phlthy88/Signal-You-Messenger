//! Main application struct and lifecycle management

use gtk4::prelude::*;
use gtk4::{gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::config;
use crate::window::SignalYouWindow;

mod imp {
    use super::*;
    use adw::subclass::prelude::*;

    #[derive(Debug, Default)]
    pub struct SignalYouApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for SignalYouApplication {
        const NAME: &'static str = "SignalYouApplication";
        type Type = super::SignalYouApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for SignalYouApplication {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_actions();
            obj.setup_accels();
        }
    }

    impl ApplicationImpl for SignalYouApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = SignalYouWindow::new(application.upcast_ref());
                window.upcast()
            };

            window.present();
        }

        fn startup(&self) {
            self.parent_startup();

            // Initialize libadwaita
            adw::init().expect("Failed to initialize libadwaita");

            // Set up application styling
            let display = gtk4::gdk::Display::default().expect("Could not get default display");
            let provider = gtk4::CssProvider::new();
            provider.load_from_string(include_str!("style.css"));

            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    impl GtkApplicationImpl for SignalYouApplication {}
    impl AdwApplicationImpl for SignalYouApplication {}
}

glib::wrapper! {
    pub struct SignalYouApplication(ObjectSubclass<imp::SignalYouApplication>)
        @extends gio::Application, gtk4::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl SignalYouApplication {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", config::APP_ID)
            .property("flags", gio::ApplicationFlags::FLAGS_NONE)
            .property("resource-base-path", "/com/signalyou/Messenger")
            .build()
    }

    fn setup_actions(&self) {
        // Quit action
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                app.quit();
            })
            .build();

        // About action
        let action_about = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| {
                app.show_about_dialog();
            })
            .build();

        // Preferences action
        let action_preferences = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| {
                app.show_preferences();
            })
            .build();

        self.add_action_entries([action_quit, action_about, action_preferences]);
    }

    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("app.preferences", &["<Control>comma"]);
        self.set_accels_for_action("win.new-chat", &["<Control>n"]);
        self.set_accels_for_action("win.search", &["<Control>f"]);
    }

    fn show_about_dialog(&self) {
        let mut builder = adw::AboutWindow::builder()
            .application_name("Signal You Messenger")
            .application_icon(config::APP_ID)
            .version(config::VERSION)
            .developer_name("Signal You Team")
            .license_type(gtk4::License::Gpl30)
            .website("https://github.com/phlthy88/Signal-You-Messenger")
            .issue_url("https://github.com/phlthy88/Signal-You-Messenger/issues")
            .developers(vec!["Signal You Team".to_string()])
            .copyright("© 2024 Signal You Team")
            .comments("A GTK4/libadwaita Material You fork of Signal Messenger for Linux")
            .modal(true);

        if let Some(window) = self.active_window() {
            builder = builder.transient_for(&window);
        }

        let dialog = builder.build();
        dialog.present();
    }

    fn show_preferences(&self) {
        // TODO: Implement preferences dialog
        tracing::info!("Preferences dialog requested");
    }
}

impl Default for SignalYouApplication {
    fn default() -> Self {
        Self::new()
    }
}
