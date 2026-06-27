// No console window for the release tray app.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// The shell exposes a broad state API; not every accessor is wired to UI yet.
#![allow(dead_code)]

#[cfg(not(windows))]
compile_error!("funput-windows builds only on Windows (global keyboard hook + SendInput).");

// Slint-generated components (SettingsWindow, OnboardingWindow, AppEntry).
slint::include_modules!();

mod commands;
mod compose;
mod dark_mode;
mod hook;
mod inject;
mod keymap;
mod settings;
mod shell;
mod tray;
mod update;
mod windows_ui;

fn main() {
    let mode = std::env::args().nth(1);

    // Settings and Onboarding run in short-lived child processes. Keeping Slint
    // out of the background path lets Windows reclaim the complete UI runtime,
    // renderer, font caches, and graphics driver allocations when the window closes.
    match mode.as_deref() {
        Some("--settings") => {
            dark_mode::allow_dark_menus();
            windows_ui::run_settings(false);
            return;
        }
        Some("--settings-check-update") => {
            dark_mode::allow_dark_menus();
            windows_ui::run_settings(true);
            return;
        }
        Some("--onboarding") => {
            dark_mode::allow_dark_menus();
            windows_ui::run_onboarding();
            return;
        }
        _ => {}
    }

    // Let Windows draw the tray's right-click menu dark when the system is dark.
    // Process-global, so set it before the tray (on the hook thread) is created.
    dark_mode::allow_dark_menus();

    // Touch the shell to load persisted settings + apply them to the engine.
    let settings = shell::snapshot();

    // Keep the OS autostart entry in sync with the saved preference.
    commands::sync_autostart(settings.launch_at_login);

    // First run: launch the short-lived Onboarding UI process.
    if !settings.has_completed_onboarding {
        windows_ui::launch_onboarding();
    }

    // The background process never initializes Slint. Its main thread is the Win32
    // message loop that owns the keyboard hook and tray icon.
    hook::run();
}
