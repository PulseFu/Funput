//! Interposer orchestration: spawn the child in a PTY and shuttle bytes both
//! ways, composing Vietnamese on the input path.

use std::io::{self, Read, Write};
use std::sync::Arc;
use std::thread;

use funput_core::InputMethod;
use funput_engine::{Action, Engine};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

use crate::config::TermConfig;
use crate::inject::result_bytes;
use crate::input::{ByteKind, Classifier};
use crate::output::forward_output;
use crate::state::SharedState;
use crate::term::{RawModeGuard, set_cursor_cue, set_title};

/// Run options resolved from config, env, and the command line.
pub struct Options {
    pub config: TermConfig,
    pub command: Vec<String>,
}

/// The user-visible composition state, reported whenever it changes so the caller
/// can refresh the indicators (window title + cursor cue).
#[derive(Debug, Clone, Copy)]
pub struct Status {
    /// VI (composing) vs EN (passthrough).
    pub enabled: bool,
    /// Active input method.
    pub method: InputMethod,
}

/// The other input method — used to cycle Telex↔VNI.
fn other_method(method: InputMethod) -> InputMethod {
    match method {
        InputMethod::Telex => InputMethod::Vni,
        InputMethod::Vni => InputMethod::Telex,
    }
}

/// Read keystrokes from `reader`, compose, and write the result bytes to `writer`.
///
/// Pure of real I/O — the caller injects the reader/writer and a toggle callback,
/// so this is unit-tested with in-memory pipes.
pub fn forward_input<R, W, F>(
    mut reader: R,
    mut writer: W,
    config: &TermConfig,
    state: &SharedState,
    mut on_status: F,
) -> io::Result<()>
where
    R: Read,
    W: Write,
    F: FnMut(Status),
{
    let mut engine = Engine::new();
    config.apply_to(&mut engine);
    engine.arm_capitalization();
    let mut classifier = Classifier::new(config.toggle, config.cycle_method);
    let mut buf = [0u8; 4096];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        for &byte in &buf[..n] {
            match classifier.classify(byte) {
                ByteKind::Toggle => {
                    let enabled = state.toggle();
                    engine.clear();
                    on_status(Status {
                        enabled,
                        method: engine.method(),
                    });
                }
                // Cycle Telex↔VNI live; flush the in-progress word so it doesn't
                // carry half-composed under the old method.
                ByteKind::CycleMethod => {
                    let method = other_method(engine.method());
                    engine.set_method(method);
                    engine.clear();
                    on_status(Status {
                        enabled: state.enabled(),
                        method,
                    });
                }
                ByteKind::Printable(ch) if state.composing() => {
                    let result = engine.process_char(ch);
                    writer.write_all(&result_bytes(ch, &result))?;
                }
                // Backspace: drop the last char from the composition so the next
                // key composes against the corrected text ("Phua" ⌫ "s" → "Phú"),
                // then pass it so the app deletes its own last char.
                ByteKind::Control if state.composing() && is_backspace(byte) => {
                    engine.on_backspace();
                    writer.write_all(&[byte])?;
                }
                // Whitespace controls (Enter, Tab) are word boundaries: route them
                // through the engine so English-restore fires before they reach the
                // child (e.g. typing "text"+Enter submits "text", not "tẽt").
                ByteKind::Control if state.composing() && is_ws_boundary(byte) => {
                    let ch = byte as char;
                    let result = engine.process_char(ch);
                    match result.action {
                        Action::None => writer.write_all(&[byte])?,
                        _ => writer.write_all(&result_bytes(ch, &result))?,
                    }
                }
                // Bracketed-paste content: forward verbatim, never compose. The
                // start marker already flushed composition via the arm below.
                ByteKind::Paste => writer.write_all(&[byte])?,
                // Control / escape / utf8 / printable-while-disabled: end the
                // current word and forward the byte unchanged.
                _ => {
                    engine.clear();
                    writer.write_all(&[byte])?;
                }
            }
        }
        writer.flush()?;
    }
    Ok(())
}

/// Whitespace control bytes that act as word boundaries (Tab, LF, CR/Enter).
fn is_ws_boundary(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\r')
}

/// Backspace bytes — DEL (0x7f, common in terminals) or BS (0x08).
fn is_backspace(byte: u8) -> bool {
    matches!(byte, 0x7f | 0x08)
}

fn pty_err<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::other(e.to_string())
}

