//! UI child-process launch, shutdown, and entry points.

use std::cell::RefCell;
use std::process::{Child, Command};

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
};

use super::{onboarding, settings_window};
use crate::shell;

thread_local! {
    static UI_PROCESS: RefCell<Option<Child>> = const { RefCell::new(None) };
}

const RECENT_APPS_ENV: &str = "FUNPUT_RECENT_APPS";
const PARENT_PID_ENV: &str = "FUNPUT_PARENT_PID";

/// Launch Settings without loading Slint into the background tray process.
pub(crate) fn launch_settings(check_update: bool) {
    launch_child(if check_update {
        "--settings-check-update"
    } else {
        "--settings"
    });
}

/// Launch the first-run guide in its own short-lived UI process.
pub(crate) fn launch_onboarding() {
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

/// Close the UI child before the user exits the tray process.
pub(crate) fn terminate_children() {
    UI_PROCESS.with(|cell| {
        if let Some(mut child) = cell.borrow_mut().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    });
}

/// Stop the old background tray before the updater launches the new executable.
pub(crate) fn terminate_parent_for_update() {
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

/// Run the only Settings window until it closes, then end the child process.
pub(crate) fn run_settings(check_update: bool) {
    seed_recent_apps();
    if check_update {
        settings_window::open_and_check_updates();
    } else {
        settings_window::open();
    }
    let _ = slint::run_event_loop();
}

/// Run the only Onboarding window until it closes.
pub(crate) fn run_onboarding() {
    onboarding::open();
    let _ = slint::run_event_loop();
}
