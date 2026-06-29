//! Persistent configuration for `funput-term`.
//!
//! Reads the project's canonical settings file —
//! `dirs::config_dir()/Funput/settings.json` — so on Linux/Windows the terminal
//! inherits the system IME's preferences, and on macOS it uses its own file at the
//! same well-known path. The on-disk schema (camelCase) mirrors the platform
//! settings readers (`platforms/windows/src/settings.rs`,
//! `platforms/linux/settings-gtk/src/settings.rs`); we keep a local copy rather
//! than depend on those platform-only crates.
//!
//! Resolution precedence is **CLI flag > env var > settings.json > built-in
//! default**. The file and env layers live here; the CLI layer is applied by the
//! caller (`main.rs`).
//!
//! The parse, env-overlay, and engine-apply steps are pure seams (no real I/O), so
//! they are unit-tested directly.

use std::path::PathBuf;

use funput_core::{InputMethod, ToneStyle};
use funput_engine::Engine;
use serde::Deserialize;

use crate::term::DEFAULT_VI_CURSOR_COLOR;

/// `Ctrl-\` (0x1c) — the default Vietnamese on/off toggle byte.
pub const DEFAULT_TOGGLE: u8 = 0x1c;

/// `Ctrl-^` (0x1e) — the default key to cycle Telex↔VNI. Almost never used by
/// shells or readline, so it is safe to claim by default; set `FUNPUT_CYCLE_METHOD`
/// to another key, or to `off`/`none` to disable.
pub const DEFAULT_CYCLE_METHOD: Option<u8> = Some(0x1e);

/// Input method as it serializes in `settings.json` (`"telex"`/`"vni"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Method {
    Telex,
    #[default]
    Vni,
}

impl From<Method> for InputMethod {
    fn from(m: Method) -> Self {
        match m {
            Method::Telex => InputMethod::Telex,
            Method::Vni => InputMethod::Vni,
        }
    }
}

/// Tone-mark placement as it serializes in `settings.json`
/// (`"traditional"`/`"modern"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ToneStyleCfg {
    #[default]
    Traditional,
    Modern,
}

impl From<ToneStyleCfg> for ToneStyle {
    fn from(t: ToneStyleCfg) -> Self {
        match t {
            ToneStyleCfg::Traditional => ToneStyle::Traditional,
            ToneStyleCfg::Modern => ToneStyle::Modern,
        }
    }
}

/// A text-expansion shortcut (gõ tắt) as stored in `settings.json`.
#[derive(Debug, Clone, Deserialize)]
struct Shortcut {
    trigger: String,
    expansion: String,
}

/// The subset of the on-disk settings that `funput-term` can act on. Every field
/// has a serde default so older or partial files still load, and so an empty
/// object (`{}`) yields the canonical defaults — the single source of truth.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileSettings {
    #[serde(default)]
    method: Method,
    #[serde(default)]
    tone_style: ToneStyleCfg,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default = "default_true")]
    smart_restore: bool,
    #[serde(default = "default_true")]
    eager_restore: bool,
    #[serde(default)]
    spell_check: bool,
    #[serde(default)]
    auto_capitalize: bool,
    #[serde(default)]
    shortcuts: Vec<Shortcut>,
}

fn default_true() -> bool {
    true
}

/// Resolved, engine-ready configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermConfig {
    pub method: InputMethod,
    pub tone_style: ToneStyle,
    /// Whether composition starts on (VI) or off (EN).
    pub enabled: bool,
    pub smart_restore: bool,
    pub eager_restore: bool,
    pub spell_check: bool,
    pub auto_capitalize: bool,
    pub shortcuts: Vec<(String, String)>,
    /// The byte that toggles VI/EN.
    pub toggle: u8,
    /// The byte that cycles Telex↔VNI at runtime, or `None` to disable.
    pub cycle_method: Option<u8>,
    /// Cursor color shown while composing (VI).
    pub vi_cursor_color: String,
}

impl Default for TermConfig {
    fn default() -> Self {
        // Parsing an empty object routes through the serde defaults so the
        // defaults are defined in exactly one place.
        from_json("{}")
    }
}

