//! Reverse of compose: turn finished Vietnamese text into the Telex/VNI keystrokes
//! that would produce it. Self-contained (Unicode NFD), used by the coverage check
//! to round-trip a corpus: `text → encode → engine → text`.

use unicode_normalization::UnicodeNormalization;

use crate::sim::Method;

#[derive(Clone, Copy)]
enum Shape {
    Circumflex, // â ê ô
    Breve,      // ă
    Horn,       // ơ ư
}

#[derive(Clone, Copy)]
enum Tone {
    Grave, // huyền
    Acute, // sắc
    Hook,  // hỏi
    Tilde, // ngã
    Dot,   // nặng
}

/// Encode finished Vietnamese `text` into the keystrokes for `method`.
pub fn encode(text: &str, method: Method) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        encode_char(ch, method, &mut out);
    }
    out
}

fn encode_char(ch: char, method: Method, out: &mut String) {
    // `đ`/`Đ` do not decompose under NFD — handle the stroke explicitly.
    match ch {
        'đ' => return push_stroke(method, 'd', out),
        'Đ' => return push_stroke(method, 'D', out),
        _ => {}
    }

    let marks: Vec<char> = ch.nfd().collect();
    let base = marks[0];
    out.push(base);

    let mut shape = None;
    let mut tone = None;
    for &m in &marks[1..] {
        match m {
            '\u{0302}' => shape = Some(Shape::Circumflex),
            '\u{0306}' => shape = Some(Shape::Breve),
            '\u{031B}' => shape = Some(Shape::Horn),
            '\u{0300}' => tone = Some(Tone::Grave),
            '\u{0301}' => tone = Some(Tone::Acute),
            '\u{0309}' => tone = Some(Tone::Hook),
            '\u{0303}' => tone = Some(Tone::Tilde),
            '\u{0323}' => tone = Some(Tone::Dot),
            _ => {}
        }
    }

    // Shape first, then tone — order is method-defined, not the NFD order.
    if let Some(s) = shape {
        push_shape(method, base, s, out);
    }
    if let Some(t) = tone {
        out.push(tone_key(method, t));
    }
}

/// `đ`: Telex doubles the `d` (`dd`/`Dd`); VNI uses the `9` modifier (`d9`/`D9`).
fn push_stroke(method: Method, d: char, out: &mut String) {
    out.push(d);
    match method {
        Method::Telex => out.push('d'),
        Method::Vni => out.push('9'),
    }
}

fn push_shape(method: Method, base: char, shape: Shape, out: &mut String) {
    match method {
        // Telex: circumflex doubles the vowel (`aa`→â); breve/horn use `w`.
        Method::Telex => match shape {
            Shape::Circumflex => out.push(base.to_ascii_lowercase()),
            Shape::Breve | Shape::Horn => out.push('w'),
        },
        // VNI: 6 = circumflex, 8 = breve, 7 = horn.
        Method::Vni => out.push(match shape {
            Shape::Circumflex => '6',
            Shape::Breve => '8',
            Shape::Horn => '7',
        }),
    }
}

fn tone_key(method: Method, tone: Tone) -> char {
    match method {
        Method::Telex => match tone {
            Tone::Acute => 's',
            Tone::Grave => 'f',
            Tone::Hook => 'r',
            Tone::Tilde => 'x',
            Tone::Dot => 'j',
        },
        Method::Vni => match tone {
            Tone::Acute => '1',
            Tone::Grave => '2',
            Tone::Hook => '3',
            Tone::Tilde => '4',
            Tone::Dot => '5',
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{simulate_with, SimConfig};
    use funput_core::ToneStyle;

    /// The meaningful property: encoding a word and typing it back reproduces it.
    /// Smart-restore off to isolate pure composition.
    fn roundtrip(word: &str, method: Method) -> String {
        let keys = encode(word, method);
        let config = SimConfig {
            method,
            tone_style: ToneStyle::Traditional,
            smart_restore: false,
            spell_check: false,
        };
        simulate_with(config, &keys).app_text
    }

    // Words chosen to be tone-style-invariant (no oa/oe/uy glide) so both Telex and
    // VNI round-trip exactly under the traditional style.
    const WORDS: &[&str] = &[
        "đầu", "việt", "nước", "Đắk", "nam", "tiếng", "người", "được", "rượu", "nghiêng",
        "Ô", "khuỷu",
    ];

    #[test]
    fn telex_roundtrip() {
        for &w in WORDS {
            assert_eq!(roundtrip(w, Method::Telex), w, "telex: {w}");
        }
    }

    #[test]
    fn vni_roundtrip() {
        for &w in WORDS {
            assert_eq!(roundtrip(w, Method::Vni), w, "vni: {w}");
        }
    }
}
