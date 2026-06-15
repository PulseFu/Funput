//! IME orchestration — session, buffer, and platform inject instructions.
//!
//! `funput-core` answers: given buffer + key, what is the new composed text?
//! `funput-engine` answers: after this key, what should the platform do?
//!
//! # Contract
//!
//! - **Stateful:** holds composition buffer across keystrokes.
//! - **Delegates transform:** all Telex/VNI rules live in `funput-core`.
//! - **No I/O:** no keyboard hooks, no inject — platform reads [`ImeResult`].

mod diff;
mod pipeline;
mod result;
mod session;

pub use result::{Action, ImeResult};
pub use session::Session;

/// Vietnamese IME engine — single source of truth for composition state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    session: Session,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            session: Session::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.session.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.session.enabled
    }

    pub fn set_method(&mut self, method: funput_core::InputMethod) {
        self.session.method = method;
    }

    pub fn method(&self) -> funput_core::InputMethod {
        self.session.method
    }

    pub fn clear(&mut self) {
        self.session.clear();
    }

    pub fn buffer(&self) -> &str {
        &self.session.buffer
    }

    /// Process one Unicode scalar (platform maps keycode → char).
    ///
    /// E0 stub: always returns [`Action::None`] and leaves the buffer unchanged.
    pub fn process_char(&mut self, _key: char) -> ImeResult {
        ImeResult::none()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use funput_core::InputMethod;

    #[test]
    fn engine_new_defaults() {
        let engine = Engine::new();
        assert!(engine.is_enabled());
        assert_eq!(engine.method(), InputMethod::Telex);
        assert_eq!(engine.buffer(), "");
    }

    #[test]
    fn set_method_vni() {
        let mut engine = Engine::new();
        engine.set_method(InputMethod::Vni);
        assert_eq!(engine.method(), InputMethod::Vni);
    }

    #[test]
    fn set_enabled_false() {
        let mut engine = Engine::new();
        engine.set_enabled(false);
        assert!(!engine.is_enabled());
    }

    #[test]
    fn clear_smoke() {
        let mut engine = Engine::new();
        engine.clear();
        assert_eq!(engine.buffer(), "");
    }

    #[test]
    fn process_char_stub_none() {
        let mut engine = Engine::new();
        let result = engine.process_char('a');
        assert_eq!(result.action, Action::None);
        assert_eq!(result.backspace, 0);
        assert!(result.output.is_empty());
        assert_eq!(engine.buffer(), "");
    }
}
