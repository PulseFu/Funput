//! Platform action and inject payload after one keystroke.

/// What the platform shell should do with the current key event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Pass the key through to the app — no inject.
    None,
    /// Delete `backspace` chars in the app, then inject `chars[..count]`.
    Send,
    /// Restore pre-composition text (e.g. ESC) — phase E5+.
    Restore,
}

/// Output of processing one keystroke.
///
/// Rust-native shape for ergonomic use and testing. `funput-ffi` marshals this
/// into its `#[repr(C)]` struct (`backspace: u8`, `chars: [u32; 32]`, `count: u8`)
/// at the FFI boundary — the 32-char / `u8` limits live there, not here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImeResult {
    pub action: Action,
    /// Characters to delete before injecting `output`.
    pub backspace: usize,
    /// Characters to inject after the backspaces (empty for [`Action::None`]).
    pub output: String,
}

impl ImeResult {
    /// No platform inject — pass the key through.
    pub fn none() -> Self {
        Self {
            action: Action::None,
            backspace: 0,
            output: String::new(),
        }
    }

    /// Delete `backspace` chars, then inject `output`.
    pub fn send(backspace: usize, output: String) -> Self {
        Self {
            action: Action::Send,
            backspace,
            output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_result_defaults() {
        let result = ImeResult::none();
        assert_eq!(result.action, Action::None);
        assert_eq!(result.backspace, 0);
        assert!(result.output.is_empty());
    }

    #[test]
    fn send_carries_backspace_and_output() {
        let result = ImeResult::send(1, "á".into());
        assert_eq!(result.action, Action::Send);
        assert_eq!(result.backspace, 1);
        assert_eq!(result.output, "á");
    }
}
