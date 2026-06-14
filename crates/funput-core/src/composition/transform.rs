//! Transform pipeline for VNI input.

use crate::composition::replace_char_at;
use crate::composition::revert::{try_revert_shape, try_revert_stroke, try_revert_tone};
use crate::input_method::vni::{classify_key, VniKeyAction};
use crate::unicode::marks::{
    apply_shape, apply_shape_to_vowel, apply_tone_to_vowel, is_vowel, shape_target_index, stroke_d,
    vowel_stem, VowelShape,
};
use crate::unicode::tone_position::{reposition_existing_tone, tone_target_vowel, tone_vowel_index};
use crate::validation::syllable::{ModifierValidation, validate_shape, validate_stroke, validate_tone};
use crate::{TransformKind, TransformResult};

fn validation_gate(buffer: &str, key: char, validation: ModifierValidation) -> Option<TransformResult> {
    match validation {
        ModifierValidation::Allow => None,
        ModifierValidation::Ignored => Some(TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        }),
        ModifierValidation::PassThrough => Some(TransformResult {
            kind: TransformKind::Pending,
            text: format!("{buffer}{key}"),
        }),
    }
}

/// Apply one VNI keystroke to `buffer`.
pub(crate) fn apply_vni(buffer: &str, key: char) -> TransformResult {
    apply_action(buffer, key, classify_key(key))
}

/// Apply a classified key action to `buffer`.
///
/// Method-agnostic: VNI and (later) Telex differ only in how a key maps to a
/// [`VniKeyAction`] — the revert → validate → apply orchestration here is shared.
/// `key` is the literal character to append for [`VniKeyAction::Normal`] and
/// pass-through.
pub(crate) fn apply_action(buffer: &str, key: char, action: VniKeyAction) -> TransformResult {
    match action {
        VniKeyAction::Stroke => {
            if let Some(text) = try_revert_stroke(buffer) {
                return TransformResult {
                    kind: TransformKind::Reverted,
                    text,
                };
            }
            if let Some(result) = validation_gate(buffer, key, validate_stroke(buffer)) {
                return result;
            }
            apply_stroke(buffer)
        }
        VniKeyAction::Tone(tone) => {
            if let Some(text) = try_revert_tone(buffer, tone) {
                return TransformResult {
                    kind: TransformKind::Reverted,
                    text,
                };
            }
            if let Some(result) = validation_gate(buffer, key, validate_tone(buffer)) {
                return result;
            }
            apply_tone_key(buffer, tone)
        }
        VniKeyAction::Shape(shape) => {
            // Apply takes priority when an unshaped target exists, so that the
            // second horn in `u7o7` shapes the `o` (→ `ươ`) instead of reverting
            // the earlier `ư`. Revert only fires when the key has no target to
            // apply to (e.g. `a66`, `uo77`).
            if shape_apply_target_exists(buffer, shape) {
                if let Some(result) = validation_gate(buffer, key, validate_shape(buffer)) {
                    return result;
                }
                return apply_shape_key(buffer, shape);
            }
            if let Some(text) = try_revert_shape(buffer, shape) {
                return TransformResult {
                    kind: TransformKind::Reverted,
                    text,
                };
            }
            if let Some(result) = validation_gate(buffer, key, validate_shape(buffer)) {
                return result;
            }
            apply_shape_key(buffer, shape)
        }
        VniKeyAction::Normal => {
            let text = format!("{buffer}{key}");
            if let Some(repositioned) = reposition_existing_tone(&text) {
                TransformResult {
                    kind: TransformKind::Applied,
                    text: repositioned,
                }
            } else {
                TransformResult {
                    kind: TransformKind::Pending,
                    text,
                }
            }
        }
    }
}

fn apply_stroke(buffer: &str) -> TransformResult {
    let mut chars: Vec<char> = buffer.chars().collect();
    let Some(last) = chars.last().copied() else {
        return TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        };
    };

    let Some(stroked) = stroke_d(last) else {
        return TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        };
    };

    let len = chars.len();
    chars[len - 1] = stroked;
    TransformResult {
        kind: TransformKind::Applied,
        text: chars.into_iter().collect(),
    }
}

