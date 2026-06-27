//! Onboarding window creation and callbacks.

use std::cell::RefCell;

use slint::{ComponentHandle, Weak};

use crate::settings::Method;
use crate::{commands, shell, OnboardingWindow};

thread_local! {
    static WINDOW: RefCell<Option<Weak<OnboardingWindow>>> = const { RefCell::new(None) };
}

pub(super) fn open() {
    if let Some(window) = WINDOW.with(|cell| cell.borrow().as_ref().and_then(Weak::upgrade)) {
        window.set_step(0);
        let _ = window.show();
        return;
    }

    let window = OnboardingWindow::new().expect("create onboarding window");
    let settings = shell::snapshot();
    window.set_method(settings.method.id().into());
    window.set_launch_at_login(settings.launch_at_login);

    let weak = window.as_weak();
    window.on_pick_method(move |value| {
        if let Some(method) = Method::from_id(&value) {
            commands::set_method(method);
        }
        if let Some(window) = weak.upgrade() {
            window.set_method(value);
        }
    });
    window.on_set_launch(commands::set_launch_at_login);

    let weak = window.as_weak();
    window.on_finish(move || {
        commands::complete_onboarding();
        if let Some(window) = weak.upgrade() {
            let _ = window.hide();
        }
    });

    let _ = window.show();
    WINDOW.with(|cell| *cell.borrow_mut() = Some(window.as_weak()));
}
