//! Vietnamese rhyme (vần) inventory — the toneless nucleus+coda part of a
//! syllable. A composed syllable "exists in Vietnamese" only if its rhyme is in
//! this set (plus a valid onset and tone). This is what lets the engine keep
//! real syllables and revert the rest, without a full word corpus.
//!
//! Generous on purpose: when unsure, a rhyme is **included** so real Vietnamese
//! is never wrongly reverted. Report any real word whose rhyme is missing — it is
//! a one-line addition here.
//!
//! Completeness is cross-checked against an attested Vietnamese word corpus by
//! `tests/spellcheck_corpus.rs`; rhymes for established loanwords and onomatopoeia
//! (`buýt`, `xoong`, `soóc`, `tuýp`, `giếng`/`giêng`, `oăm`, `huých`…) are included
//! so spell-check ("Kiểm tra chính tả") never blocks a real word.

/// Valid toneless rhymes (lowercase, shaped vowels: `ươ`, `iê`, …).
const VALID_RHYMES: &[&str] = &[
    // Open (no coda)
    "a", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y", "ia", "ya", "ai", "ao", "au", "ay", "âu",
    "ây", "eo", "êu", "iu", "oa", "oe", "oi", "ôi", "ơi", "ua", "ui", "uê", "uy", "uơ", "ưa", "ưi",
    "ưu", "oai", "oay", "oeo", "uôi", "uây", "uya", "uyu", "ươi", "ươu", "iêu", "yêu", "oao", "uêu",
    // -m
    "am", "ăm", "âm", "em", "êm", "im", "om", "ôm", "ơm", "um", "iêm", "yêm", "uôm", "ươm", "oam",
    "oăm", "oem",
    // -p
    "ap", "ăp", "âp", "ep", "êp", "ip", "op", "ôp", "ơp", "up", "iêp", "ươp", "oap", "uyp",
    // -n
    "an", "ăn", "ân", "en", "ên", "in", "on", "ôn", "ơn", "un", "ưn", "iên", "yên", "uôn", "ươn",
    "oan", "oăn", "oen", "uân", "uyên", "uyn",
    // -t
    "at", "ăt", "ât", "et", "êt", "it", "ot", "ôt", "ơt", "ut", "ưt", "iêt", "yêt", "uôt", "ươt",
    "oat", "oăt", "oet", "uât", "uyêt", "yt", "uyt",
    // -ng
    "ang", "ăng", "âng", "eng", "ong", "ông", "ung", "ưng", "iêng", "uông", "ương", "oang", "oăng",
    "uâng", "oong", "êng", "yêng",
    // -c
    "ac", "ăc", "âc", "oc", "ôc", "uc", "ưc", "iêc", "uôc", "ươc", "oac", "oăc", "ec", "ooc",
    // -nh
    "anh", "ênh", "inh", "ynh", "uynh", "uênh", "oanh",
    // -ch
    "ach", "êch", "ich", "uêch", "oach", "uych",
];

/// True if `rhyme` (toneless, lowercase) is a valid Vietnamese rhyme.
pub fn is_valid_rhyme(rhyme: &str) -> bool {
    VALID_RHYMES.contains(&rhyme)
}

/// The full toneless rhyme inventory (for prefix/reachability checks).
pub fn all() -> &'static [&'static str] {
    VALID_RHYMES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_rhymes_present() {
        for ok in [
            "a", "ương", "iêt", "uyên", "uyêt", "anh", "ach", "uêch", "uyu", "uya", "ưng", "oăn",
            "uâng", "ynh",
            // Loanword / onomatopoeia rhymes added for spell-check completeness.
            "uyt", "yt", "oong", "ooc", "ec", "êng", "yêng", "oăm", "uych", "uyp", "uyn", "uêu",
        ] {
            assert!(is_valid_rhyme(ok), "{ok} should be a valid rhyme");
        }
    }

    #[test]
    fn nonexistent_rhymes_absent() {
        for bad in ["eg", "id", "ub", "az", "onk", "erf"] {
            assert!(!is_valid_rhyme(bad), "{bad} should not be a rhyme");
        }
    }
}
