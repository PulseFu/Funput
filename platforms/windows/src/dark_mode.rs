//! Make the tray's right-click menu follow the Windows light/dark setting.
//!
//! Win32 popup menus — drawn by `TrackPopupMenu`, which is what `tray-icon`/`muda`
//! use for the tray menu — ignore the system dark theme unless the process opts in
//! via two *undocumented* `uxtheme.dll` exports: `SetPreferredAppMode` (ordinal
//! 135) and `FlushMenuThemes` (ordinal 136), the same trick win32-darkmode uses.
//! muda themes the menu *bar* but explicitly not popups, so we do this ourselves.

use windows::core::{s, PCSTR};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

/// Opt the process into dark menus when Windows is in dark mode (and light when
/// light). Call once at startup, before the tray menu is first shown. No-op on
/// Windows builds that lack these exports.
pub fn allow_dark_menus() {
    // PreferredAppMode::AllowDark — menus track the system light/dark setting.
    const ALLOW_DARK: i32 = 1;

    unsafe {
        let Ok(uxtheme) = LoadLibraryA(s!("uxtheme.dll")) else {
            return;
        };
        // The two uxtheme functions are exported by ordinal only (no public names),
        // so look them up via MAKEINTRESOURCE-style ordinal pointers.
        if let Some(proc) = GetProcAddress(uxtheme, PCSTR(135 as *const u8)) {
            let set_preferred_app_mode: extern "system" fn(i32) -> i32 =
                std::mem::transmute(proc);
            set_preferred_app_mode(ALLOW_DARK);
        }
        if let Some(proc) = GetProcAddress(uxtheme, PCSTR(136 as *const u8)) {
            let flush_menu_themes: extern "system" fn() = std::mem::transmute(proc);
            flush_menu_themes();
        }
    }
}
