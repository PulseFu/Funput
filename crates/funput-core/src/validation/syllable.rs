//! Syllable-structure validation for modifier keys (tone / shape / stroke).
//!
//! Decides whether a modifier should apply, be ignored, or pass through as a
//! literal key (non-Vietnamese structure the engine restores later).

use crate::unicode::marks::vowel_stem;
use crate::validation::parse::{is_valid_onset, parse_syllable};

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

const VALID_CODAS: &[&str] = &["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

fn violates_ckg_spelling(onset: &str, nucleus: &str) -> bool {
    let Some(first) = nucleus.chars().next().and_then(vowel_stem) else {
        return false;
    };
    let stem = char::to_lowercase(first).next().unwrap_or(first);

    match onset.to_lowercase().as_str() {
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

    if parts.invalid_onset || (!parts.onset.is_empty() && !is_valid_onset(&parts.onset.to_lowercase()))
    {
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
    match buffer.chars().last() {
        Some('d' | 'D') => ModifierValidation::Allow,
        _ => ModifierValidation::Ignored,
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
        // `gi` digraph stays valid, `ge` would need `gh`.
        assert_eq!(validate_tone("gi"), ModifierValidation::Allow);
    }
}
