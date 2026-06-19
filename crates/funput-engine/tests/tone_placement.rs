//! End-to-end tone placement: `k`+`y` words and triphthong (kiểu truyền thống).

use funput_engine::Engine;
use funput_core::InputMethod;
fn run(m: InputMethod, keys: &str) -> String {
    let mut e = Engine::new(); e.set_method(m);
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
