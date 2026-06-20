//! Emit an [`InjectPlan`] to the focused app: Backspace presses, then Unicode
//! characters, via `SendInput`. Every synthesized event carries [`INJECT_TAG`] in
//! `dwExtraInfo` so the hook ignores them (no re-entrancy).

use funput_desktop::InjectPlan;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK,
};

use crate::shell::INJECT_TAG;

fn vk_event(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let dw_flags = if up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: dw_flags,
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    }
}

fn unicode_event(unit: u16, up: bool) -> INPUT {
    let mut dw_flags = KEYEVENTF_UNICODE;
    if up {
        dw_flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: dw_flags,
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    }
}

/// Send the deletions then the new text as one atomic `SendInput` batch.
pub fn send_plan(plan: &InjectPlan) {
    if plan.is_noop() {
        return;
    }
    let mut inputs: Vec<INPUT> = Vec::with_capacity(plan.backspaces * 2 + plan.units.len() * 2);
    for _ in 0..plan.backspaces {
        inputs.push(vk_event(VK_BACK, false));
        inputs.push(vk_event(VK_BACK, true));
    }
    for &unit in &plan.units {
        inputs.push(unicode_event(unit, false));
        inputs.push(unicode_event(unit, true));
    }
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}
