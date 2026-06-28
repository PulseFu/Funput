//! Split a syllable chunk into onset + vowel nucleus + coda.

use crate::unicode::marks::is_vowel;

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

/// Consonant-cluster onsets that occur only in Central Highlands (Tây Nguyên)
/// toponyms borrowed from Ê Đê / Jarai / Bahnar / M'Nông (`Pleiku`, `Krông`,
/// `Glong`, `Blơr`, `Drăng`). Not native Vietnamese onsets — kept separate so the
/// inventory above stays "pure Vietnamese". A valid rhyme is still required after
/// the onset, so this barely affects English auto-restore.
const ETHNIC_ONSETS: &[&str] = &["bl", "br", "dr", "gl", "gr", "kl", "kr", "pl", "pr"];

/// True if `onset_lower` is a valid Vietnamese onset (`đ` included), or a Tây
/// Nguyên toponym cluster ([`ETHNIC_ONSETS`]).
pub(crate) fn is_valid_onset(onset_lower: &str) -> bool {
    onset_lower.is_empty()
        || onset_lower == "đ"
        || VALID_ONSETS.contains(&onset_lower)
        || ETHNIC_ONSETS.contains(&onset_lower)
}

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
    let mut nucleus = String::new();
    let mut coda = String::new();
    for ch in rest.chars() {
        if is_vowel(ch) {
            nucleus.push(ch);
        } else {
            coda.push(ch);
        }
    }

    SyllableParts {
        onset,
        nucleus,
        coda,
        invalid_onset,
    }
}

fn match_onset(buffer: &str) -> (String, &str, bool) {
    if let Some(first) = buffer.chars().next()
        && (first == 'đ' || first == 'Đ')
    {
        let rest = buffer.strip_prefix(first).unwrap_or("");
        return (first.to_string(), rest, false);
    }

    let lower_chars: Vec<char> = buffer.to_lowercase().chars().collect();

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

    if let Some(first) = buffer.chars().next()
        && is_vowel(first)
    {
        return (String::new(), buffer, false);
    }

    (String::new(), buffer, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(onset: &str, nucleus: &str, coda: &str) -> SyllableParts {
        SyllableParts {
            onset: onset.into(),
            nucleus: nucleus.into(),
            coda: coda.into(),
            invalid_onset: false,
        }
    }

    #[test]
    fn parse_syllable_cases() {
        assert_eq!(parse_syllable("tr"), parts("tr", "", ""));
        assert_eq!(parse_syllable("ng"), parts("ng", "", ""));
        assert_eq!(parse_syllable("ma"), parts("m", "a", ""));
        assert_eq!(parse_syllable("text"), parts("t", "e", "xt"));
        assert_eq!(parse_syllable("mix"), parts("m", "i", "x"));
        assert_eq!(parse_syllable("trung"), parts("tr", "u", "ng"));
        assert_eq!(parse_syllable("đ"), parts("đ", "", ""));
        // `gi` releases its `i` as the nucleus when no vowel follows.
        assert_eq!(parse_syllable("gi"), parts("g", "i", ""));
        assert_eq!(parse_syllable("gia"), parts("gi", "a", ""));
    }
}
