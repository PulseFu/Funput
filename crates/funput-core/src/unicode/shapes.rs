//! Vowel shape modifiers (mũ / móc / trần) and their glyph mappings.

use crate::unicode::marks::{apply_tone, tone_on_vowel, vowel_stem};

/// Vowel shape modifiers (mũ, móc, trần).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelShape {
    Circumflex,
    Horn,
    Breve,
}

/// Apply a shape modifier to a base vowel.
pub fn apply_shape(base: char, shape: VowelShape) -> Option<char> {
    match (base, shape) {
        ('a' | 'A', VowelShape::Circumflex) => Some(case_pair(base, 'â', 'Â')),
        ('e' | 'E', VowelShape::Circumflex) => Some(case_pair(base, 'ê', 'Ê')),
        ('o' | 'O', VowelShape::Circumflex) => Some(case_pair(base, 'ô', 'Ô')),
        ('o' | 'O', VowelShape::Horn) => Some(case_pair(base, 'ơ', 'Ơ')),
        ('u' | 'U', VowelShape::Horn) => Some(case_pair(base, 'ư', 'Ư')),
        ('a' | 'A', VowelShape::Breve) => Some(case_pair(base, 'ă', 'Ă')),
        _ => None,
    }
}

/// Apply a shape to any vowel, stripping an existing tone first.
pub fn apply_shape_to_vowel(vowel: char, shape: VowelShape) -> Option<char> {
    let stem = vowel_stem(vowel)?;
    apply_shape(stem, shape)
}

/// Index of the last vowel that can *receive* `shape` (for applying 6/7/8).
///
/// Picks the appropriate target rather than the last vowel: e.g. `muoi` + `6`
/// targets `o` (→ `muôi`), not the trailing `i` which cannot take a circumflex.
pub fn shape_target_index(syllable: &str, shape: VowelShape) -> Option<usize> {
    let mut first: Option<usize> = None;
    let mut last: Option<usize> = None;
    for (i, ch) in syllable.chars().enumerate() {
        if apply_shape_to_vowel(ch, shape).is_some() {
            first.get_or_insert(i);
            last = Some(i);
        }
    }
    // Horn on a bare `uu` run forms the falling diphthong `ưu` (cừu, trừu, cứu) —
    // the *first* `u` is horned. Other shapes (and single-candidate clusters)
    // target the last receiving vowel, e.g. `muoi` + circumflex → `muôi`. The `uo`
    // horn compound (→ `ươ`) is handled earlier by `apply_uo_compound`.
    match shape {
        VowelShape::Horn => first,
        _ => last,
    }
}

/// Index of the last vowel currently carrying `shape` (for reverting 6/7/8).
pub fn shaped_vowel_index(syllable: &str, shape: VowelShape) -> Option<usize> {
    let mut target: Option<usize> = None;
    for (i, ch) in syllable.chars().enumerate() {
        if shape_on_vowel(ch) == Some(shape) {
            target = Some(i);
        }
    }
    target
}

/// Detect vowel shape modifier on a vowel (ignores tone).
pub(crate) fn shape_on_vowel(vowel: char) -> Option<VowelShape> {
    let stem = vowel_stem(vowel)?;
    Some(shaped_entry(stem)?.shape)
}

/// Remove mũ/móc/trần from a vowel, preserving tone if present.
pub(crate) fn strip_shape(vowel: char) -> Option<char> {
    let tone = tone_on_vowel(vowel);
    let shaped_stem = vowel_stem(vowel)?;
    let entry = shaped_entry(shaped_stem)?;
    let unshaped = case_pair(
        shaped_stem,
        entry.base_lower,
        char::to_uppercase(entry.base_lower)
            .next()
            .unwrap_or(entry.base_lower),
    );
    match tone {
        Some(t) => apply_tone(unshaped, t),
        None => Some(unshaped),
    }
}

fn case_pair(base: char, lower: char, upper: char) -> char {
    if base.is_uppercase() {
        upper
    } else {
        lower
    }
}

