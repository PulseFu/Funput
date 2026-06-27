//! Settings / Onboarding process launcher plus the UI implementation used inside
//! those short-lived child processes. The background tray process never initializes
//! Slint, so closing the child lets Windows reclaim the entire UI runtime.
//!
//! Launch functions run on the background hook/tray thread. Window functions run
//! only in the child process's main Slint event-loop thread.

use std::cell::RefCell;
use std::process::{Child, Command};

use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

use crate::compose::FieldComposer;
use crate::settings::{Hotkey, Method, ToneStyle};
use crate::{commands, shell, AppEntry, Compose, OnboardingWindow, SettingsWindow, ShortcutEntry};

thread_local! {
    static SETTINGS: RefCell<Option<Weak<SettingsWindow>>> = const { RefCell::new(None) };
    static ONBOARDING: RefCell<Option<Weak<OnboardingWindow>>> = const { RefCell::new(None) };
    static UI_PROCESS: RefCell<Option<Child>> = const { RefCell::new(None) };
    /// Vietnamese composer for the gõ tắt expansion field (UI thread only).
    static COMPOSER: RefCell<FieldComposer> = RefCell::new(FieldComposer::new());
}

const RECENT_APPS_ENV: &str = "FUNPUT_RECENT_APPS";
const PARENT_PID_ENV: &str = "FUNPUT_PARENT_PID";

// --- child-process lifecycle -----------------------------------------------

/// Launch Settings without loading Slint into the background tray process.
pub fn launch_settings(check_update: bool) {
    let arg = if check_update {
        "--settings-check-update"
    } else {
        "--settings"
    };
    launch_child(arg);
}

/// Launch the first-run guide in its own short-lived UI process.
pub fn launch_onboarding() {
    launch_child("--onboarding");
}

fn launch_child(arg: &str) {
    let already_running = UI_PROCESS.with(|cell| {
        let mut child = cell.borrow_mut();
        let running = child
            .as_mut()
            .is_some_and(|process| matches!(process.try_wait(), Ok(None)));
        if !running {
            *child = None;
        }
        running
    });
    if already_running {
        return;
    }

    let Some(exe) = std::env::current_exe().ok() else {
        return;
    };
    let recent = serde_json::to_string(&shell::recent_apps()).unwrap_or_else(|_| "[]".into());
    let child = Command::new(exe)
        .arg(arg)
        .env(RECENT_APPS_ENV, recent)
        .env(PARENT_PID_ENV, std::process::id().to_string())
        .spawn()
        .ok();
    UI_PROCESS.with(|cell| *cell.borrow_mut() = child);
}

/// Close any UI children before the user exits the tray process.
pub fn terminate_children() {
    UI_PROCESS.with(|cell| {
        if let Some(mut child) = cell.borrow_mut().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    });
}

/// The updater runs in the Settings child. Stop the old background tray before
/// spawning the newly installed executable, otherwise two tray processes would
/// remain active after an update.
pub fn terminate_parent_for_update() {
    let Some(pid) = std::env::var(PARENT_PID_ENV)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
    else {
        return;
    };
    unsafe {
        let access = PROCESS_TERMINATE | PROCESS_SYNCHRONIZE;
        if let Ok(process) = OpenProcess(access, false, pid) {
            let _ = TerminateProcess(process, 0);
            let _ = WaitForSingleObject(process, 5_000);
            let _ = CloseHandle(process);
        }
    }
}

fn seed_recent_apps() {
    let apps = std::env::var(RECENT_APPS_ENV)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();
    shell::seed_recent_apps(apps);
}

/// Entry point for a Settings child process. `run_event_loop` returns when its
/// only window closes, then normal process teardown releases all Slint resources.
pub fn run_settings(check_update: bool) {
    seed_recent_apps();
    if check_update {
        open_settings_and_check_updates();
    } else {
        open_settings();
    }
    let _ = slint::run_event_loop();
}

/// Entry point for an Onboarding child process.
pub fn run_onboarding() {
    open_onboarding();
    let _ = slint::run_event_loop();
}

// --- Settings --------------------------------------------------------------

