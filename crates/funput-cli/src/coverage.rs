//! Round-trip coverage check: for each Vietnamese syllable in a corpus, encode
//! it to keystrokes, type it back through the engine, and compare.
//!
//! A syllable counts as correct if it reproduces under **either** tone style —
//! `hòa` and `hoà` are both valid spellings, so producing either is correct typing.
//! Smart-restore is off to measure pure composition.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use funput_core::{ToneStyle, is_complete_syllable};

use crate::encode::encode;
use crate::sim::{Method, SimConfig, simulate_with};

/// A token is a testable Vietnamese syllable when (a) every letter is in the
/// Vietnamese alphabet (`f`/`j`/`w`/`z` and non-letters are not) and (b) it forms a
/// structurally valid syllable. This excludes acronyms (AIDS), chemical symbols
/// (Ar/As), and foreign words (becgiê) so the score reflects real Vietnamese.
fn is_vietnamese_syllable(token: &str) -> bool {
    !token.is_empty()
        && token.chars().all(|c| {
            let l = c.to_lowercase().next().unwrap_or(c);
            l.is_alphabetic() && !matches!(l, 'f' | 'j' | 'w' | 'z')
        })
        && is_complete_syllable(token)
}

/// Compose `keys` with smart-restore off (pure composition), traditional tone style —
/// used to show what the engine produced for a mismatch.
fn composed(keys: &str, method: Method) -> String {
    let config = SimConfig {
        method,
        tone_style: ToneStyle::Traditional,
        smart_restore: false,
        spell_check: false,
    };
    simulate_with(config, keys).app_text
}

/// Read a corpus (one word per line; words may have multiple space-separated
/// syllables), split into unique Vietnamese syllables.
fn load_syllables(path: &Path) -> std::io::Result<BTreeSet<String>> {
    let text = fs::read_to_string(path)?;
    let mut set = BTreeSet::new();
    for line in text.lines() {
        for token in line.split_whitespace() {
            let trimmed = token.trim_matches(|c: char| !c.is_alphabetic());
            if is_vietnamese_syllable(trimmed) {
                set.insert(trimmed.to_string());
            }
        }
    }
    Ok(set)
}

/// Does `syllable` reproduce when encoded and typed back under `method`?
/// Checks both tone styles.
fn round_trips(syllable: &str, method: Method) -> bool {
    let keys = encode(syllable, method);
    [ToneStyle::Traditional, ToneStyle::Modern]
        .iter()
        .any(|&style| {
            let config = SimConfig {
                method,
                tone_style: style,
                smart_restore: false,
                spell_check: false,
            };
            simulate_with(config, &keys).app_text == syllable
        })
}

struct MethodResult {
    label: &'static str,
    total: usize,
    covered: usize,
    mismatches: Vec<(String, String)>, // (syllable, produced under traditional)
}

fn evaluate(
    syllables: &[String],
    method: Method,
    label: &'static str,
    keep_mismatches: usize,
) -> MethodResult {
    let mut covered = 0;
    let mut mismatches = Vec::new();
    for s in syllables {
        if round_trips(s, method) {
            covered += 1;
        } else {
            if mismatches.len() < keep_mismatches {
                mismatches.push((s.clone(), composed(&encode(s, method), method)));
            }
        }
    }
    MethodResult {
        label,
        total: syllables.len(),
        covered,
        mismatches,
    }
}

fn pct(covered: usize, total: usize) -> f64 {
    if total == 0 {
        100.0
    } else {
        covered as f64 / total as f64 * 100.0
    }
}

/// Run the round-trip coverage check and print a report.
pub fn run(
    corpus: &Path,
    json: bool,
    show_mismatches: usize,
    limit: Option<usize>,
) -> std::io::Result<()> {
    let mut syllables: Vec<String> = load_syllables(corpus)?.into_iter().collect();
    if let Some(n) = limit {
        syllables.truncate(n);
    }

    let telex = evaluate(&syllables, Method::Telex, "Telex", show_mismatches);
    let vni = evaluate(&syllables, Method::Vni, "VNI", show_mismatches);

    if json {
        print_json(corpus, &syllables, &telex, &vni);
    } else {
        print_human(corpus, &syllables, &telex, &vni, show_mismatches);
    }
    Ok(())
}

fn print_human(
    corpus: &Path,
    syllables: &[String],
    telex: &MethodResult,
    vni: &MethodResult,
    show_mismatches: usize,
) {
    println!(
        "Funput coverage — {} syllables (corpus: {})",
        syllables.len(),
        corpus.display()
    );
    for r in [telex, vni] {
        println!(
            "  {:<6} {:.2}%  ({} wrong of {})",
            r.label,
            pct(r.covered, r.total),
            r.total - r.covered,
            r.total
        );
    }
    if show_mismatches > 0 {
        for r in [telex, vni] {
            if r.mismatches.is_empty() {
                continue;
            }
            println!("\nSample mismatches ({}): syllable → produced", r.label);
            for (syl, produced) in &r.mismatches {
                println!("  {syl} → {produced}");
            }
        }
    }
}

fn print_json(corpus: &Path, syllables: &[String], telex: &MethodResult, vni: &MethodResult) {
    let one = |r: &MethodResult| {
        format!(
            "{{\"total\":{},\"covered\":{},\"coverage\":{:.4}}}",
            r.total,
            r.covered,
            pct(r.covered, r.total) / 100.0
        )
    };
    println!(
        "{{\"corpus\":{:?},\"syllables\":{},\"telex\":{},\"vni\":{}}}",
        corpus.display().to_string(),
        syllables.len(),
        one(telex),
        one(vni)
    );
}
