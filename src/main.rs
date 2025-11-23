//! Signal You Messenger - A GTK4/libadwaita Signal client for Linux
//!
//! This application provides a native GTK4 interface for Signal Messenger,
//! featuring Material You theming and deep GNOME integration.

mod application;
mod config;
mod services;
mod signal;
mod ui;
mod window;

use gtk4::prelude::*;
use gtk4::{gio, glib};

use application::SignalYouApplication;

fn main() -> glib::ExitCode {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize gettext for i18n
    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR)
        .expect("Failed to bind text domain");
    gettextrs::textdomain(config::GETTEXT_PACKAGE).expect("Failed to set text domain");

    // Register resources
    gio::resources_register_include!("signal-you-messenger.gresource")
        .expect("Failed to register resources");

    // Create and run the application
    let app = SignalYouApplication::new();
    app.run()
}
