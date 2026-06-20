//! Tauri commands the web UI (`platforms/ui/src/lib/api.ts`) calls. Each mutates
//! the shared `shell` state (which applies to the engine + persists), and a couple
//! apply an OS side effect.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::settings::{Hotkey, Method, Settings};
use crate::shell;

#[tauri::command]
pub fn get_settings() -> Settings {
    shell::snapshot()
}

#[tauri::command]
pub fn set_method(method: Method) {
    shell::set_method(method.core());
}

#[tauri::command]
pub fn set_enabled(on: bool) {
    shell::set_enabled(on);
}

#[tauri::command]
pub fn set_smart_restore(on: bool) {
    shell::set_smart_restore(on);
}

#[tauri::command]
pub fn set_eager_restore(on: bool) {
    shell::set_eager_restore(on);
}

#[tauri::command]
pub fn set_toggle_hotkey(hotkey: Hotkey) {
    shell::set_toggle_hotkey(hotkey);
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, on: bool) {
    shell::set_launch_at_login(on);
    let mgr = app.autolaunch();
    let _ = if on { mgr.enable() } else { mgr.disable() };
}

#[tauri::command]
pub fn complete_onboarding() {
    shell::complete_onboarding();
}