/// Spawn `opts.command` in a PTY and run the interposer until the child exits.
/// Returns the child's exit code.
pub fn run(opts: Options) -> io::Result<i32> {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let size = PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    };

    // Enter raw mode first: if there is no real terminal this fails fast, before
    // we spawn a child we'd have to clean up. Restored on drop.
    let _raw = RawModeGuard::enter()?;

    let pair = native_pty_system().openpty(size).map_err(pty_err)?;

    let mut cmd = CommandBuilder::new(&opts.command[0]);
    for arg in &opts.command[1..] {
        cmd.arg(arg);
    }
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }
    for (key, value) in std::env::vars() {
        cmd.env(key, value);
    }

    let mut child = pair.slave.spawn_command(cmd).map_err(pty_err)?;
    drop(pair.slave);

    let writer = pair.master.take_writer().map_err(pty_err)?;
    let reader = pair.master.try_clone_reader().map_err(pty_err)?;

    let state = Arc::new(SharedState::new(opts.config.enabled));

    spawn_resize_thread(pair.master);

    // Reflect the initial composition state in the title and cursor cue.
    let vi_color = opts.config.vi_cursor_color.clone();
    update_indicators(
        Status {
            enabled: opts.config.enabled,
            method: opts.config.method,
        },
        &vi_color,
    );

    // Child -> terminal (own thread; ends at EOF when the child exits).
    let state_out = Arc::clone(&state);
    let output = thread::spawn(move || {
        let _ = forward_output(reader, io::stdout(), &state_out);
    });

    // Terminal -> child (detached; blocks on stdin until the process exits).
    let state_in = Arc::clone(&state);
    let config = opts.config;
    let status_color = vi_color.clone();
    thread::spawn(move || {
        let _ = forward_input(io::stdin(), writer, &config, &state_in, |status| {
            update_indicators(status, &status_color);
        });
    });

    let status = child.wait().map_err(pty_err)?;
    let _ = output.join();

    // Restore the user's default cursor — never leave it recolored after exit.
    let _ = set_cursor_cue(&mut io::stdout(), false, &vi_color);

    Ok(status.exit_code() as i32)
}

/// Update both VI/EN indicators (window title + cursor color) to match `status`.
/// The title also shows the active method so a live Telex↔VNI switch is visible.
fn update_indicators(status: Status, vi_color: &str) {
    let mut out = io::stdout();
    let vi_en = if status.enabled { "VI" } else { "EN" };
    let method = match status.method {
        InputMethod::Telex => "Telex",
        InputMethod::Vni => "VNI",
    };
    let _ = set_title(&mut out, &format!("Funput · {vi_en} · {method}"));
    let _ = set_cursor_cue(&mut out, status.enabled, vi_color);
}

