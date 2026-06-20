//! Global engine + settings state shared between the keyboard-hook thread, the
//! tray, and the Tauri commands. The hook callback is a bare `extern "system"`
//! function with no user pointer, so this lives in a process-global behind a mutex.
//! No Windows APIs here.

use std::sync::{Mutex, OnceLock};

use funput_core::InputMethod;
use funput_engine::{Engine, ImeResult};

use crate::settings::{Hotkey, Method, Settings};

/// Tag stamped into `dwExtraInfo` of every event we synthesize via `SendInput`, so
/// the hook can recognize and ignore its own injected keystrokes (no re-entrancy).
pub const INJECT_TAG: usize = 0x4655_4E50; // "FUNP"

struct Shell {
    engine: Engine,
    settings: Settings,
}

static SHELL: OnceLock<Mutex<Shell>> = OnceLock::new();

fn apply_to_engine(engine: &mut Engine, s: &Settings) {
    engine.set_method(s.method.core());
    engine.set_enabled(s.enabled);
    engine.set_smart_restore(s.smart_restore);
    engine.set_eager_restore(s.eager_restore);
    engine.clear();
}

fn shell() -> &'static Mutex<Shell> {
    SHELL.get_or_init(|| {
        let settings = Settings::load();
        let mut engine = Engine::new();
        apply_to_engine(&mut engine, &settings);
        Mutex::new(Shell { engine, settings })
    })
}

fn with<R>(f: impl FnOnce(&mut Shell) -> R) -> R {
    let mut guard = shell().lock().expect("shell mutex poisoned");
    f(&mut guard)
}

// --- reads -----------------------------------------------------------------

pub fn snapshot() -> Settings {
    with(|s| s.settings)
}
pub fn enabled() -> bool {
    with(|s| s.settings.enabled)
}
pub fn method() -> InputMethod {
    with(|s| s.settings.method.core())
}
pub fn toggle_hotkey() -> Hotkey {
    with(|s| s.settings.toggle_hotkey)
}
pub fn is_composing() -> bool {
    with(|s| !s.engine.buffer().is_empty())
}

// --- writes (each persists) ------------------------------------------------

/// Flip VI/EN; returns the new state.
pub fn toggle_enabled() -> bool {
    with(|s| {
        let on = !s.settings.enabled;
        s.settings.enabled = on;
        s.engine.set_enabled(on);
        if !on {
            s.engine.clear();
        }
        s.settings.save();
        on
    })
}

pub fn set_enabled(on: bool) {
    with(|s| {
        s.settings.enabled = on;
        s.engine.set_enabled(on);
        if !on {
            s.engine.clear();
        }
        s.settings.save();
    });
}

pub fn set_method(method: InputMethod) {
    with(|s| {
        s.settings.method = Method::from_core(method);
        s.engine.set_method(method);
        s.engine.clear();
        s.settings.save();
    });
}

pub fn set_smart_restore(on: bool) {
    with(|s| {
        s.settings.smart_restore = on;
        s.engine.set_smart_restore(on);
        s.settings.save();
    });
}

pub fn set_eager_restore(on: bool) {
    with(|s| {
        s.settings.eager_restore = on;
        s.engine.set_eager_restore(on);
        s.settings.save();
    });
}

pub fn set_toggle_hotkey(hotkey: Hotkey) {
    with(|s| {
        s.settings.toggle_hotkey = hotkey;
        s.settings.save();
    });
}

/// Persist the launch-at-login preference. The registry side effect is applied by
/// the command (it needs the Tauri app handle / autostart plugin).
pub fn set_launch_at_login(on: bool) {
    with(|s| {
        s.settings.launch_at_login = on;
        s.settings.save();
    });
}

pub fn complete_onboarding() {
    with(|s| {
        s.settings.has_completed_onboarding = true;
        s.settings.save();
    });
}

// --- composition driving (called from the hook) ----------------------------

pub fn process_char(c: char) -> ImeResult {
    with(|s| s.engine.process_char(c))
}

/// Sync the engine after Backspace while composing; the physical Backspace then
/// passes through so the app deletes its own visible char (like `funput-term`).
pub fn on_backspace() {
    with(|s| {
        s.engine.on_backspace();
    });
}

pub fn clear() {
    with(|s| s.engine.clear());
}
