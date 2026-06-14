//! Single source of truth for Vietnamese vowel glyphs (base + tone variants).

/// One vowel family: base (no tone) + five tone marks (sắc, huyền, hỏi, ngã, nặng).
struct VowelFamily {
    base: char,
    tones: [char; 5],
}

const VOWEL_FAMILIES: &[VowelFamily] = &[
    VowelFamily {
        base: 'a',
        tones: ['á', 'à', 'ả', 'ã', 'ạ'],
    },
    VowelFamily {
        base: 'ă',
        tones: ['ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
    },
    VowelFamily {
        base: 'â',
        tones: ['ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
    },
    VowelFamily {
        base: 'e',
        tones: ['é', 'è', 'ẻ', 'ẽ', 'ẹ'],
    },
    VowelFamily {
        base: 'ê',
        tones: ['ế', 'ề', 'ể', 'ễ', 'ệ'],
    },
    VowelFamily {
        base: 'i',
        tones: ['í', 'ì', 'ỉ', 'ĩ', 'ị'],
    },
    VowelFamily {
        base: 'o',
        tones: ['ó', 'ò', 'ỏ', 'õ', 'ọ'],
    },
    VowelFamily {
        base: 'ô',
        tones: ['ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
    },
    VowelFamily {
        base: 'ơ',
        tones: ['ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
    },
    VowelFamily {
        base: 'u',
        tones: ['ú', 'ù', 'ủ', 'ũ', 'ụ'],
    },
    VowelFamily {
        base: 'ư',
        tones: ['ứ', 'ừ', 'ử', 'ữ', 'ự'],
    },
    VowelFamily {
        base: 'y',
        tones: ['ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
    },
];

fn fold_lower(c: char) -> char {
    char::to_lowercase(c).next().unwrap_or(c)
}

fn with_case(template: char, reference: char) -> char {
    if reference.is_uppercase() {
        char::to_uppercase(template).next().unwrap_or(template)
    } else {
        template
    }
}

fn family_of(c: char) -> Option<&'static VowelFamily> {
    let lower = fold_lower(c);
    VOWEL_FAMILIES
        .iter()
        .find(|family| family.base == lower || family.tones.contains(&lower))
}

/// Returns true if `c` is a vowel (ASCII or precomposed Vietnamese).
pub fn is_vowel(c: char) -> bool {
    family_of(c).is_some()
}

/// Strip tone from a vowel, keeping shape (e.g. `á` → `a`, `ấ` → `â`).
pub fn vowel_stem(c: char) -> Option<char> {
    let family = family_of(c)?;
    Some(with_case(family.base, c))
}

/// Apply tone by index: 0 = sắc … 4 = nặng.
pub fn toned_vowel(base: char, tone_index: usize) -> Option<char> {
    let lower = fold_lower(base);
    let family = VOWEL_FAMILIES.iter().find(|f| f.base == lower)?;
    let toned = *family.tones.get(tone_index)?;
    Some(with_case(toned, base))
}

/// Tone index on a vowel, if any (0 = sắc … 4 = nặng).
pub fn tone_index_on_vowel(c: char) -> Option<usize> {
    let lower = fold_lower(c);
    let family = family_of(c)?;
    if lower == family.base {
        return None;
    }
    family.tones.iter().position(|&t| t == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_covers_all_stems_and_tones() {
        for family in VOWEL_FAMILIES {
            assert!(is_vowel(family.base));
            for &toned in &family.tones {
                assert!(is_vowel(toned));
                assert_eq!(vowel_stem(toned), Some(family.base));
            }
            for (i, &toned) in family.tones.iter().enumerate() {
                assert_eq!(toned_vowel(family.base, i), Some(toned));
                assert_eq!(tone_index_on_vowel(toned), Some(i));
            }
        }
    }

    #[test]
    fn uppercase_round_trip() {
        assert_eq!(toned_vowel('A', 0), Some('Á'));
        assert_eq!(vowel_stem('Ắ'), Some('Ă'));
        assert_eq!(tone_index_on_vowel('Ứ'), Some(0));
    }
}