pub fn open_settings() {
    if let Some(win) = SETTINGS.with(|c| c.borrow().as_ref().and_then(|w| w.upgrade())) {
        populate_settings(&win);
        let _ = win.show();
        return;
    }
    let win = SettingsWindow::new().expect("create settings window");
    populate_settings(&win);
    wire_settings(&win);
    let _ = win.show();
    SETTINGS.with(|c| *c.borrow_mut() = Some(win.as_weak()));
}

fn populate_settings(win: &SettingsWindow) {
    let s = shell::snapshot();
    win.set_method(s.method.id().into());
    win.set_tone_style(s.tone_style.id().into());
    win.set_hotkey(s.toggle_hotkey.id().into());
    win.set_hotkey_caps(caps_model(s.toggle_hotkey));
    win.set_smart_restore(s.smart_restore);
    win.set_eager_restore(s.eager_restore);
    win.set_launch_at_login(s.launch_at_login);
    win.set_version(env!("CARGO_PKG_VERSION").into());
    // Reset the updater UI each time the window is (re)shown.
    win.set_update_state("idle".into());
    win.set_update_version("".into());
    win.set_update_message("".into());
    refresh_apps(win);
    win.set_shortcuts(shortcuts_model(&shell::shortcuts()));
}

fn wire_settings(win: &SettingsWindow) {
    let w = win.as_weak();
    win.on_pick_method(move |v| {
        if let Some(m) = Method::from_id(&v) {
            commands::set_method(m);
        }
        if let Some(win) = w.upgrade() {
            win.set_method(v);
        }
    });

    let w = win.as_weak();
    win.on_pick_tone(move |v| {
        if let Some(t) = ToneStyle::from_id(&v) {
            commands::set_tone_style(t);
        }
        if let Some(win) = w.upgrade() {
            win.set_tone_style(v);
        }
    });

    let w = win.as_weak();
    win.on_pick_hotkey(move |v| {
        if let Some(h) = Hotkey::from_id(&v) {
            commands::set_toggle_hotkey(h);
            if let Some(win) = w.upgrade() {
                win.set_hotkey(v);
                win.set_hotkey_caps(caps_model(h));
            }
        }
    });

    // The Switch values are two-way bound, so the property is already updated; we
    // only need to persist (and apply the OS side effect for launch-at-login).
    win.on_set_smart(commands::set_smart_restore);
    win.on_set_eager(commands::set_eager_restore);
    win.on_set_launch(commands::set_launch_at_login);

    let w = win.as_weak();
    win.on_add_app(move |id| {
        if let Some(app) = shell::recent_apps()
            .into_iter()
            .find(|a| a.id == id.as_str())
        {
            commands::add_excluded_app(app);
        }
        if let Some(win) = w.upgrade() {
            refresh_apps(&win);
        }
    });

    let w = win.as_weak();
    win.on_remove_app(move |id| {
        commands::remove_excluded_app(&id);
        if let Some(win) = w.upgrade() {
            refresh_apps(&win);
        }
    });

    let w = win.as_weak();
    win.on_add_shortcut(move || {
        commands::add_shortcut();
        if let Some(win) = w.upgrade() {
            win.set_shortcuts(shortcuts_model(&shell::shortcuts()));
        }
    });

    let w = win.as_weak();
    win.on_remove_shortcut(move |index| {
        commands::remove_shortcut(index.max(0) as usize);
        if let Some(win) = w.upgrade() {
            win.set_shortcuts(shortcuts_model(&shell::shortcuts()));
        }
    });

    // Editing a field only persists (engine + settings); the model is NOT rebuilt,
    // so the LineEdit keeps its text and caret while the user types.
    win.on_edit_trigger(|index, text| {
        commands::set_shortcut_trigger(index.max(0) as usize, text.to_string());
    });
    win.on_edit_expansion(|index, text| {
        commands::set_shortcut_expansion(index.max(0) as usize, text.to_string());
    });

    // In-process Vietnamese composition for the expansion field (the global hook
    // can't reach our own window). The Slint TextInput forwards each keystroke here.
    let compose = win.global::<Compose>();
    compose.on_reset(|text| {
        let (method, tone) = shell::method_and_tone();
        COMPOSER.with(|c| c.borrow_mut().reset(text.as_str(), method, tone));
    });
    compose.on_key(|ch| {
        let c = ch.chars().next().unwrap_or('\0');
        COMPOSER.with(|comp| comp.borrow_mut().key(c)).into()
    });
    compose.on_backspace(|| COMPOSER.with(|comp| comp.borrow_mut().backspace()).into());

    win.on_open_link(|url| commands::open_url(url.as_str()));

    // Auto-update: the buttons are argument-free; state flows back through
    // `set_update_state`. `commands` runs the work off the main thread.
    win.on_check_update(commands::check_for_updates);
    win.on_install_update(commands::install_update);
    win.on_relaunch_now(commands::relaunch_after_update);
}

