//! VNI key classification.
//!
//! ## Revert (Cách A — gõ đúp số)
//!
//! Gõ lại **cùng phím modifier** trên đúng mục tiêu → bỏ **một lớp** modifier:
//!
//! | Chuỗi gõ | Kết quả | Ghi chú |
//! |----------|---------|---------|
//! | `a11` | `a` | Bỏ sắc |
//! | `a66` | `a` | Bỏ mũ |
//! | `d99` | `d` | Bỏ gạch ngang |
//! | `a611` | `â` | Bỏ sắc, giữ mũ |
//! | `a12` | `à` | Khác tone → thay tone, không revert |
//! | `hoa22` | `hoa` | Bỏ huyền trên `a` |
//! | `uo77` | `uo` | Bỏ móc compound `ươ` |
//!
//! Trả `TransformKind::Reverted` khi revert thành công.

use crate::unicode::marks::{Tone, VowelShape};

/// Action implied by a VNI keystroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VniKeyAction {
    Stroke,
    Tone(Tone),
    Shape(VowelShape),
    Normal,
}

/// Map digit keys 1–5 to tone marks.
pub fn tone_from_digit(key: char) -> Option<Tone> {
    match key {
        '1' => Some(Tone::Sac),
        '2' => Some(Tone::Huyen),
        '3' => Some(Tone::Hoi),
        '4' => Some(Tone::Nga),
        '5' => Some(Tone::Nang),
        _ => None,
    }
}

/// Map digit keys 6–8 to vowel shape modifiers.
pub fn shape_from_digit(key: char) -> Option<VowelShape> {
    match key {
        '6' => Some(VowelShape::Circumflex),
        '7' => Some(VowelShape::Horn),
        '8' => Some(VowelShape::Breve),
        _ => None,
    }
}

/// Classify a VNI keystroke.
pub fn classify_key(key: char) -> VniKeyAction {
    match key {
        '9' => VniKeyAction::Stroke,
        '1'..='5' => VniKeyAction::Tone(tone_from_digit(key).expect("digit 1-5")),
        '6'..='8' => VniKeyAction::Shape(shape_from_digit(key).expect("digit 6-8")),
        _ => VniKeyAction::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_stroke_and_tones() {
        assert_eq!(classify_key('9'), VniKeyAction::Stroke);
        assert_eq!(classify_key('1'), VniKeyAction::Tone(Tone::Sac));
        assert_eq!(classify_key('5'), VniKeyAction::Tone(Tone::Nang));
        assert_eq!(classify_key('m'), VniKeyAction::Normal);
    }

    #[test]
    fn classify_shapes() {
        assert_eq!(classify_key('6'), VniKeyAction::Shape(VowelShape::Circumflex));
        assert_eq!(classify_key('7'), VniKeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key('8'), VniKeyAction::Shape(VowelShape::Breve));
    }
}
