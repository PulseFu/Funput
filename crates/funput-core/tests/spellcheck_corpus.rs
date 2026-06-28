//! Corpus-backed checks for spell-check ("Kiểm tra chính tả") completeness &
//! tightness.
//!
//! The valid-rhyme inventory in `validation::rhyme` was cross-checked against an
//! attested Vietnamese word corpus (Viet74K). These tests lock in both directions:
//! (a) **recall** — real syllables, including established loanwords and onomatopoeia,
//! are never rejected (else spell-check would wrongly block a real word); and
//! (b) **precision** — structurally-plausible non-syllables are rejected.

use funput_core::{apply_checked, is_complete_syllable, InputMethod, ToneStyle, TransformKind};

/// Real Vietnamese syllables (incl. established loanwords / onomatopoeia). Each MUST
/// be accepted as a complete syllable — otherwise spell-check would block a real word.
const MUST_ACCEPT: &[&str] = &[
    // Everyday vocabulary across the rhyme space.
    "ăn", "uống", "người", "nghiêng", "được", "trường", "khỏe", "thúy", "quỳnh", "khuỷu", "hươu",
    "rượu", "khuya", "nghệ", "ngoại", "huỳnh", "quặng", "thoăn", "choắt",
    // The loanword / onomatopoeia rhymes added in Mức B (each previously blocked).
    "buýt", "huýt", "suýt", "quýt", "quỵt", "giêng", "giếng", "giềng", "yểng", "tuýp", "tuyn",
    "luyn", "xoong", "boong", "soóc", "moóc", "oăm", "khoằm", "huých", "uỵch", "khuều", "ngoao",
    "ngoém", "héc", "véc",
];

/// Toned forms that are NOT valid Vietnamese but are structurally close enough to
/// slip past a naive check. Each MUST be rejected.
const MUST_REJECT: &[&str] = &[
    "tèt", // stop coda + huyền (a stop coda allows only sắc / nặng)
    "bèc", // stop coda + huyền
    "tẽt", // stop coda + ngã ("text")
    "eg", "id", "oab", "onk", "erf", "az", "ngb", // non-Vietnamese rhyme / structure
];

#[test]
fn corpus_real_syllables_accepted() {
    for &s in MUST_ACCEPT {
        assert!(is_complete_syllable(s), "real syllable wrongly rejected: {s}");
    }
}

#[test]
fn corpus_nonsyllables_rejected() {
    for &s in MUST_REJECT {
        assert!(!is_complete_syllable(s), "non-syllable wrongly accepted: {s}");
    }
}

/// Type a Telex sequence with the spell-check gate on.
fn type_checked(keys: &str) -> String {
    let mut buf = String::new();
    for k in keys.chars() {
        buf = apply_checked(&buf, k, InputMethod::Telex, ToneStyle::Traditional, true).text;
    }
    buf
}

/// End-to-end recall: typing a real word with spell-check ON must still compose it —
/// proof that the Mức B tightening never blocks a real word via the gate. (Sequences
/// chosen to avoid the `oo`→`ô` Telex digraph; those rhymes are covered above.)
#[test]
fn spell_check_keeps_real_words() {
    for (keys, expected) in [
        ("quyts", "quýt"),
        ("buyts", "buýt"),
        ("suyts", "suýt"),
        ("huychs", "huých"),
        ("gieengs", "giếng"),
        ("nghieeng", "nghiêng"),
    ] {
        assert_eq!(type_checked(keys), expected, "blocked real word: {keys}");
    }
}

/// The gate still rejects a diacritic that would form a non-syllable, passing the
/// modifier key through as a literal.
#[test]
fn spell_check_blocks_nonsyllable_via_gate() {
    let r = apply_checked("bec", 'f', InputMethod::Telex, ToneStyle::Traditional, true);
    assert_eq!(r.text, "becf");
    assert_eq!(r.kind, TransformKind::Pending);
}
