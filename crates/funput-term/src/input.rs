//! Pure classification of the raw keyboard byte stream.
//!
//! The driver feeds bytes one at a time; the classifier tracks just enough state
//! to recognise escape sequences (arrows, function keys, Alt-combos) so they are
//! never mistaken for composable letters.

/// What a single input byte means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteKind {
    /// Printable ASCII (`0x20..=0x7e`) — a candidate to feed the engine.
    Printable(char),
    /// Control byte (Enter, Tab, Backspace, Ctrl-key) — forward raw, flush composition.
    Control,
    /// Part of an escape sequence — forward raw, flush composition.
    Escape,
    /// UTF-8 lead/continuation (`>= 0x80`) — forward raw (pasted/precomposed text).
    Utf8,
    /// The configured toggle key — consume, do not forward.
    Toggle,
    /// The configured cycle-method key (Telex↔VNI) — consume, do not forward.
    CycleMethod,
    /// Byte inside a bracketed paste — forward raw, never compose.
    Paste,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Normal,
    /// Saw `ESC`; the next byte decides the sequence kind.
    AfterEsc,
    /// Inside a CSI (`ESC[`) or SS3 (`ESC O`) sequence.
    Csi,
}

/// Byte-stream classifier with minimal escape-sequence state.
#[derive(Debug)]
pub struct Classifier {
    toggle: u8,
    /// Key that cycles Telex↔VNI, or `None` when disabled.
    cycle_method: Option<u8>,
    phase: Phase,
    /// Inside a bracketed paste (`ESC[200~` … `ESC[201~`): forward content raw.
    in_paste: bool,
    /// Parameter bytes of the CSI sequence being parsed, kept only to recognise
    /// the `200`/`201` bracketed-paste markers. Capped at `MAX_CSI_PARAMS`.
    params: Vec<u8>,
}

const ESC: u8 = 0x1b;
/// Bracketed-paste markers (`ESC[200~` start, `ESC[201~` end).
const PASTE_START: &[u8] = b"200";
const PASTE_END: &[u8] = b"201";
/// Upper bound on retained CSI parameter bytes — enough for the markers we match,
/// while keeping a malformed, never-terminated CSI from growing the buffer.
const MAX_CSI_PARAMS: usize = PASTE_START.len();

impl Classifier {
    pub fn new(toggle: u8, cycle_method: Option<u8>) -> Self {
        Self {
            toggle,
            cycle_method,
            phase: Phase::Normal,
            in_paste: false,
            params: Vec::new(),
        }
    }

    pub fn classify(&mut self, byte: u8) -> ByteKind {
        match self.phase {
            Phase::Normal => self.classify_normal(byte),
            Phase::AfterEsc => {
                // `ESC [` (CSI) or `ESC O` (SS3) start a multi-byte sequence;
                // anything else is a 2-byte sequence (e.g. Alt+key).
                self.phase = if byte == b'[' || byte == b'O' {
                    self.params.clear();
                    Phase::Csi
                } else {
                    Phase::Normal
                };
                ByteKind::Escape
            }
            Phase::Csi => {
                // Final byte of a CSI/SS3 sequence is in `0x40..=0x7e`; bytes
                // before it are parameters we track to spot the bracketed-paste
                // markers.
                if (0x40..=0x7e).contains(&byte) {
                    self.phase = Phase::Normal;
                    if byte == b'~' {
                        match self.params.as_slice() {
                            PASTE_START => self.in_paste = true,
                            PASTE_END => self.in_paste = false,
                            _ => {}
                        }
                    }
                } else if self.params.len() <= MAX_CSI_PARAMS {
                    // Retain one byte beyond a marker's length so an over-long
                    // run (e.g. `2000`) can't equal a marker, then stop growing.
                    self.params.push(byte);
                }
                ByteKind::Escape
            }
        }
    }

