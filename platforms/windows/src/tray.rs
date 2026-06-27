//! Tray icon + menu, built on the standalone `tray-icon` crate (no WebView2).
//! Left-click toggles Tiếng Việt (VI/EN) like Unikey; the icon reflects the state
//! (color = VI, monochrome white = EN). Right-click opens the menu: pick Telex/VNI,
//! settings, guide, quit.
//!
//! This lives on the keyboard-hook thread (the one running a Win32 message loop):
//! `install()` creates the tray there, and `drain_events()` — called after each
//! message dispatch — reacts to clicks/menu picks. Settings and Onboarding are
//! launched as short-lived child processes so the tray process stays lightweight.

use std::cell::RefCell;

use funput_core::InputMethod;
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::{hook, shell, windows_ui};

const TRAY_PNG: &[u8] = include_bytes!("../icons/tray.png"); // VI: original color icon
const TRAY_MONO_PNG: &[u8] = include_bytes!("../icons/tray-mono.png"); // EN: monochrome white

struct TrayState {
    tray: TrayIcon,
    vni: CheckMenuItem,
    telex: CheckMenuItem,
}

thread_local! {
    static TRAY: RefCell<Option<TrayState>> = const { RefCell::new(None) };
}

/// Build the tray icon + menu on the current thread. Must run on a thread with a
/// Win32 message loop (the hook thread) so menu/click messages are delivered.
pub fn install() {
    let on = shell::enabled();
    let method = shell::method();

    let vni = CheckMenuItem::with_id("vni", "VNI", true, method == InputMethod::Vni, None);
    let telex = CheckMenuItem::with_id("telex", "Telex", true, method == InputMethod::Telex, None);
    let settings = MenuItem::with_id("settings", "Cài đặt…", true, None);
    let guide = MenuItem::with_id("guide", "Hướng dẫn", true, None);
    let update = MenuItem::with_id("check-update", "Kiểm tra cập nhật…", true, None);
    let quit = MenuItem::with_id("quit", "Thoát", true, None);

    let menu = Menu::new();
    menu.append_items(&[
        &vni,
        &telex,
        &PredefinedMenuItem::separator(),
        &settings,
        &guide,
        &update,
        &PredefinedMenuItem::separator(),
        &quit,
    ])
    .expect("build tray menu");

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        // Left-click toggles VI/EN; the menu opens on right-click instead.
        .with_menu_on_left_click(false)
        .with_tooltip(tooltip(on))
        .with_icon(make_icon(on).expect("tray icon"))
        .build()
        .expect("build tray icon");

    TRAY.with(|c| *c.borrow_mut() = Some(TrayState { tray, vni, telex }));

    // Keep the tray icon/tooltip in sync when VI/EN flips from the keyboard hotkey
    // or per-app auto-switch — both fire on this (hook) thread.
    hook::set_on_toggle(|on| refresh(on));
}

/// Drain pending tray + menu events. Call after each `DispatchMessageW` so the
/// events the tray window proc just queued are handled promptly.
pub fn drain_events() {
    while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = ev
        {
            let on = shell::toggle_enabled();
            refresh(on);
        }
    }

    while let Ok(ev) = MenuEvent::receiver().try_recv() {
        match ev.id.0.as_str() {
            "vni" => {
                shell::set_method(InputMethod::Vni);
                set_checks(true, false);
            }
            "telex" => {
                shell::set_method(InputMethod::Telex);
                set_checks(false, true);
            }
            "settings" => {
                windows_ui::launch_settings(false);
            }
            "guide" => {
                windows_ui::launch_onboarding();
            }
            "check-update" => {
                windows_ui::launch_settings(true);
            }
            "quit" => {
                windows_ui::terminate_children();
                hook::quit();
            }
            _ => {}
        }
    }
}

/// Refresh every tray field derived from persisted settings after a Settings child
/// changed the config file and the background engine reloaded it.
pub fn sync_from_shell() {
    let method = shell::method();
    set_checks(method == InputMethod::Vni, method == InputMethod::Telex);
    refresh(shell::enabled());
}

fn refresh(on: bool) {
    TRAY.with(|c| {
        if let Some(s) = c.borrow().as_ref() {
            if let Some(icon) = make_icon(on) {
                let _ = s.tray.set_icon(Some(icon));
            }
            let _ = s.tray.set_tooltip(Some(tooltip(on)));
        }
    });
}

fn set_checks(vni: bool, telex: bool) {
    TRAY.with(|c| {
        if let Some(s) = c.borrow().as_ref() {
            s.vni.set_checked(vni);
            s.telex.set_checked(telex);
        }
    });
}

fn make_icon(on: bool) -> Option<Icon> {
    let bytes = if on { TRAY_PNG } else { TRAY_MONO_PNG };
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Icon::from_rgba(img.into_raw(), w, h).ok()
}

fn tooltip(enabled: bool) -> String {
    if enabled {
        "Funput — Tiếng Việt (VI)".to_string()
    } else {
        "Funput — Tắt (EN)".to_string()
    }
}
