//! Tone mark placement — modern Vietnamese reposition rules.

use crate::unicode::marks::{apply_tone_to_vowel, is_vowel, tone_on_vowel, vowel_stem};
use crate::unicode::shapes::{apply_shape, shape_on_vowel, VowelShape};

struct VowelCluster {
    indices: Vec<usize>,
}

/// True if the vowel at `idx` is the onset glide of `qu`/`gi` (the `u`/`i` belongs
/// to the leading consonant, not the tonal nucleus).
fn is_onset_glide(chars: &[char], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }
    let vowel = chars[idx];
    let prev = chars[idx - 1];
    (vowel.eq_ignore_ascii_case(&'u') && prev.eq_ignore_ascii_case(&'q'))
        || (vowel.eq_ignore_ascii_case(&'i') && prev.eq_ignore_ascii_case(&'g'))
}

fn vowel_cluster(buffer: &str) -> Option<VowelCluster> {
    let chars: Vec<char> = buffer.chars().collect();
    let start = chars.iter().position(|c| is_vowel(*c))?;
    let mut indices = Vec::new();

    for (i, ch) in chars.iter().enumerate().skip(start) {
        if is_vowel(*ch) {
            indices.push(i);
        } else {
            break;
        }
    }

    if indices.is_empty() {
        return None;
    }

    // Drop the onset glide of `qu`/`gi` when a real nucleus vowel follows
    // (e.g. `qua` → tone on `a`, `gia` → tone on `a`), but keep it when it is
    // the only vowel (e.g. `gì`, `gìn`).
    if indices.len() >= 2 && is_onset_glide(&chars, indices[0]) {
        indices.remove(0);
    }

    Some(VowelCluster { indices })
}

fn cluster_pattern(buffer: &str, indices: &[usize]) -> String {
    indices
        .iter()
        .map(|&i| vowel_stem(buffer.chars().nth(i).expect("index in bounds")).unwrap())
        .collect()
}

/// Which vowel in the cluster receives the tone (0-based within cluster).
///
/// Only consulted for clusters with no shaped vowel (those are handled by
/// [`tone_vowel_index`]), so every pattern here is plain base letters.
fn tone_offset_in_cluster(pattern: &str, cluster_len: usize) -> usize {
    match pattern {
        // Tone on second vowel (quy tắc mới).
        "oa" | "oe" | "uy" => 1,
        // Tone on first vowel — open diphthongs (ia, ua, ưa) and others.
        "ao" | "ai" | "au" | "ay" | "eu" | "eo" | "ou" | "oi" | "ui" | "iu" | "ia"
        | "ua" | "ưa" => 0,
        _ => cluster_len.saturating_sub(1),
    }
}

/// Char index where a tone mark should be placed.
pub fn tone_vowel_index(buffer: &str) -> Option<usize> {
    let cluster = vowel_cluster(buffer)?;
    let chars: Vec<char> = buffer.chars().collect();

    // A vowel carrying mũ/móc/trần (â ê ô ơ ư ă) takes the tone. For `ươ` (two
    // horned vowels) the tone sits on the second one (`ơ`): trường, được, rượu.
    let last_shaped = cluster
        .indices
        .iter()
        .copied()
        .rfind(|&i| shape_on_vowel(chars[i]).is_some());
    if let Some(i) = last_shaped {
        return Some(i);
    }

    // No shaped vowel: fall back to the base-letter diphthong rules.
    let pattern = cluster_pattern(buffer, &cluster.indices);
    let offset = tone_offset_in_cluster(&pattern, cluster.indices.len());
    Some(cluster.indices[offset.min(cluster.indices.len() - 1)])
}

