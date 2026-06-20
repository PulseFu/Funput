//! The global low-level keyboard hook: intercepts keys, drives the engine, and
//! injects composed text. Runs on its own thread with a message loop (required for
//! `WH_KEYBOARD_LL`). The hook callback is a bare C function, so it reaches the
//! engine through [`crate::shell`]'s process-global state.

use std::sync::OnceLock;

use funput_desktop::{classify, plan_inject, KeyKind};
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HC_ACTION,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use crate::{inject, keymap, shell};

/// Called after a Ctrl+` toggle so the tray can refresh its checkmark/icon.
type ToggleCb = Box<dyn Fn(bool) + Send + Sync>;
static ON_TOGGLE: OnceLock<ToggleCb> = OnceLock::new();

pub fn set_on_toggle(f: impl Fn(bool) + Send + Sync + 'static) {
    let _ = ON_TOGGLE.set(Box::new(f));
}

/// Install the hook on a dedicated thread with its own message pump.
pub fn spawn() {
    std::thread::spawn(|| unsafe {
        let hmod = GetModuleHandleW(None).unwrap_or_default();
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE(hmod.0), 0);
        if hook.is_err() {
            eprintln!("Funput: failed to install keyboard hook: {hook:?}");
            return;
        }
        // LL keyboard hooks are delivered through this thread's message queue.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        // Skip the events we ourselves synthesized via SendInput (no re-entrancy).
        if kbd.dwExtraInfo == shell::INJECT_TAG {
            return CallNextHookEx(None, code, wparam, lparam);
        }
        let msg = wparam.0 as u32;
        if (msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN) && handle_keydown(kbd) {
            return LRESULT(1); // swallow: do not pass the key to the focused app
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

/// Returns true if the key should be swallowed (we injected a replacement), false
/// to let it reach the app.
fn handle_keydown(kbd: &KBDLLHOOKSTRUCT) -> bool {
    let vk = VIRTUAL_KEY(kbd.vkCode as u16);
    let mods = keymap::read_mods();

    if keymap::is_toggle(vk, mods, shell::toggle_hotkey()) {
        let on = shell::toggle_enabled();
        if let Some(cb) = ON_TOGGLE.get() {
            cb(on);
        }
        return true;
    }

    if !shell::enabled() {
        return false; // English mode: hands off
    }

    match classify(&keymap::to_key_event(kbd)) {
        KeyKind::Compose(c) => {
            let plan = plan_inject(&shell::process_char(c));
            if plan.is_noop() {
                false // Action::None — the literal key reaches the app
            } else {
                inject::send_plan(&plan); // delete + retype the composed text
                true
            }
        }
        KeyKind::Backspace => {
            if shell::is_composing() {
                shell::on_backspace(); // sync engine; app deletes its own char
            }
            false
        }
        KeyKind::Flush => {
            shell::clear(); // commit what is shown; nav/Enter/Tab/shortcut passes
            false
        }
        KeyKind::PassThrough => false,
    }
}