    fn classify_normal(&mut self, byte: u8) -> ByteKind {
        if byte == ESC {
            self.phase = Phase::AfterEsc;
            return ByteKind::Escape;
        }
        // Inside a paste, every non-ESC byte is literal content: forward it raw
        // before any toggle/printable handling, so pasted letters and even the
        // toggle key are never interpreted as commands.
        if self.in_paste {
            return ByteKind::Paste;
        }
        if byte == self.toggle {
            return ByteKind::Toggle;
        }
        if self.cycle_method == Some(byte) {
            return ByteKind::CycleMethod;
        }
        match byte {
            0x20..=0x7e => ByteKind::Printable(byte as char),
            0x80..=0xff => ByteKind::Utf8,
            _ => ByteKind::Control,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTRL_BACKSLASH: u8 = 0x1c;

    fn classify_all(toggle: u8, bytes: &[u8]) -> Vec<ByteKind> {
        let mut c = Classifier::new(toggle, None);
        bytes.iter().map(|&b| c.classify(b)).collect()
    }

    #[test]
    fn printable_letters_and_space() {
        assert_eq!(
            classify_all(CTRL_BACKSLASH, b"a s"),
            vec![
                ByteKind::Printable('a'),
                ByteKind::Printable(' '),
                ByteKind::Printable('s'),
            ]
        );
    }

    #[test]
    fn control_bytes() {
        // Enter, Tab, Backspace, Ctrl-C.
        assert_eq!(
            classify_all(CTRL_BACKSLASH, &[0x0d, 0x09, 0x7f, 0x03]),
            vec![ByteKind::Control; 4]
        );
    }

    #[test]
    fn toggle_key_recognised() {
        assert_eq!(
            classify_all(CTRL_BACKSLASH, &[CTRL_BACKSLASH]),
            vec![ByteKind::Toggle]
        );
    }

    #[test]
    fn cycle_method_key_recognised_only_when_configured() {
        const CTRL_CARET: u8 = 0x1e;
        // Configured → CycleMethod; nothing else changes.
        let mut c = Classifier::new(CTRL_BACKSLASH, Some(CTRL_CARET));
        assert_eq!(c.classify(CTRL_CARET), ByteKind::CycleMethod);
        assert_eq!(c.classify(CTRL_BACKSLASH), ByteKind::Toggle);
        // Disabled → the same byte is just a control byte.
        let mut c = Classifier::new(CTRL_BACKSLASH, None);
        assert_eq!(c.classify(CTRL_CARET), ByteKind::Control);
    }

    #[test]
    fn arrow_key_is_escape_sequence() {
        // Up arrow = ESC [ A — all three bytes are Escape, then back to normal.
        let mut c = Classifier::new(CTRL_BACKSLASH, None);
        assert_eq!(c.classify(0x1b), ByteKind::Escape);
        assert_eq!(c.classify(b'['), ByteKind::Escape);
        assert_eq!(c.classify(b'A'), ByteKind::Escape);
        assert_eq!(c.classify(b'a'), ByteKind::Printable('a')); // sequence ended
    }

    #[test]
    fn alt_key_is_two_byte_escape() {
        let mut c = Classifier::new(CTRL_BACKSLASH, None);
        assert_eq!(c.classify(0x1b), ByteKind::Escape);
        assert_eq!(c.classify(b'x'), ByteKind::Escape); // ESC x = Alt-x
        assert_eq!(c.classify(b'y'), ByteKind::Printable('y'));
    }

    #[test]
    fn utf8_bytes_passthrough() {
        // "á" = 0xC3 0xA1
        assert_eq!(
            classify_all(CTRL_BACKSLASH, "á".as_bytes()),
            vec![ByteKind::Utf8, ByteKind::Utf8]
        );
    }

    #[test]
    fn bracketed_paste_content_is_raw() {
        // ESC[200~as ESC[201~b : "as" is paste content (not composed), then
        // "b" composes normally once the paste ends.
        let mut c = Classifier::new(CTRL_BACKSLASH, None);
        for &b in b"\x1b[200~" {
            assert_eq!(c.classify(b), ByteKind::Escape);
        }
        assert_eq!(c.classify(b'a'), ByteKind::Paste);
        assert_eq!(c.classify(b's'), ByteKind::Paste);
        for &b in b"\x1b[201~" {
            assert_eq!(c.classify(b), ByteKind::Escape);
        }
        assert_eq!(c.classify(b'b'), ByteKind::Printable('b'));
    }

    #[test]
    fn paste_marker_split_across_chunks() {
        // The marker can arrive byte-by-byte across reads; classifier state
        // persists, so paste mode still engages.
        let mut c = Classifier::new(CTRL_BACKSLASH, None);
        for &b in b"\x1b[20" {
            c.classify(b);
        }
        for &b in b"0~" {
            c.classify(b);
        }
        assert_eq!(c.classify(b'x'), ByteKind::Paste);
    }

    #[test]
    fn toggle_and_letters_inside_paste_are_raw() {
        // Pasted content must never be interpreted as commands: the toggle key
        // and letters alike are literal Paste bytes.
        let mut c = Classifier::new(CTRL_BACKSLASH, None);
        for &b in b"\x1b[200~" {
            c.classify(b);
        }
        assert_eq!(c.classify(CTRL_BACKSLASH), ByteKind::Paste);
        assert_eq!(c.classify(b'a'), ByteKind::Paste);
    }

    #[test]
    fn over_long_csi_is_not_a_paste_marker() {
        // A CSI whose parameters exceed a marker's length must not toggle paste,
        // and its parameter buffer must stay bounded.
        let mut c = Classifier::new(CTRL_BACKSLASH, None);
        for &b in b"\x1b[200000~" {
            c.classify(b);
        }
        assert_eq!(c.classify(b'a'), ByteKind::Printable('a'));
        assert!(c.params.len() <= MAX_CSI_PARAMS + 1);
    }
}