/// Vowel character used when applying a tone (handles `ie`/`gie` → tonal base `ê`).
///
/// Plain `e` preceded by `i` forms the rising diphthong written with a circumflex
/// (`viết`, `giết`), so the tone lands on `ê`. This covers both the `i` nucleus of
/// `viet` and the `i` that is part of the `gi` onset of `giet`.
pub fn tone_target_vowel(buffer: &str, vowel_idx: usize) -> Option<char> {
    let chars: Vec<char> = buffer.chars().collect();
    let vowel = *chars.get(vowel_idx)?;
    let stem = vowel_stem(vowel)?;
    if !stem.eq_ignore_ascii_case(&'e') {
        return Some(vowel);
    }
    if vowel_idx == 0 || !chars[vowel_idx - 1].eq_ignore_ascii_case(&'i') {
        return Some(vowel);
    }

    // `apply_shape` already preserves case (`e` → `ê`, `E` → `Ê`).
    apply_shape(stem, VowelShape::Circumflex)
}

/// If a tone exists on the wrong vowel, move it to the correct position.
pub fn reposition_existing_tone(buffer: &str) -> Option<String> {
    let desired = tone_vowel_index(buffer)?;

    let mut toned_index: Option<(usize, crate::unicode::marks::Tone)> = None;
    for (i, ch) in buffer.chars().enumerate() {
        if is_vowel(ch) && let Some(tone) = tone_on_vowel(ch) {
            toned_index = Some((i, tone));
            break;
        }
    }

    let (current, tone) = toned_index?;
    if current == desired {
        return None;
    }

    let mut chars: Vec<char> = buffer.chars().collect();
    let old_stem = vowel_stem(chars[current])?;
    chars[current] = old_stem;

    let new_vowel = chars[desired];
    let tone_base = tone_target_vowel(buffer, desired).unwrap_or(new_vowel);
    let new_stem = vowel_stem(tone_base)?;
    let toned = apply_tone_to_vowel(new_stem, tone)?;
    chars[desired] = toned;

    Some(chars.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_at(buffer: &str, index: usize) -> char {
        buffer.chars().nth(index).expect("char at index")
    }

    #[test]
    fn tone_vowel_index_oa_and_ao() {
        assert_eq!(char_at("hoa", tone_vowel_index("hoa").unwrap()), 'a');
        assert_eq!(char_at("chao", tone_vowel_index("chao").unwrap()), 'a');
    }

    #[test]
    fn tone_vowel_index_uy() {
        assert_eq!(char_at("thuy", tone_vowel_index("thuy").unwrap()), 'y');
    }

    #[test]
    fn tone_vowel_index_single_vowel() {
        assert_eq!(char_at("ma", tone_vowel_index("ma").unwrap()), 'a');
        assert_eq!(char_at("ho", tone_vowel_index("ho").unwrap()), 'o');
    }

    #[test]
    fn tone_vowel_index_uo_horn_cluster() {
        let buffer = "trương";
        assert_eq!(char_at(buffer, tone_vowel_index(buffer).unwrap()), 'ơ');
    }

    #[test]
    fn tone_vowel_index_open_diphthongs_ia_ua() {
        assert_eq!(char_at("mia", tone_vowel_index("mia").unwrap()), 'i');
        assert_eq!(char_at("mua", tone_vowel_index("mua").unwrap()), 'u');
        assert_eq!(char_at("cua", tone_vowel_index("cua").unwrap()), 'u');
        assert_eq!(char_at("lua", tone_vowel_index("lua").unwrap()), 'u');
    }

    #[test]
    fn tone_vowel_index_uoi_cluster() {
        assert_eq!(char_at("ngươi", tone_vowel_index("ngươi").unwrap()), 'ơ');
    }

    #[test]
    fn tone_target_vowel_ie_uses_circumflex_e() {
        assert_eq!(tone_target_vowel("viet", 2), Some('ê'));
        assert_eq!(tone_target_vowel("lien", 2), Some('ê'));
    }

    #[test]
    fn reposition_moves_tone_from_o_to_a() {
        let misplaced = "hòa";
        let result = reposition_existing_tone(misplaced);
        assert_eq!(result.as_deref(), Some("hoà"));
    }
}
