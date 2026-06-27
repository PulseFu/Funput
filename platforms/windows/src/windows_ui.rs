//! Short-lived Slint UI processes for Settings and Onboarding.
//!
//! The background tray process only uses [`lifecycle`] to launch a child. Window
//! components and callbacks are initialized exclusively inside that child.

mod lifecycle;
mod models;
mod onboarding;
mod settings_callbacks;
mod settings_window;

pub(crate) use lifecycle::{
    launch_onboarding, launch_settings, run_onboarding, run_settings, terminate_children,
    terminate_parent_for_update,
};
pub(crate) use settings_window::set_update_state;