#[cfg(unix)]
fn spawn_resize_thread(master: Box<dyn portable_pty::MasterPty + Send>) {
    use signal_hook::consts::SIGWINCH;
    use signal_hook::iterator::Signals;

    thread::spawn(move || {
        let Ok(mut signals) = Signals::new([SIGWINCH]) else {
            return;
        };
        for _ in signals.forever() {
            if let Ok((cols, rows)) = crossterm::terminal::size() {
                let _ = master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
        }
    });
}

#[cfg(not(unix))]
fn spawn_resize_thread(_master: Box<dyn portable_pty::MasterPty + Send>) {
    // Windows resize handling lands with ConPTY support (phase TT6).
}

#[cfg(test)]
mod tests {
    use super::*;

    use funput_core::InputMethod;

    fn config_for(method: InputMethod) -> TermConfig {
        TermConfig {
            method,
            ..TermConfig::default()
        }
    }

    fn compose(method: InputMethod, input: &[u8]) -> Vec<u8> {
        let state = SharedState::new(true);
        let mut out = Vec::new();
        forward_input(input, &mut out, &config_for(method), &state, |_| {}).unwrap();
        out
    }

    #[test]
    fn telex_word_emits_backspace_and_unicode() {
        // "as": 'a' passes through, 's' deletes 'a' and injects 'á'.
        let mut expected = b"a".to_vec();
        expected.push(0x7f);
        expected.extend_from_slice("á".as_bytes());
        assert_eq!(compose(InputMethod::Telex, b"as"), expected);
    }

    /// Reconstruct the child's visible text from the byte stream we send it
    /// (DEL = backspace one char; other bytes are injected UTF-8).
    fn reconstruct(bytes: &[u8]) -> String {
        let mut text = String::new();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b == 0x7f {
                text.pop();
                i += 1;
            } else {
                let len = match b {
                    _ if b < 0x80 => 1,
                    _ if b >> 5 == 0b110 => 2,
                    _ if b >> 4 == 0b1110 => 3,
                    _ => 4,
                };
                if let Ok(s) = std::str::from_utf8(&bytes[i..i + len]) {
                    text.push_str(s);
                }
                i += len;
            }
        }
        text
    }

    #[test]
    fn enter_triggers_english_restore() {
        // "text" composes to "tẽt", but Enter (a boundary) restores the raw word.
        let out = compose(InputMethod::Telex, b"text\r");
        assert_eq!(reconstruct(&out), "text\r");
    }

    #[test]
    fn backspace_corrects_mid_composition() {
        // "Phua" (typo), Backspace the "a", then "s" → "Phú".
        let out = compose(InputMethod::Telex, b"Phua\x7fs");
        assert_eq!(reconstruct(&out), "Phú");
    }

    #[test]
    fn double_modifier_revert_sends_no_extra_char() {
        // "mix" → "mĩ"; pressing 'x' again reverts to the raw "mix" (one x, not
        // "mixx"). Confirms funput-term emits the correct bytes.
        let out = compose(InputMethod::Telex, b"mixx");
        assert_eq!(reconstruct(&out), "mix");
    }

    #[test]
    fn enter_keeps_valid_vietnamese() {
        // A real syllable is finalized, not restored.
        let out = compose(InputMethod::Telex, b"mas\r");
        assert_eq!(reconstruct(&out), "má\r");
    }

    #[test]
    fn bracketed_paste_is_forwarded_verbatim() {
        // "as" pasted inside ESC[200~…ESC[201~ must stay "as", not compose to
        // "á"; the markers themselves are forwarded to the child untouched.
        let out = compose(InputMethod::Telex, b"\x1b[200~as\x1b[201~");
        assert!(out.starts_with(b"\x1b[200~"));
        assert!(out.ends_with(b"\x1b[201~"));
        assert_eq!(reconstruct(&out), "\x1b[200~as\x1b[201~");
    }

    #[test]
    fn composition_resumes_after_paste() {
        // Compose, paste raw, then compose again — all in one buffer.
        let out = compose(InputMethod::Telex, b"as\x1b[200~as\x1b[201~as");
        // First "as" → "á", pasted "as" stays literal, trailing "as" → "á".
        assert_eq!(reconstruct(&out), "á\x1b[200~as\x1b[201~á");
    }

    #[test]
    fn toggle_off_disables_composition() {
        let state = SharedState::new(true);
        let mut out = Vec::new();
        let mut toggles = Vec::new();
        // Ctrl-\ then "as": composition is off, so bytes pass through raw.
        forward_input(
            &[0x1c, b'a', b's'][..],
            &mut out,
            &config_for(InputMethod::Telex),
            &state,
            |status| toggles.push(status.enabled),
        )
        .unwrap();
        assert_eq!(out, b"as");
        assert_eq!(toggles, vec![false]);
        assert!(!state.composing());
    }

    #[test]
    fn cycle_method_key_switches_telex_vni() {
        // Start Telex (cycle key Ctrl-^ = 0x1e): "as" → "á". After cycling to VNI,
        // "as" stays "as" (s is not a VNI modifier). Status reports the new method.
        let config = TermConfig {
            cycle_method: Some(0x1e),
            ..config_for(InputMethod::Telex)
        };
        let state = SharedState::new(true);
        let mut out = Vec::new();
        let mut methods = Vec::new();
        forward_input(b"as\x1eas".as_ref(), &mut out, &config, &state, |s| {
            methods.push(s.method)
        })
        .unwrap();
        assert_eq!(reconstruct(&out), "áas");
        assert_eq!(methods, vec![InputMethod::Vni]);
    }

    #[test]
    fn config_disabled_composes_nothing() {
        // enabled = false in config → start in EN, keystrokes pass through raw.
        let config = TermConfig {
            enabled: false,
            ..config_for(InputMethod::Telex)
        };
        let state = SharedState::new(config.enabled);
        let mut out = Vec::new();
        forward_input(b"as".as_ref(), &mut out, &config, &state, |_| {}).unwrap();
        assert_eq!(out, b"as");
    }

    #[test]
    fn config_shortcut_expands_at_boundary() {
        // A gõ-tắt shortcut from config expands when the word boundary arrives.
        let config = TermConfig {
            shortcuts: vec![("vn".to_string(), "Việt Nam".to_string())],
            ..config_for(InputMethod::Telex)
        };
        let state = SharedState::new(true);
        let mut out = Vec::new();
        forward_input(b"vn ".as_ref(), &mut out, &config, &state, |_| {}).unwrap();
        assert_eq!(reconstruct(&out), "Việt Nam ");
    }
}