impl From<FileSettings> for TermConfig {
    fn from(f: FileSettings) -> Self {
        TermConfig {
            method: f.method.into(),
            tone_style: f.tone_style.into(),
            enabled: f.enabled,
            smart_restore: f.smart_restore,
            eager_restore: f.eager_restore,
            spell_check: f.spell_check,
            auto_capitalize: f.auto_capitalize,
            shortcuts: f
                .shortcuts
                .into_iter()
                .map(|s| (s.trigger, s.expansion))
                .collect(),
            toggle: DEFAULT_TOGGLE,
            cycle_method: DEFAULT_CYCLE_METHOD,
            vi_cursor_color: DEFAULT_VI_CURSOR_COLOR.to_string(),
        }
    }
}

/// Parse `settings.json` contents into a resolved config. Malformed or empty input
/// falls back to the canonical defaults.
pub fn from_json(s: &str) -> TermConfig {
    serde_json::from_str::<FileSettings>(s)
        .or_else(|_| serde_json::from_str::<FileSettings>("{}"))
        .expect("an empty JSON object is always valid FileSettings")
        .into()
}

impl TermConfig {
    /// Overlay environment-variable overrides (higher precedence than the file).
    /// The getter is injected so this is unit-testable without touching the
    /// process environment.
    pub fn apply_env(mut self, get: impl Fn(&str) -> Option<String>) -> Self {
        if let Some(m) = get("FUNPUT_METHOD").as_deref().and_then(parse_method) {
            self.method = m;
        }
        if let Some(t) = get("FUNPUT_TONE_STYLE").as_deref().and_then(parse_tone) {
            self.tone_style = t;
        }
        if let Some(e) = get("FUNPUT_ENABLED").as_deref().and_then(parse_bool) {
            self.enabled = e;
        }
        if let Some(b) = get("FUNPUT_TOGGLE").as_deref().and_then(parse_toggle) {
            self.toggle = b;
        }
        if let Some(spec) = get("FUNPUT_CYCLE_METHOD") {
            // `off`/`none`/empty disables; otherwise parse a key spec (invalid
            // specs are ignored, keeping the previous value).
            match spec.trim().to_ascii_lowercase().as_str() {
                "" | "off" | "none" | "disable" | "disabled" => self.cycle_method = None,
                _ => {
                    if let Some(b) = parse_toggle(&spec) {
                        self.cycle_method = Some(b);
                    }
                }
            }
        }
        if let Some(c) = get("FUNPUT_CURSOR_COLOR_VI").filter(|c| !c.is_empty()) {
            self.vi_cursor_color = c;
        }
        self
    }

    /// Push the engine-affecting options into a fresh engine. Replaces the whole
    /// shortcut table so repeated calls stay consistent.
    pub fn apply_to(&self, engine: &mut Engine) {
        engine.set_method(self.method);
        engine.set_tone_style(self.tone_style);
        engine.set_smart_restore(self.smart_restore);
        engine.set_eager_restore(self.eager_restore);
        engine.set_spell_check(self.spell_check);
        engine.set_auto_capitalize(self.auto_capitalize);
        engine.clear_shortcuts();
        for (trigger, expansion) in &self.shortcuts {
            engine.add_shortcut(trigger.clone(), expansion.clone());
        }
    }
}

/// Path to the settings file: `$FUNPUT_CONFIG` if set, else the canonical
/// `dirs::config_dir()/Funput/settings.json`.
pub fn settings_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("FUNPUT_CONFIG") {
        return Some(PathBuf::from(p));
    }
    dirs::config_dir().map(|d| d.join("Funput").join("settings.json"))
}

/// Load the effective config: settings file (or defaults if missing/unreadable)
/// with environment overrides applied.
pub fn load() -> TermConfig {
    let from_file = settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| from_json(&s))
        .unwrap_or_default();
    from_file.apply_env(|k| std::env::var(k).ok())
}

fn parse_method(s: &str) -> Option<InputMethod> {
    match s.to_ascii_lowercase().as_str() {
        "telex" => Some(InputMethod::Telex),
        "vni" => Some(InputMethod::Vni),
        _ => None,
    }
}

