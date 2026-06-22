//! End-to-end tone placement: `k`+`y` words and triphthong (kiểu truyền thống).

use funput_engine::Engine;
use funput_core::{InputMethod, ToneStyle};
fn run(m: InputMethod, keys: &str) -> String {
    let mut e = Engine::new(); e.set_method(m);
    for k in keys.chars() { e.process_char(k); }
    e.buffer().to_string()
}
fn run_modern(m: InputMethod, keys: &str) -> String {
    let mut e = Engine::new(); e.set_method(m); e.set_tone_style(ToneStyle::Modern);
    for k in keys.chars() { e.process_char(k); }
    e.buffer().to_string()
}
#[test]
fn traditional_tone_placement_and_ky_words() {
    assert_eq!(run(InputMethod::Vni, "ky2"), "kỳ");
    assert_eq!(run(InputMethod::Telex, "kyf"), "kỳ");
    assert_eq!(run(InputMethod::Telex, "ngoaif"), "ngoài");
    assert_eq!(run(InputMethod::Vni, "ngoai2"), "ngoài");
    assert_eq!(run(InputMethod::Telex, "kyx"), "kỹ"); // kỹ
}

#[test]
fn modern_tone_style_oa_oe_uy() {
    // "Kiểu mới": tone on the second vowel of open oa/oe/uy, independent of where
    // the tone key is typed.
    assert_eq!(run_modern(InputMethod::Telex, "hoaf"), "hoà");
    assert_eq!(run_modern(InputMethod::Telex, "hofa"), "hoà"); // tone before the `a`
    assert_eq!(run_modern(InputMethod::Telex, "thuyr"), "thuỷ");
    assert_eq!(run_modern(InputMethod::Telex, "khoer"), "khoẻ");
    assert_eq!(run_modern(InputMethod::Vni, "hoa2"), "hoà");
    assert_eq!(run_modern(InputMethod::Vni, "thuy3"), "thuỷ");
    // Unchanged between styles: ia/ua, coda, triphthong, shaped vowel.
    assert_eq!(run_modern(InputMethod::Telex, "muaf"), "mùa");
    assert_eq!(run_modern(InputMethod::Telex, "hoanf"), "hoàn");
    assert_eq!(run_modern(InputMethod::Telex, "ngoaif"), "ngoài");
    assert_eq!(run_modern(InputMethod::Vni, "tru7o7n2g"), "trường");
}

#[test]
fn traditional_keeps_oa_oe_uy_on_first_vowel() {
    // Same words in the default style stay on the first vowel.
    assert_eq!(run(InputMethod::Telex, "hoaf"), "hòa");
    assert_eq!(run(InputMethod::Telex, "thuyr"), "thủy");
    assert_eq!(run(InputMethod::Telex, "khoer"), "khỏe");
}

#[test]
fn uu_diphthong_horns_first_vowel() {
    // The `ưu` falling diphthong horns the first `u`, so the tone lands on `ư`
    // regardless of whether the horn key comes before or after the second `u`.
    assert_eq!(run(InputMethod::Telex, "cuuwf"), "cừu");
    assert_eq!(run(InputMethod::Telex, "truuwf"), "trừu");
    assert_eq!(run(InputMethod::Telex, "cuwuf"), "cừu"); // horn typed mid-cluster
    assert_eq!(run(InputMethod::Vni, "cuu72"), "cừu");
    assert_eq!(run(InputMethod::Vni, "tru7u2"), "trừu");
    assert_eq!(run(InputMethod::Telex, "cuuws"), "cứu"); // sắc on ư too
}