struct ShapedVowel {
    shaped_lower: char,
    base_lower: char,
    shape: VowelShape,
}

const SHAPED_VOWELS: &[ShapedVowel] = &[
    ShapedVowel {
        shaped_lower: 'â',
        base_lower: 'a',
        shape: VowelShape::Circumflex,
    },
    ShapedVowel {
        shaped_lower: 'ă',
        base_lower: 'a',
        shape: VowelShape::Breve,
    },
    ShapedVowel {
        shaped_lower: 'ê',
        base_lower: 'e',
        shape: VowelShape::Circumflex,
    },
    ShapedVowel {
        shaped_lower: 'ô',
        base_lower: 'o',
        shape: VowelShape::Circumflex,
    },
    ShapedVowel {
        shaped_lower: 'ơ',
        base_lower: 'o',
        shape: VowelShape::Horn,
    },
    ShapedVowel {
        shaped_lower: 'ư',
        base_lower: 'u',
        shape: VowelShape::Horn,
    },
];

fn shaped_entry(shaped_stem: char) -> Option<&'static ShapedVowel> {
    let lower = char::to_lowercase(shaped_stem).next().unwrap_or(shaped_stem);
    SHAPED_VOWELS.iter().find(|entry| entry.shaped_lower == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_shape_basic() {
        assert_eq!(apply_shape('a', VowelShape::Circumflex), Some('â'));
        assert_eq!(apply_shape('o', VowelShape::Horn), Some('ơ'));
        assert_eq!(apply_shape('a', VowelShape::Breve), Some('ă'));
    }

    #[test]
    fn apply_shape_to_vowel_strips_tone() {
        assert_eq!(apply_shape_to_vowel('á', VowelShape::Breve), Some('ă'));
        assert_eq!(apply_shape_to_vowel('â', VowelShape::Circumflex), None);
    }

    #[test]
    fn shape_target_index_picks_receiving_vowel() {
        // `muoi` + circumflex targets the `o` (index 2), not the trailing `i`.
        assert_eq!(shape_target_index("muoi", VowelShape::Circumflex), Some(2));
        assert_eq!(shape_target_index("loi", VowelShape::Circumflex), Some(1));
        assert_eq!(shape_target_index("to", VowelShape::Circumflex), Some(1));
        // No vowel can take the shape.
        assert_eq!(shape_target_index("ly", VowelShape::Circumflex), None);
    }

    #[test]
    fn shape_target_index_horn_on_uu_targets_first_u() {
        // Horn on a bare `uu` run makes `ưu` (cừu, trừu): the first `u` is horned,
        // not the trailing one (which would give the invalid `uư`).
        assert_eq!(shape_target_index("cuu", VowelShape::Horn), Some(1));
        assert_eq!(shape_target_index("truu", VowelShape::Horn), Some(2));
        // A single `u` is unambiguous.
        assert_eq!(shape_target_index("cu", VowelShape::Horn), Some(1));
    }

    #[test]
    fn shaped_vowel_index_finds_existing_shape() {
        assert_eq!(shaped_vowel_index("muôi", VowelShape::Circumflex), Some(2));
        assert_eq!(shaped_vowel_index("muoi", VowelShape::Circumflex), None);
    }

    #[test]
    fn shape_on_vowel_and_strip_shape() {
        for (shaped, shape, base) in [
            ('â', VowelShape::Circumflex, 'a'),
            ('ă', VowelShape::Breve, 'a'),
            ('ê', VowelShape::Circumflex, 'e'),
            ('ô', VowelShape::Circumflex, 'o'),
            ('ơ', VowelShape::Horn, 'o'),
            ('ư', VowelShape::Horn, 'u'),
        ] {
            assert_eq!(shape_on_vowel(shaped), Some(shape));
            assert_eq!(strip_shape(shaped), Some(base));
            assert_eq!(apply_shape(base, shape), Some(shaped));
        }
        assert_eq!(strip_shape('ấ'), Some('á'));
        assert_eq!(shape_on_vowel('ấ'), Some(VowelShape::Circumflex));
    }
}