/// Reflect an update step on the Settings window (no-op if it is not open).
/// Called on the main thread via `slint::invoke_from_event_loop`.
pub fn set_update_state(state: &str, version: &str, message: &str) {
    SETTINGS.with(|c| {
        if let Some(win) = c.borrow().as_ref().and_then(|w| w.upgrade()) {
            win.set_update_state(state.into());
            win.set_update_version(version.into());
            win.set_update_message(message.into());
        }
    });
}

/// Open Settings on the "Giới thiệu" tab and immediately check for updates — the
/// tray's "Kiểm tra cập nhật…" entry point.
pub fn open_settings_and_check_updates() {
    open_settings();
    SETTINGS.with(|c| {
        if let Some(win) = c.borrow().as_ref().and_then(|w| w.upgrade()) {
            win.set_active("about".into());
        }
    });
    commands::check_for_updates();
}

/// Refresh the excluded list and the "recent" picker (recent minus already-excluded).
fn refresh_apps(win: &SettingsWindow) {
    let excluded = shell::excluded_apps();
    let addable: Vec<_> = shell::recent_apps()
        .into_iter()
        .filter(|r| !excluded.iter().any(|e| e.id == r.id))
        .collect();
    win.set_excluded_apps(apps_model(&excluded));
    win.set_recent_apps(apps_model(&addable));
}

// --- Onboarding ------------------------------------------------------------

pub fn open_onboarding() {
    if let Some(win) = ONBOARDING.with(|c| c.borrow().as_ref().and_then(|w| w.upgrade())) {
        win.set_step(0);
        let _ = win.show();
        return;
    }
    let win = OnboardingWindow::new().expect("create onboarding window");
    let s = shell::snapshot();
    win.set_method(s.method.id().into());
    win.set_launch_at_login(s.launch_at_login);

    let w = win.as_weak();
    win.on_pick_method(move |v| {
        if let Some(m) = Method::from_id(&v) {
            commands::set_method(m);
        }
        if let Some(win) = w.upgrade() {
            win.set_method(v);
        }
    });
    win.on_set_launch(commands::set_launch_at_login);

    let w = win.as_weak();
    win.on_finish(move || {
        commands::complete_onboarding();
        if let Some(win) = w.upgrade() {
            let _ = win.hide();
        }
    });

    let _ = win.show();
    ONBOARDING.with(|c| *c.borrow_mut() = Some(win.as_weak()));
}

// --- helpers ---------------------------------------------------------------

fn caps_model(hotkey: Hotkey) -> ModelRc<SharedString> {
    let caps: Vec<SharedString> = hotkey.caps().iter().map(|c| (*c).into()).collect();
    ModelRc::new(VecModel::from(caps))
}

fn apps_model(apps: &[crate::settings::ExcludedApp]) -> ModelRc<AppEntry> {
    let rows: Vec<AppEntry> = apps
        .iter()
        .map(|a| AppEntry {
            id: a.id.clone().into(),
            name: a.name.clone().into(),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}

fn shortcuts_model(shortcuts: &[crate::settings::Shortcut]) -> ModelRc<ShortcutEntry> {
    let rows: Vec<ShortcutEntry> = shortcuts
        .iter()
        .map(|s| ShortcutEntry {
            trigger: s.trigger.clone().into(),
            expansion: s.expansion.clone().into(),
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}
