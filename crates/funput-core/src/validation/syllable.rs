//! Syllable structure validation for modifier keys (tone / shape / stroke).

use crate::unicode::marks::{is_vowel, vowel_stem};

/// Result of validating a modifier keystroke against the current buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierValidation {
    /// Apply Vietnamese transform.
    Allow,
    /// No valid target — discard key.
    Ignored,
    /// Non-Vietnamese structure — append key literally (engine restores later).
    PassThrough,
}

/// Parsed components of a single syllable chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyllableParts {
    pub onset: String,
    pub nucleus: String,
    pub coda: String,
    /// Leading consonants do not form a valid Vietnamese onset.
    pub invalid_onset: bool,
}

const VALID_ONSETS: &[&str] = &[
    "", "b", "c", "ch", "d", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh",
    "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x",
];

const VALID_CODAS: &[&str] = &["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

/// Parse one syllable chunk into onset, vowel nucleus, and coda.
pub fn parse_syllable(buffer: &str) -> SyllableParts {
    if buffer.is_empty() {
        return SyllableParts {
            onset: String::new(),
            nucleus: String::new(),
            coda: String::new(),
            invalid_onset: false,
        };
    }

    let (onset, rest, invalid_onset) = match_onset(buffer);
    let mut nucleus_chars = String::new();
    let mut coda_chars = String::new();
    for ch in rest.chars() {
        if is_vowel(ch) {
            nucleus_chars.push(ch);
        } else {
            coda_chars.push(ch);
        }
    }

    SyllableParts {
        onset,
        nucleus: nucleus_chars,
        coda: coda_chars,
        invalid_onset,
    }
}

fn match_onset(buffer: &str) -> (String, &str, bool) {
    if let Some(first) = buffer.chars().next()
        && (first == 'đ' || first == 'Đ')
    {
        let prefix = first.to_string();
        let rest = buffer.strip_prefix(first).unwrap_or("");
        return (prefix, rest, false);
    }

    let lower = buffer.to_lowercase();
    let lower_chars: Vec<char> = lower.chars().collect();

    for len in (1..=3).rev() {
        if lower_chars.len() < len {
            continue;
        }
        let prefix_lower: String = lower_chars[..len].iter().collect();
        if !is_valid_onset(&prefix_lower) {
            continue;
        }
        let prefix: String = buffer.chars().take(len).collect();
        let rest = &buffer[prefix.len()..];

        // In `gi`, the `i` is part of the onset only when another vowel follows
        // (`gia`, `giết`). When `i` is the lone vowel it is the nucleus (`gì`,
        // `gìn`), so fall back to the shorter `g` onset.
        if prefix_lower == "gi" && !rest.chars().next().is_some_and(is_vowel) {
            continue;
        }

        return (prefix, rest, false);
    }

    if let Some(first) = buffer.chars().next() && is_vowel(first) {
        return (String::new(), buffer, false);
    }

    (String::new(), buffer, true)
}

fn is_valid_onset(onset_lower: &str) -> bool {
    if onset_lower.is_empty() {
        return true;
    }
    if onset_lower == "đ" {
        return true;
    }
    VALID_ONSETS.contains(&onset_lower)
}

fn first_vowel_stem(nucleus: &str) -> Option<char> {
    let first = nucleus.chars().next()?;
    vowel_stem(first)
}

fn fold_lower(c: char) -> char {
    char::to_lowercase(c).next().unwrap_or(c)
}

fn violates_ckg_spelling(onset: &str, nucleus: &str) -> bool {
    let stem = match first_vowel_stem(nucleus) {
        Some(s) => fold_lower(s),
        None => return false,
    };

    let onset_lower = onset.to_lowercase();
    match onset_lower.as_str() {
        "c" => !matches!(stem, 'a' | 'ă' | 'â' | 'o' | 'ô' | 'ơ' | 'u' | 'ư'),
        "k" => !matches!(stem, 'e' | 'ê' | 'i'),
        // `g` + `i` is the valid `gi` digraph (gì, gìn); `g` + e/ê uses `gh`.
        "g" => !matches!(stem, 'a' | 'ă' | 'â' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'i'),
        "gh" => !matches!(stem, 'e' | 'ê' | 'i'),
        "ngh" => !matches!(stem, 'e' | 'ê' | 'i'),
        _ => false,
    }
}

fn validate_modifier(buffer: &str) -> ModifierValidation {
    let parts = parse_syllable(buffer);

    if parts.invalid_onset {
        return ModifierValidation::PassThrough;
    }

    if !parts.onset.is_empty() && !is_valid_onset(&parts.onset.to_lowercase()) {
        return ModifierValidation::PassThrough;
    }

    if parts.nucleus.is_empty() {
        return ModifierValidation::Ignored;
    }

    if violates_ckg_spelling(&parts.onset, &parts.nucleus) {
        return ModifierValidation::PassThrough;
    }

    // Two or more trailing consonants can't form a Vietnamese coda → likely an
    // English word, pass the key through. A single trailing consonant is allowed
    // (the user may still be typing, e.g. "mix" → "mĩx").
    let coda_lower = parts.coda.to_lowercase();
    if parts.coda.chars().count() >= 2 && !VALID_CODAS.contains(&coda_lower.as_str()) {
        return ModifierValidation::PassThrough;
    }

    ModifierValidation::Allow
}

/// Validate tone key (1–5) against the current buffer.
pub fn validate_tone(buffer: &str) -> ModifierValidation {
    validate_modifier(buffer)
}

/// Validate shape key (6–8) against the current buffer.
pub fn validate_shape(buffer: &str) -> ModifierValidation {
    validate_modifier(buffer)
}

/// Validate stroke key (9) against the current buffer.
pub fn validate_stroke(buffer: &str) -> ModifierValidation {
    let Some(last) = buffer.chars().last() else {
        return ModifierValidation::Ignored;
    };
    if last == 'd' || last == 'D' {
        ModifierValidation::Allow
    } else {
        ModifierValidation::Ignored
    }
}

/// Returns true if the syllable structure is valid for transform.
#[allow(dead_code)] // Public API for engine / future callers.
pub fn is_valid(buffer: &str) -> bool {
    matches!(validate_modifier(buffer), ModifierValidation::Allow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_syllable_cases() {
        assert_eq!(
            parse_syllable("tr"),
            SyllableParts {
                onset: "tr".into(),
                nucleus: String::new(),
                coda: String::new(),
                invalid_onset: false,
            }
        );
        assert_eq!(
            parse_syllable("ng"),
            SyllableParts {
                onset: "ng".into(),
                nucleus: String::new(),
                coda: String::new(),
                invalid_onset: false,
            }
        );
        assert_eq!(
            parse_syllable("ma"),
            SyllableParts {
                onset: "m".into(),
                nucleus: "a".into(),
                coda: String::new(),
                invalid_onset: false,
            }
        );
        assert_eq!(
            parse_syllable("text"),
            SyllableParts {
                onset: "t".into(),
                nucleus: "e".into(),
                coda: "xt".into(),
                invalid_onset: false,
            }
        );
        assert_eq!(
            parse_syllable("mix"),
            SyllableParts {
                onset: "m".into(),
                nucleus: "i".into(),
                coda: "x".into(),
                invalid_onset: false,
            }
        );
        assert_eq!(
            parse_syllable("trung"),
            SyllableParts {
                onset: "tr".into(),
                nucleus: "u".into(),
                coda: "ng".into(),
                invalid_onset: false,
            }
        );
        assert_eq!(
            parse_syllable("đ"),
            SyllableParts {
                onset: "đ".into(),
                nucleus: String::new(),
                coda: String::new(),
                invalid_onset: false,
            }
        );
    }

    #[test]
    fn validate_tone_cases() {
        assert_eq!(validate_tone("ng"), ModifierValidation::Ignored);
        assert_eq!(validate_tone("text"), ModifierValidation::PassThrough);
        assert_eq!(validate_tone("mix"), ModifierValidation::Allow);
        assert_eq!(validate_tone("ma"), ModifierValidation::Allow);
        assert_eq!(validate_tone("zt"), ModifierValidation::PassThrough);
    }

    #[test]
    fn validate_stroke_cases() {
        assert_eq!(validate_stroke("d"), ModifierValidation::Allow);
        assert_eq!(validate_stroke("x"), ModifierValidation::Ignored);
    }

    #[test]
    fn ckg_spelling() {
        assert_eq!(validate_tone("ke"), ModifierValidation::Allow);
        assert_eq!(validate_tone("ka"), ModifierValidation::PassThrough);
        assert_eq!(validate_tone("ca"), ModifierValidation::Allow);
    }
}
