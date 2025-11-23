//! Application configuration constants
//!
//! These constants are used for i18n, resource paths, and application metadata.

/// The gettext package name for translations
pub const GETTEXT_PACKAGE: &str = "signal-you-messenger";

/// Directory containing locale files
pub const LOCALEDIR: &str = "/usr/share/locale";

/// Application ID
pub const APP_ID: &str = "com.signalyou.Messenger";

/// Application name
pub const APP_NAME: &str = "Signal You Messenger";

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application website
pub const WEBSITE: &str = "https://github.com/phlthy88/Signal-You-Messenger";