fn parse_tone(s: &str) -> Option<ToneStyle> {
    match s.to_ascii_lowercase().as_str() {
        "traditional" | "old" => Some(ToneStyle::Traditional),
        "modern" | "new" => Some(ToneStyle::Modern),
        _ => None,
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Parse a toggle-key spec into its control byte: `ctrl-\`, `ctrl-^`, `ctrl-]`,
/// `ctrl-space`, etc. A control byte is the key's ASCII value masked with 0x1f
/// (`\` 0x5c → 0x1c), and `space` maps to NUL (0x00, i.e. `Ctrl-Space`).
fn parse_toggle(s: &str) -> Option<u8> {
    let key = s.trim().to_ascii_lowercase();
    let key = key
        .strip_prefix("ctrl-")
        .or_else(|| key.strip_prefix("c-"))?;
    match key {
        "space" | "spc" => Some(0x00),
        k if k.chars().count() == 1 => {
            let ch = k.chars().next().unwrap();
            ch.is_ascii().then_some((ch as u8) & 0x1f)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_camelcase_file() {
        let json = r#"{
            "method": "telex",
            "toneStyle": "modern",
            "enabled": true,
            "smartRestore": true,
            "eagerRestore": false,
            "spellCheck": true,
            "autoCapitalize": true,
            "shortcuts": [{ "trigger": "vn", "expansion": "Việt Nam" }]
        }"#;
        let c = from_json(json);
        assert_eq!(c.method, InputMethod::Telex);
        assert_eq!(c.tone_style, ToneStyle::Modern);
        assert!(c.enabled);
        assert!(!c.eager_restore);
        assert!(c.spell_check);
        assert!(c.auto_capitalize);
        assert_eq!(
            c.shortcuts,
            vec![("vn".to_string(), "Việt Nam".to_string())]
        );
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        // Empty object: VNI, traditional, enabled, restore flags on, others off.
        let c = from_json("{}");
        assert_eq!(c, TermConfig::default());
        assert_eq!(c.method, InputMethod::Vni);
        assert_eq!(c.tone_style, ToneStyle::Traditional);
        assert!(c.enabled && c.smart_restore && c.eager_restore);
        assert!(!c.spell_check && !c.auto_capitalize);
        assert_eq!(c.toggle, DEFAULT_TOGGLE);
        assert_eq!(c.vi_cursor_color, DEFAULT_VI_CURSOR_COLOR);
    }

    #[test]
    fn malformed_json_falls_back_to_defaults() {
        assert_eq!(from_json("not json"), TermConfig::default());
    }

    #[test]
    fn env_overrides_file_values() {
        let base = from_json(r#"{ "method": "vni", "enabled": true }"#);
        let env = |k: &str| match k {
            "FUNPUT_METHOD" => Some("telex".to_string()),
            "FUNPUT_ENABLED" => Some("false".to_string()),
            "FUNPUT_TOGGLE" => Some("ctrl-^".to_string()),
            _ => None,
        };
        let c = base.apply_env(env);
        assert_eq!(c.method, InputMethod::Telex); // env beat the file
        assert!(!c.enabled);
        assert_eq!(c.toggle, 0x1e); // '^' (0x5e) & 0x1f
    }

    #[test]
    fn env_ignores_unset_and_invalid() {
        let base = from_json(r#"{ "method": "telex" }"#);
        let c = base.clone().apply_env(|_| None);
        assert_eq!(c, base); // nothing set → unchanged
        let c = base.apply_env(|k| (k == "FUNPUT_METHOD").then(|| "garbage".to_string()));
        assert_eq!(c.method, InputMethod::Telex); // invalid value ignored
    }

    #[test]
    fn cycle_method_key_defaults_and_overrides() {
        assert_eq!(from_json("{}").cycle_method, DEFAULT_CYCLE_METHOD);
        // A custom key spec.
        let c =
            from_json("{}").apply_env(|k| (k == "FUNPUT_CYCLE_METHOD").then(|| "ctrl-]".into()));
        assert_eq!(c.cycle_method, Some(0x1d));
        // Disabled.
        let c = from_json("{}").apply_env(|k| (k == "FUNPUT_CYCLE_METHOD").then(|| "off".into()));
        assert_eq!(c.cycle_method, None);
    }

    #[test]
    fn toggle_spec_parsing() {
        assert_eq!(parse_toggle("ctrl-\\"), Some(0x1c));
        assert_eq!(parse_toggle("Ctrl-Space"), Some(0x00));
        assert_eq!(parse_toggle("c-]"), Some(0x1d));
        assert_eq!(parse_toggle("nope"), None);
    }
}
