//! Pure Vietnamese input transform — one keystroke at a time.
//!
//! `funput-core` answers: given the current syllable buffer and a key, what is the
//! new text according to VNI or Telex?
//!
//! # API FROZEN (Phase 8)
//!
//! The public surface is intentionally minimal for `funput-engine`:
//! [`InputMethod`], [`TransformKind`], [`TransformResult`], [`apply`], and the
//! syllable-structure checks [`is_valid`] (lenient) / [`is_complete_syllable`]
//! (strict, for word boundaries).
//! Breaking changes require semver coordination with the engine.
//!
//! # Contract
//!
//! - **Stateless:** no session, no backspace count — the engine diffs `buffer` vs
//!   `result.text`.
//! - **Syllable chunk:** the engine passes one syllable buffer per word; core does not
//!   split words on spaces.
//! - **No I/O:** no platform hooks, config files, or English auto-restore (engine).

mod composition;
mod input_method;
mod unicode;
mod validation;

/// Input method selector — the engine passes this on each [`apply`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    /// VNI digit modifiers (`1`–`9`).
    Vni,
    /// Telex letter modifiers (`s`/`f`/`r`/`x`/`j`, `aa`/`dd`/`w`, …).
    Telex,
}

/// Tone-mark placement style — where the tone lands on an open glide-initial
/// diphthong (`oa`, `oe`, `uy`). The only syllables on which the two styles
/// disagree; everything else is identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToneStyle {
    /// "Kiểu cũ": tone on the first vowel — `hòa`, `khỏe`, `thúy`.
    #[default]
    Traditional,
    /// "Kiểu mới": tone on the main (second) vowel — `hoà`, `khoẻ`, `thuý`.
    Modern,
}

/// Result kind for a single keystroke transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformKind {
    /// Normal key appended; no Vietnamese transform yet (or pass-through modifier on
    /// non-Vietnamese text — e.g. `text` + `1` → `"text1"`).
    Pending,
    /// Tone, shape, stroke, or reposition produced new composed text in `text`.
    Applied,
    /// Double modifier restores the raw keystrokes — strips the diacritic and
    /// appends the literal key (e.g. `a11` → `a1`, Telex `ass` → `as`).
    Reverted,
    /// Modifier rejected — `text` unchanged (e.g. `ng` + `1`, stroke on non-`d`).
    Ignored,
}

/// Output of applying one keystroke to the current buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformResult {
    /// How the engine should interpret this step.
    pub kind: TransformKind,
    /// Buffer after this keystroke (compare with the previous buffer for backspace).
    pub text: String,
}

/// Apply one keystroke to the current syllable buffer.
///
/// # Examples
///
/// ```
/// use funput_core::{apply, InputMethod, ToneStyle, TransformKind, TransformResult};
///
/// let r = apply("a", '1', InputMethod::Vni, ToneStyle::Traditional);
/// assert_eq!(
///     r,
///     TransformResult {
///         kind: TransformKind::Applied,
///         text: "á".into(),
///     }
/// );
/// ```
pub use validation::syllable::{is_complete_syllable, is_definitely_invalid, is_valid};

pub fn apply(buffer: &str, key: char, method: InputMethod, tone_style: ToneStyle) -> TransformResult {
    apply_checked(buffer, key, method, tone_style, false)
}

/// Like [`apply`], but with the **spell-check** ("Kiểm tra chính tả") gate.
///
/// When `spell_check` is true, a tone / shape / stroke is only placed if the
/// result can still become a real Vietnamese syllable; otherwise the modifier key
/// is passed through as a literal character (UniKey-style strict diacritics).
/// `spell_check = false` reproduces [`apply`] exactly.
pub fn apply_checked(
    buffer: &str,
    key: char,
    method: InputMethod,
    tone_style: ToneStyle,
    spell_check: bool,
) -> TransformResult {
    match method {
        InputMethod::Vni => composition::transform::apply_vni(buffer, key, tone_style, spell_check),
        InputMethod::Telex => {
            composition::transform::apply_telex(buffer, key, tone_style, spell_check)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_vni_appends_normal_key() {
        let result = apply("", 'a', InputMethod::Vni, ToneStyle::Traditional);
        assert_eq!(
            result,
            TransformResult {
                kind: TransformKind::Pending,
                text: "a".into(),
            }
        );
    }

    #[test]
    fn apply_telex_tone() {
        let result = apply("a", 's', InputMethod::Telex, ToneStyle::Modern);
        assert_eq!(
            result,
            TransformResult {
                kind: TransformKind::Applied,
                text: "á".into(),
            }
        );
    }
}
