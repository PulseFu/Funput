//! Shared helpers for integration tests.

#![allow(dead_code)]

use funput_core::{apply, InputMethod, ToneStyle, TransformKind};

pub fn type_keys(method: InputMethod, keys: &str) -> String {
    let mut buffer = String::new();
    for key in keys.chars() {
        buffer = apply(&buffer, key, method, ToneStyle::Traditional).text;
    }
    buffer
}

/// Type each space-separated chunk as its own syllable (simulates word-boundary clear).
pub fn type_words(method: InputMethod, text: &str) -> String {
    text.split(' ')
        .map(|word| type_keys(method, word))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn type_keys_with_kinds(method: InputMethod, keys: &str) -> (String, Vec<TransformKind>) {
    let mut buffer = String::new();
    let mut kinds = Vec::new();
    for key in keys.chars() {
        let result = apply(&buffer, key, method, ToneStyle::Traditional);
        kinds.push(result.kind);
        buffer = result.text;
    }
    (buffer, kinds)
}