fn apply_tone_key(buffer: &str, tone: crate::unicode::marks::Tone) -> TransformResult {
    let Some(vowel_idx) = tone_vowel_index(buffer) else {
        return TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        };
    };

    let vowel = buffer.chars().nth(vowel_idx).expect("vowel index in bounds");
    let tone_target = tone_target_vowel(buffer, vowel_idx).unwrap_or(vowel);
    let Some(toned) = apply_tone_to_vowel(tone_target, tone) else {
        return TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        };
    };

    TransformResult {
        kind: TransformKind::Applied,
        text: replace_char_at(buffer, vowel_idx, toned),
    }
}

fn uo_pair_in_vowel_cluster(buffer: &str) -> Option<(usize, usize)> {
    let chars: Vec<(usize, char)> = buffer.chars().enumerate().collect();
    let start = chars.iter().position(|(_, c)| is_vowel(*c))?;
    let mut indices = Vec::new();

    for (i, ch) in &chars[start..] {
        if is_vowel(*ch) {
            indices.push(*i);
        } else {
            break;
        }
    }

    for pair in indices.windows(2) {
        let u_idx = pair[0];
        let o_idx = pair[1];
        let u = buffer.chars().nth(u_idx)?;
        let o = buffer.chars().nth(o_idx)?;
        if !u.eq_ignore_ascii_case(&'u') || !o.eq_ignore_ascii_case(&'o') {
            continue;
        }
        let u_stem = vowel_stem(u)?;
        let o_stem = vowel_stem(o)?;
        if u_stem.eq_ignore_ascii_case(&'u') && o_stem.eq_ignore_ascii_case(&'o') {
            return Some((u_idx, o_idx));
        }
    }
    None
}

fn apply_uo_compound(buffer: &str) -> Option<String> {
    let (u_idx, o_idx) = uo_pair_in_vowel_cluster(buffer)?;

    let mut chars: Vec<char> = buffer.chars().collect();
    let u = chars[u_idx];
    let o = chars[o_idx];
    let shaped_u = apply_shape(u, VowelShape::Horn)?;
    let shaped_o = apply_shape(o, VowelShape::Horn)?;
    chars[u_idx] = shaped_u;
    chars[o_idx] = shaped_o;
    Some(chars.into_iter().collect())
}

/// True if `shape` can still be applied to some vowel in `buffer` (the horn
/// `uo` compound, or a single vowel that can receive the shape).
fn shape_apply_target_exists(buffer: &str, shape: VowelShape) -> bool {
    if shape == VowelShape::Horn && uo_pair_in_vowel_cluster(buffer).is_some() {
        return true;
    }
    shape_target_index(buffer, shape).is_some()
}

