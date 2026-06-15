//! Key classification per input method.
//!
//! Each method (VNI, Telex, …) maps a keystroke to a shared [`KeyAction`]; the
//! composition pipeline in [`crate::composition`] is method-agnostic and only
//! ever sees [`KeyAction`]s. Adding a method means adding a classifier module
//! here — nothing downstream changes.

pub mod vni;

use crate::unicode::marks::Tone;
use crate::unicode::shapes::VowelShape;

/// What a keystroke means, independent of input method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Đ stroke (`d` → `đ`).
    Stroke,
    /// Tone mark (sắc / huyền / hỏi / ngã / nặng).
    Tone(Tone),
    /// Vowel shape (mũ / móc / trần).
    Shape(VowelShape),
    /// Ordinary character — appended as-is.
    Normal,
}
