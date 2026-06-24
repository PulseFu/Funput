//! Tray icon + menu. Left-click toggles Tiếng Việt (VI/EN) like Unikey; the icon
//! reflects the state (color = VI, monochrome white = EN). Right-click opens the
//! menu: pick Telex/VNI, settings, guide, quit. Tauri 2 menu/tray API — names may
//! need to track the exact tauri version when building.

use funput_core::InputMethod;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{image::Image, AppHandle};

use crate::{hook, shell, windows_ui};

const TRAY_PNG: &[u8] = include_bytes!("../icons/tray.png"); // VI: original color icon
const TRAY_MONO_PNG: &[u8] = include_bytes!("../icons/tray-mono.png"); // EN: monochrome white

/// Update the tray icon + tooltip to reflect the VI/EN state. The icon is the only
/// state indicator now (no menu checkmark), so every place that flips enabled goes
/// through here: left-click, the Ctrl+` hotkey, and per-app auto-switch.
fn refresh_tray(app: &AppHandle, on: bool) {
    if let Some(tray) = app.tray_by_id("funput") {
        let bytes = if on { TRAY_PNG } else { TRAY_MONO_PNG };
        if let Ok(img) = Image::from_bytes(bytes) {
            let _ = tray.set_icon(Some(img));
        }
        let _ = tray.set_tooltip(Some(tooltip(on)));
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let vni = CheckMenuItemBuilder::with_id("vni", "VNI")
        .checked(shell::method() == InputMethod::Vni)
        .build(app)?;
    let telex = CheckMenuItemBuilder::with_id("telex", "Telex")
        .checked(shell::method() == InputMethod::Telex)
        .build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Cài đặt…").build(app)?;
    let guide = MenuItemBuilder::with_id("guide", "Hướng dẫn").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Thoát").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&vni)
        .item(&telex)
        .separator()
        .item(&settings)
        .item(&guide)
        .separator()
        .item(&quit)
        .build()?;

    // Initial icon/tooltip reflect the current state (may start in EN).
    let on = shell::enabled();
    let icon = Image::from_bytes(if on { TRAY_PNG } else { TRAY_MONO_PNG })?;

    // Clones moved into the event handlers (menu items are Arc-backed / cheap).
    let (vni_e, telex_e) = (vni.clone(), telex.clone());
    let app_click = app.clone();

    let tray = TrayIconBuilder::with_id("funput")
        .icon(icon)
        .tooltip(tooltip(on))
        .menu(&menu)
        // Left-click toggles VI/EN instead of opening the menu; right-click opens it.
        .show_menu_on_left_click(false)
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let on = shell::toggle_enabled();
                refresh_tray(&app_click, on);
            }
        })
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "vni" => {
                shell::set_method(InputMethod::Vni);
                let _ = vni_e.set_checked(true);
                let _ = telex_e.set_checked(false);
            }
            "telex" => {
                shell::set_method(InputMethod::Telex);
                let _ = telex_e.set_checked(true);
                let _ = vni_e.set_checked(false);
            }
            "settings" => windows_ui::open_settings(app),
            "guide" => windows_ui::open_onboarding(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    // Keep the tray icon/tooltip in sync when the toggle comes from the keyboard
    // hotkey (Ctrl+`) or per-app auto-switch, both of which fire on the hook thread.
    let handle = app.clone();
    hook::set_on_toggle(move |on| {
        let inner = handle.clone();
        let _ = handle.run_on_main_thread(move || refresh_tray(&inner, on));
    });

    let _ = tray;
    Ok(())
}

fn tooltip(enabled: bool) -> String {
    if enabled {
        "Funput — Tiếng Việt (VI)".to_string()
    } else {
        "Funput — Tắt (EN)".to_string()
    }
}
