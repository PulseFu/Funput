//! Unicode tone and vowel shape mappings.

use crate::unicode::vowels;

/// Vietnamese tone marks (thanh điệu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Sac,
    Huyen,
    Hoi,
    Nga,
    Nang,
}

impl Tone {
    fn index(self) -> usize {
        match self {
            Tone::Sac => 0,
            Tone::Huyen => 1,
            Tone::Hoi => 2,
            Tone::Nga => 3,
            Tone::Nang => 4,
        }
    }

    fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Tone::Sac),
            1 => Some(Tone::Huyen),
            2 => Some(Tone::Hoi),
            3 => Some(Tone::Nga),
            4 => Some(Tone::Nang),
            _ => None,
        }
    }
}

/// Vowel shape modifiers (mũ, móc, trần).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelShape {
    Circumflex,
    Horn,
    Breve,
}

/// Apply a tone mark to a vowel character.
pub fn apply_tone(base: char, tone: Tone) -> Option<char> {
    vowels::toned_vowel(base, tone.index())
}

/// Apply a tone to any vowel, replacing an existing tone if present.
pub fn apply_tone_to_vowel(vowel: char, tone: Tone) -> Option<char> {
    let stem = vowel_stem(vowel)?;
    apply_tone(stem, tone)
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

/// Convert d/D to đ/Đ stroke letter.
pub fn stroke_d(c: char) -> Option<char> {
    match c {
        'd' => Some('đ'),
        'D' => Some('Đ'),
        _ => None,
    }
}

/// Returns true if `c` is a vowel (ASCII or precomposed Vietnamese).
pub fn is_vowel(c: char) -> bool {
    vowels::is_vowel(c)
}

/// Index of the vowel where a tone mark should be placed (modern Vietnamese rules).
#[allow(dead_code)] // Public API — used by engine and external callers.
pub fn main_vowel_index(syllable: &str) -> Option<usize> {
    crate::unicode::tone_position::tone_vowel_index(syllable)
}

/// Index of the last vowel that can *receive* `shape` (for applying 6/7/8).
///
/// Picks the appropriate target rather than the last vowel: e.g. `muoi` + `6`
/// targets `o` (→ `muôi`), not the trailing `i` which cannot take a circumflex.
pub fn shape_target_index(syllable: &str, shape: VowelShape) -> Option<usize> {
    let mut target: Option<usize> = None;
    for (i, ch) in syllable.chars().enumerate() {
        if apply_shape_to_vowel(ch, shape).is_some() {
            target = Some(i);
        }
    }
    target
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

/// Detect tone on a vowel character, if any.
pub(crate) fn tone_on_vowel(c: char) -> Option<Tone> {
    let index = vowels::tone_index_on_vowel(c)?;
    Tone::from_index(index)
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
    SHAPED_VOWELS
        .iter()
        .find(|entry| entry.shaped_lower == lower)
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

/// Strip tone from a vowel, keeping shape (e.g. `á` → `a`, `ấ` → `â`).
pub(crate) fn vowel_stem(c: char) -> Option<char> {
    vowels::vowel_stem(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_tone_basic_vowels() {
        for (base, sac, huyen, hoi, nga, nang) in [
            ('a', 'á', 'à', 'ả', 'ã', 'ạ'),
            ('e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'),
            ('i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'),
            ('o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'),
            ('u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'),
            ('y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'),
        ] {
            assert_eq!(apply_tone(base, Tone::Sac), Some(sac));
            assert_eq!(apply_tone(base, Tone::Huyen), Some(huyen));
            assert_eq!(apply_tone(base, Tone::Hoi), Some(hoi));
            assert_eq!(apply_tone(base, Tone::Nga), Some(nga));
            assert_eq!(apply_tone(base, Tone::Nang), Some(nang));
        }
    }

    #[test]
    fn apply_tone_on_shaped_vowels() {
        assert_eq!(apply_tone('â', Tone::Sac), Some('ấ'));
        assert_eq!(apply_tone('ơ', Tone::Huyen), Some('ờ'));
        assert_eq!(apply_tone('ư', Tone::Nang), Some('ự'));
    }

    #[test]
    fn apply_shape_basic() {
        assert_eq!(apply_shape('a', VowelShape::Circumflex), Some('â'));
        assert_eq!(apply_shape('o', VowelShape::Horn), Some('ơ'));
        assert_eq!(apply_shape('a', VowelShape::Breve), Some('ă'));
    }

    #[test]
    fn apply_shape_to_vowel_strips_tone() {
        assert_eq!(
            apply_shape_to_vowel('á', VowelShape::Breve),
            Some('ă')
        );
        assert_eq!(apply_shape_to_vowel('â', VowelShape::Circumflex), None);
    }

    #[test]
    fn stroke_d_maps_correctly() {
        assert_eq!(stroke_d('d'), Some('đ'));
        assert_eq!(stroke_d('D'), Some('Đ'));
        assert_eq!(stroke_d('x'), None);
    }

    #[test]
    fn main_vowel_index_delegates_to_tone_rules() {
        assert_eq!(main_vowel_index("hoa"), Some(2));
        assert_eq!(main_vowel_index("chao"), Some(2));
        assert_eq!(main_vowel_index("ma"), Some(1));
        assert_eq!(main_vowel_index("ng"), None);
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
