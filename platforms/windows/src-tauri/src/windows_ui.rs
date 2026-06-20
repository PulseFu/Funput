//! Open the Settings / Onboarding windows on demand (tray-only app otherwise).
//! Windows are created lazily and destroyed on close, so WebView2 only consumes
//! memory while a window is actually open. Acrylic gives the frosted-glass look.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

pub fn open_settings(app: &AppHandle) {
    open(app, "settings", "Funput — Cài đặt", 720.0, 480.0);
}

pub fn open_onboarding(app: &AppHandle) {
    open(app, "onboarding", "Chào mừng đến Funput", 460.0, 540.0);
}

fn open(app: &AppHandle, label: &str, title: &str, w: f64, h: f64) {
    // Focus an already-open window instead of spawning a second one.
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.set_focus();
        return;
    }

    let url = WebviewUrl::App(format!("index.html?view={label}").into());
    let built = WebviewWindowBuilder::new(app, label, url)
        .title(title)
        .inner_size(w, h)
        .resizable(false)
        .transparent(true)
        .center()
        .build();

    #[cfg(windows)]
    if let Ok(win) = built {
        // Subtle dark-tinted Acrylic; the transparent webview body shows it through.
        let _ = window_vibrancy::apply_acrylic(&win, Some((18, 18, 18, 125)));
    }
}
