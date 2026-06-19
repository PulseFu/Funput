//! The đ stroke targets the syllable's leading `d`, so the key may be typed after
//! the rest of the word — `dang9` → `đang` (VNI), not only `d9ang`.

mod support;

use funput_core::InputMethod;

fn vni(keys: &str) -> String {
    support::type_keys_buffer(InputMethod::Vni, keys)
}
fn telex(keys: &str) -> String {
    support::type_keys_buffer(InputMethod::Telex, keys)
}

#[test]
fn vni_stroke_reaches_back_to_leading_d() {
    assert_eq!(vni("dang9"), "đang"); // 9 typed after the whole syllable
    assert_eq!(vni("d9ang"), "đang"); // 9 right after d still works
    assert_eq!(vni("dong9"), "đong");
    // Stroke last, after shape (7) and tone (1) are already on the word.
    assert_eq!(vni("dung719"), "đứng");
}

#[test]
fn vni_double_stroke_reverts_leading() {
    // Pressing 9 again on a stroked syllable reverts the đ (restore-raw semantics).
    assert_eq!(vni("dang99"), "dang9");
}

#[test]
fn telex_dd_digraph_unaffected() {
    assert_eq!(telex("ddang"), "đang"); // consecutive dd still strokes
    assert_eq!(telex("dd"), "đ");
}

#[test]
fn stroke_in_abbreviation_targets_the_d() {
    // Abbreviations: the đ is the onset of a later "syllable" in the run, i.e. the
    // last d — GĐ (Giám đốc), QĐ (Quyết định).
    assert_eq!(vni("GD9"), "GĐ");
    assert_eq!(vni("QD9"), "QĐ");
    assert_eq!(telex("GDD"), "GĐ");
}
