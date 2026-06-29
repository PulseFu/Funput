//! Real-terminal helpers: raw-mode guard and window-title indicator.

use std::io::{self, Write};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

/// Enables raw mode and restores it on drop — including on panic or early return,
/// so the user's terminal is never left in a broken state.
pub struct RawModeGuard;

impl RawModeGuard {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

/// Terminal multiplexer wrapping the session, if any. A bare OSC title sequence
/// is swallowed by tmux/screen, so it must be wrapped in their passthrough form
/// to reach the outer terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mux {
    None,
    Tmux,
    Screen,
}

/// Detect the surrounding multiplexer from the environment. `$TMUX` is set inside
/// tmux; GNU screen is recognised by `$STY` or a `screen`/`tmux` `$TERM` (best
/// effort, since screen's passthrough is more limited).
pub fn detect_mux() -> Mux {
    if std::env::var_os("TMUX").is_some() {
        return Mux::Tmux;
    }
    let term = std::env::var("TERM").unwrap_or_default();
    if std::env::var_os("STY").is_some() || term.starts_with("screen") || term.starts_with("tmux") {
        return Mux::Screen;
    }
    Mux::None
}

/// Default cursor color shown while Vietnamese composition is on — the brand
/// green used in the README badges.
pub const DEFAULT_VI_CURSOR_COLOR: &str = "#22C55E";

/// Wrap a terminal control sequence so it survives the surrounding multiplexer
/// and reaches the outer terminal.
///
/// Pure (no I/O) so the exact wrapping is unit-tested.
fn wrap_for_mux(seq: &str, mux: Mux) -> String {
    match mux {
        Mux::None => seq.to_string(),
        // tmux DCS passthrough: every inner ESC must be doubled.
        Mux::Tmux => format!("\x1bPtmux;{}\x1b\\", seq.replace('\x1b', "\x1b\x1b")),
        // screen DCS passthrough: forward the sequence as-is.
        Mux::Screen => format!("\x1bP{seq}\x1b\\"),
    }
}

/// Build the byte sequence that sets the terminal window title to `text`,
/// wrapped for `mux` so it reaches the outer terminal.
pub fn title_sequence(text: &str, mux: Mux) -> String {
    wrap_for_mux(&format!("\x1b]0;{text}\x07"), mux)
}

/// Build the cursor-color cue: a brand-colored cursor while composing (VI), reset
/// to the terminal default otherwise (EN). A non-intrusive indicator that works
/// even when the window title is hidden.
///
/// VI sets the cursor color via `OSC 12`; EN resets it via `OSC 112` rather than
/// guessing the user's default. Wrapped for `mux` like the title.
pub fn cursor_cue(enabled: bool, vi_color: &str, mux: Mux) -> String {
    let seq = if enabled {
        format!("\x1b]12;{vi_color}\x07")
    } else {
        "\x1b]112\x07".to_string()
    };
    wrap_for_mux(&seq, mux)
}

/// Set the terminal window title — a non-intrusive status indicator that does not
/// draw over the child app's UI, wrapped for any surrounding multiplexer.
pub fn set_title(out: &mut impl Write, text: &str) -> io::Result<()> {
    out.write_all(title_sequence(text, detect_mux()).as_bytes())?;
    out.flush()
}

/// Update the cursor-color cue to match the composition state. Always safe to
/// call with `enabled = false` on exit to restore the user's default cursor.
pub fn set_cursor_cue(out: &mut impl Write, enabled: bool, vi_color: &str) -> io::Result<()> {
    out.write_all(cursor_cue(enabled, vi_color, detect_mux()).as_bytes())?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_title_is_bare_osc() {
        assert_eq!(
            title_sequence("funput · VI", Mux::None),
            "\x1b]0;funput · VI\x07"
        );
    }

    #[test]
    fn tmux_title_doubles_inner_escapes() {
        assert_eq!(
            title_sequence("VI", Mux::Tmux),
            "\x1bPtmux;\x1b\x1b]0;VI\x07\x1b\\"
        );
    }

    #[test]
    fn screen_title_wraps_in_dcs() {
        assert_eq!(
            title_sequence("VI", Mux::Screen),
            "\x1bP\x1b]0;VI\x07\x1b\\"
        );
    }

    #[test]
    fn cursor_cue_sets_color_when_enabled() {
        assert_eq!(
            cursor_cue(true, "#22C55E", Mux::None),
            "\x1b]12;#22C55E\x07"
        );
    }

    #[test]
    fn cursor_cue_resets_when_disabled() {
        assert_eq!(cursor_cue(false, "#22C55E", Mux::None), "\x1b]112\x07");
    }

    #[test]
    fn cursor_cue_wraps_for_tmux() {
        // Inner ESC doubled inside the tmux DCS passthrough.
        assert_eq!(
            cursor_cue(true, "#fff", Mux::Tmux),
            "\x1bPtmux;\x1b\x1b]12;#fff\x07\x1b\\"
        );
    }

    #[test]
    fn cursor_cue_wraps_for_screen() {
        assert_eq!(
            cursor_cue(false, "#fff", Mux::Screen),
            "\x1bP\x1b]112\x07\x1b\\"
        );
    }
}