fn apply_shape_key(buffer: &str, shape: VowelShape) -> TransformResult {
    if shape == VowelShape::Horn && let Some(text) = apply_uo_compound(buffer) {
        return TransformResult {
            kind: TransformKind::Applied,
            text,
        };
    }

    let Some(vowel_idx) = shape_target_index(buffer, shape) else {
        return TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        };
    };

    let vowel = buffer.chars().nth(vowel_idx).expect("vowel index in bounds");
    let Some(shaped) = apply_shape_to_vowel(vowel, shape) else {
        return TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        };
    };

    TransformResult {
        kind: TransformKind::Applied,
        text: replace_char_at(buffer, vowel_idx, shaped),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply, InputMethod, TransformKind};

    #[test]
    fn stroke_d9() {
        assert_eq!(
            apply_vni("d", '9'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "đ".into(),
            }
        );
    }

    #[test]
    fn stroke_uppercase_d9() {
        assert_eq!(
            apply_vni("D", '9'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "Đ".into(),
            }
        );
    }

    #[test]
    fn tone_on_single_vowel() {
        for (key, expected) in [('1', "á"), ('2', "à"), ('3', "ả"), ('4', "ã"), ('5', "ạ")] {
            assert_eq!(
                apply_vni("a", key),
                TransformResult {
                    kind: TransformKind::Applied,
                    text: expected.into(),
                },
                "key {key}"
            );
        }
    }

    #[test]
    fn reposition_tone_cases() {
        assert_eq!(type_keys("hoa2"), "hoà");
        assert_eq!(type_keys("chao2"), "chào");
        assert_eq!(type_keys("thuy3"), "thuỷ");
        assert_eq!(type_keys("khoe3"), "khoẻ");
        assert_eq!(type_keys("hoaf2"), "hoàf");
        assert_eq!(type_keys("tru7o7n2g"), "trường");
    }

    fn type_keys(keys: &str) -> String {
        let mut buf = String::new();
        for key in keys.chars() {
            buf = apply_vni(&buf, key).text;
        }
        buf
    }

    #[test]
    fn shape_on_single_vowel() {
        assert_eq!(
            apply_vni("a", '6'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "â".into(),
            }
        );
        assert_eq!(
            apply_vni("e", '6'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "ê".into(),
            }
        );
        assert_eq!(
            apply_vni("o", '6'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "ô".into(),
            }
        );
        assert_eq!(
            apply_vni("o", '7'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "ơ".into(),
            }
        );
        assert_eq!(
            apply_vni("u", '7'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "ư".into(),
            }
        );
        assert_eq!(
            apply_vni("a", '8'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "ă".into(),
            }
        );
    }

    #[test]
    fn shape_then_tone() {
        let mut buf = String::new();
        for key in "o71".chars() {
            buf = apply_vni(&buf, key).text;
        }
        assert_eq!(buf, "ớ");

        let mut buf = String::new();
        for key in "a61".chars() {
            buf = apply_vni(&buf, key).text;
        }
        assert_eq!(buf, "ấ");
    }

    #[test]
    fn uo_compound_horn() {
        assert_eq!(
            apply_vni("uo", '7'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "ươ".into(),
            }
        );
    }

    #[test]
    fn shape_on_empty_or_consonant_ignored() {
        assert_eq!(
            apply_vni("", '6'),
            TransformResult {
                kind: TransformKind::Ignored,
                text: String::new(),
            }
        );
        assert_eq!(
            apply_vni("ng", '7'),
            TransformResult {
                kind: TransformKind::Ignored,
                text: "ng".into(),
            }
        );
    }

    #[test]
    fn normal_key_appends() {
        assert_eq!(
            apply_vni("a", 'b'),
            TransformResult {
                kind: TransformKind::Pending,
                text: "ab".into(),
            }
        );
    }

    #[test]
    fn tone_on_empty_buffer_ignored() {
        assert_eq!(
            apply_vni("", '1'),
            TransformResult {
                kind: TransformKind::Ignored,
                text: String::new(),
            }
        );
    }

    #[test]
    fn multi_char_syllable_via_apply() {
        let mut buf = String::new();
        for key in "ma1".chars() {
            let result = apply(&buf, key, InputMethod::Vni);
            buf = result.text;
        }
        assert_eq!(buf, "má");
    }

    #[test]
    fn revert_cases() {
        assert_eq!(
            apply_vni("á", '1'),
            TransformResult {
                kind: TransformKind::Reverted,
                text: "a".into(),
            }
        );
        assert_eq!(
            apply_vni("â", '6'),
            TransformResult {
                kind: TransformKind::Reverted,
                text: "a".into(),
            }
        );
        assert_eq!(
            apply_vni("đ", '9'),
            TransformResult {
                kind: TransformKind::Reverted,
                text: "d".into(),
            }
        );
        assert_eq!(
            apply_vni("ấ", '1'),
            TransformResult {
                kind: TransformKind::Reverted,
                text: "â".into(),
            }
        );
        assert_eq!(
            apply_vni("à", '2'),
            TransformResult {
                kind: TransformKind::Reverted,
                text: "a".into(),
            }
        );
        assert_eq!(
            apply_vni("a", '2'),
            TransformResult {
                kind: TransformKind::Applied,
                text: "à".into(),
            }
        );
        assert_eq!(type_keys("a12"), "à");
        assert_eq!(type_keys("a11"), "a");
        assert_eq!(type_keys("a66"), "a");
        assert_eq!(type_keys("d99"), "d");
        assert_eq!(type_keys("hoa22"), "hoa");
        assert_eq!(type_keys("uo77"), "uo");
        assert_eq!(type_keys("tru7o7n2g2"), "trương");
    }

    #[test]
    fn complex_syllable_sequences() {
        assert_eq!(type_keys("vie5t"), "việt");
        assert_eq!(type_keys("truo7ng"), "trương");
        assert_eq!(type_keys("ngu7o7i2"), "người");
        assert_eq!(type_keys("phuo7ng"), "phương");
    }
}
