//! Tray icon + menu: toggle Tiếng Việt, pick Telex/VNI, quit. The menu is the
//! only UI in the MVP (no window). Tauri 2 menu/tray API — names may need to track
//! the exact tauri version when building.

use funput_core::InputMethod;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{image::Image, AppHandle, Manager};

use crate::{hook, shell, windows_ui};

const TRAY_PNG: &[u8] = include_bytes!("../icons/tray.png");

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let vi = CheckMenuItemBuilder::with_id("toggle", "Tiếng Việt")
        .checked(shell::enabled())
        .build(app)?;
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
        .item(&vi)
        .separator()
        .item(&vni)
        .item(&telex)
        .separator()
        .item(&settings)
        .item(&guide)
        .separator()
        .item(&quit)
        .build()?;

    let icon = Image::from_bytes(TRAY_PNG)?;

    // Clones moved into the event handlers (menu items are Arc-backed / cheap).
    let (vi_e, vni_e, telex_e) = (vi.clone(), vni.clone(), telex.clone());

    let tray = TrayIconBuilder::with_id("funput")
        .icon(icon)
        .tooltip(tooltip(shell::enabled()))
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => {
                let on = !shell::enabled();
                shell::set_enabled(on);
                let _ = vi_e.set_checked(on);
                if let Some(tray) = app.tray_by_id("funput") {
                    let _ = tray.set_tooltip(Some(tooltip(on)));
                }
            }
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

    // Keep the tray's checkmark/tooltip in sync when the toggle comes from the
    // keyboard hotkey (Ctrl+`), which fires on the hook thread.
    let handle = app.clone();
    let vi_h = vi.clone();
    hook::set_on_toggle(move |on| {
        let handle = handle.clone();
        let vi_h = vi_h.clone();
        let _ = handle.run_on_main_thread(move || {
            let _ = vi_h.set_checked(on);
            if let Some(tray) = handle.tray_by_id("funput") {
                let _ = tray.set_tooltip(Some(tooltip(on)));
            }
        });
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
