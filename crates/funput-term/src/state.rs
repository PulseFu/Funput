//! Session flags shared across the input/output threads.

use std::sync::atomic::{AtomicBool, Ordering};

/// Composition state shared between threads via atomics (no locking).
#[derive(Debug)]
pub struct SharedState {
    /// User toggle: Vietnamese composition on/off.
    enabled: AtomicBool,
    /// Child is in the alternate screen (full-screen TUI) — composition pauses.
    alt_screen: AtomicBool,
}

impl SharedState {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: AtomicBool::new(enabled),
            alt_screen: AtomicBool::new(false),
        }
    }

    /// True when keystrokes should be composed (enabled and not in a full-screen app).
    pub fn composing(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && !self.alt_screen.load(Ordering::Relaxed)
    }

    /// The user's VI/EN toggle, independent of alt-screen — used for the indicator.
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Flip the toggle; returns the new value.
    pub fn toggle(&self) -> bool {
        !self.enabled.fetch_xor(true, Ordering::Relaxed)
    }

    pub fn set_alt_screen(&self, value: bool) {
        self.alt_screen.store(value, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_flips_and_reports_new_value() {
        let s = SharedState::new(true);
        assert!(s.composing());
        assert!(!s.toggle()); // true -> false
        assert!(!s.composing());
        assert!(s.toggle()); // false -> true
        assert!(s.composing());
    }

    #[test]
    fn alt_screen_pauses_composition() {
        let s = SharedState::new(true);
        s.set_alt_screen(true);
        assert!(!s.composing());
        // Leaving alt-screen resumes composing — proving the toggle stayed on.
        s.set_alt_screen(false);
        assert!(s.composing());
    }
}
