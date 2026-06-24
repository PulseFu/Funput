//! Funput Linux — Settings & Onboarding window (GTK4 + libadwaita).
//!
//! Typing is handled by the Fcitx5 addon (`platforms/linux/fcitx5`) or the IBus
//! engine (`platforms/linux/ibus`); this binary only edits the shared settings
//! file (`~/.config/Funput/settings.json`). No tray: VI/EN toggling and the status
//! icon are provided by Fcitx5/IBus themselves. Replaces the retired Tauri shell.

#[cfg(not(target_os = "linux"))]
compile_error!("funput-settings builds only on Linux (the typing engine ships as a Fcitx5/IBus shell).");

mod framework;
mod onboarding;
mod settings;
mod settings_window;

use adw::prelude::*;
use adw::Application;
use gtk::glib;

use crate::settings::Settings;

// Matches the bundle identifier used across platforms.
const APP_ID: &str = "app.funput.funput";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(on_activate);
    // run() reads std::env::args; AdwApplication initializes libadwaita on startup.
    app.run()
}

/// Single-instance behaviour: GApplication routes a second launch here, so just
/// re-present the existing window instead of opening a duplicate. First run walks
/// onboarding; afterwards it opens Settings directly.
fn on_activate(app: &Application) {
    if let Some(win) = app.active_window() {
        win.present();
        return;
    }

    let settings = Settings::load();
    if settings.has_completed_onboarding {
        settings_window::build(app).present();
    } else {
        onboarding::build(app).present();
    }
}
